use astro_lib::atmosphere::{
    equilibrium_temperature, greenhouse_effect, surface_pressure_estimate, surface_temperature,
};
use astro_lib::binary::{
    binary_orbital_period, combined_luminosity, p_type_critical_radius, s_type_critical_radius,
};
use astro_lib::habitability::{is_habitable_tilt, is_in_habitable_zone};
use astro_lib::moon::{
    angular_size_arcmin, hill_sphere_au, hill_sphere_planet_radii, is_moon_orbit_valid,
    moon_gravity, moon_mass, moon_orbital_period_days, roche_limit_planet_radii,
    stable_orbit_limit,
};
use astro_lib::orbit::{
    aphelion, orbital_period, orbital_velocity, perihelion, polar_circle, tropic_latitude,
};
use astro_lib::planet::{
    density, escape_velocity, gravity, has_solid_surface, planet_type, surface_area, volume,
    PlanetType,
};
use astro_lib::star::{
    frost_line, habitable_zone, lifetime, luminosity, peak_wavelength, radius, spectral_class,
    star_display_color, temperature,
};
use astro_lib::tidal::{is_tidally_locked, planet_lock_time_years, RIGIDITY_ROCKY};
use crate::i18n::*;
use leptos::prelude::*;

use super::planets::{host_tag, star_inputs, PlanetSigs, HOST_B, HOST_BOTH};
use super::storage::{load_moon_ids, load_planet_ids, ls_f64, ls_f64_dyn, ls_string_dyn, ls_u32_dyn};
use super::ui::{fmt_result, fmt_years, BoolRow, ResultRow, SectionHeader};

const R_EARTH_KM: f64 = 6_371.0;

// ── SVG geometry ──────────────────────────────────────────────────────────────

/// Top-view map canvas.
const MAP_W: f64 = 640.0;
const MAP_H: f64 = 420.0;
const CX: f64 = 320.0;
const CY: f64 = 210.0;
const R_MIN: f64 = 16.0;
const R_MAX: f64 = 196.0;

/// Moon inset canvas.
const INSET_W: f64 = 640.0;
const INSET_H: f64 = 100.0;
const X0: f64 = 56.0;
const X1: f64 = 600.0;
const AXIS_Y: f64 = 42.0;

/// Maps a physical distance onto pixels logarithmically, so orbits spanning
/// several orders of magnitude (0.1 AU star offsets to 20 AU companions)
/// stay readable on one disc.
struct LogScale {
    lo_log: f64,
    span_log: f64,
    px_min: f64,
    px_max: f64,
}

impl LogScale {
    fn new(lo: f64, hi: f64, px_min: f64, px_max: f64) -> Self {
        let lo = lo.max(1e-6);
        let hi = hi.max(lo * 1.01);
        Self {
            lo_log: lo.log10(),
            span_log: hi.log10() - lo.log10(),
            px_min,
            px_max,
        }
    }

    fn px(&self, v: f64) -> f64 {
        let t = ((v.max(1e-6).log10() - self.lo_log) / self.span_log).clamp(0.0, 1.0);
        self.px_min + t * (self.px_max - self.px_min)
    }
}

fn ring(cx: f64, cy: f64, r: f64, stroke: &str, width: f64, dash: &str, opacity: f64) -> String {
    let dash_attr = if dash.is_empty() {
        String::new()
    } else {
        format!(" stroke-dasharray='{dash}'")
    };
    format!(
        "<circle cx='{cx:.1}' cy='{cy:.1}' r='{r:.1}' fill='none' stroke='{stroke}' \
         stroke-width='{width}' stroke-opacity='{opacity}'{dash_attr}/>"
    )
}

/// Splits a radial band at `c`. `stable_inside` says which side of `c`
/// hosts stable orbits (around one star: inside the critical radius;
/// circumbinary: outside). Returns `(stable, unstable)` sub-bands.
fn split_band(
    lo: f64,
    hi: f64,
    c: f64,
    stable_inside: bool,
) -> (Option<(f64, f64)>, Option<(f64, f64)>) {
    if stable_inside {
        ((lo < c).then(|| (lo, hi.min(c))), (hi > c).then(|| (lo.max(c), hi)))
    } else {
        ((hi > c).then(|| (lo.max(c), hi)), (lo < c).then(|| (lo, hi.min(c))))
    }
}

/// Green band between two radii (stroke-width trick) plus edge rings.
fn annulus(
    scale: &LogScale,
    cx: f64,
    cy: f64,
    lo: f64,
    hi: f64,
    fill_opacity: f64,
    edge_opacity: f64,
    dash: &str,
) -> String {
    let r_in = scale.px(lo);
    let r_out = scale.px(hi);
    let mut s = format!(
        "<circle cx='{cx:.1}' cy='{cy:.1}' r='{:.1}' fill='none' stroke='#34d399' \
         stroke-width='{:.1}' stroke-opacity='{fill_opacity}'/>",
        (r_in + r_out) / 2.0,
        (r_out - r_in).max(1.0),
    );
    s.push_str(&ring(cx, cy, r_in, "#34d399", 1.0, dash, edge_opacity));
    s.push_str(&ring(cx, cy, r_out, "#34d399", 1.0, dash, edge_opacity));
    s
}

/// Habitable zone split at a stability limit: solid green where orbits
/// survive, dimmed & dashed where they don't.
fn hz_annuli(
    scale: &LogScale,
    cx: f64,
    cy: f64,
    hz: &astro_lib::star::HabitableZone,
    crit: f64,
    stable_inside: bool,
) -> String {
    let (stable, unstable) = split_band(hz.inner, hz.outer, crit, stable_inside);
    let mut s = String::new();
    if let Some((lo, hi)) = stable {
        s.push_str(&annulus(scale, cx, cy, lo, hi, 0.12, 0.35, ""));
    }
    if let Some((lo, hi)) = unstable {
        s.push_str(&annulus(scale, cx, cy, lo, hi, 0.04, 0.18, "3 4"));
    }
    s
}

fn star_glyph(x: f64, y: f64, r: f64, color: &str) -> String {
    format!(
        "<circle cx='{x:.1}' cy='{y:.1}' r='{:.1}' fill='{color}' opacity='0.15'/>\
         <circle cx='{x:.1}' cy='{y:.1}' r='{r:.1}' fill='{color}'/>",
        r * 2.4
    )
}

/// Escapes user text (planet/moon names) for embedding in the SVG string.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

fn fmt_ang(total_min: f64) -> String {
    let deg = (total_min / 60.0) as u32;
    let min = total_min % 60.0;
    if deg > 0 {
        format!("{deg}\u{b0}  {min:.0}'")
    } else {
        format!("{min:.1}'")
    }
}

/// Legend entry: colored dot + label.
fn chip(color: &str, label: AnyView) -> impl IntoView {
    view! {
        <span class="inline-flex items-center gap-1.5 text-[10px]">
            <span class="w-2 h-2 rounded-full shrink-0" style=format!("background:{color}") />
            <span class="text-hint">{label}</span>
        </span>
    }
}

/// Per-moon reactive state, read from the keys the Moon tab persists.
#[derive(Clone, Copy)]
struct MoonSigs {
    parent: RwSignal<u32>,
    name: RwSignal<String>,
    radius: RwSignal<f64>,
    density: RwSignal<f64>,
    distance: RwSignal<f64>,
}

/// Kepler orbit r(θ) pushed through the log scale, so the perihelion/aphelion
/// asymmetry survives the distorted radial axis.
fn orbit_path(a: f64, e: f64, scale: &LogScale, cx: f64, cy: f64) -> String {
    let mut path = String::new();
    for i in 0..=120u32 {
        let th = (i as f64 * 3.0).to_radians();
        let r_au = a * (1.0 - e * e) / (1.0 + e * th.cos());
        let r_px = scale.px(r_au);
        let x = cx + r_px * th.cos();
        let y = cy - r_px * th.sin();
        let cmd = if i == 0 { 'M' } else { 'L' };
        path.push_str(&format!("{cmd}{x:.1} {y:.1}"));
    }
    path
}

/// A planet's orbit + marker + name label around the given center.
fn planet_orbit_svg(idx: usize, a: f64, e: f64, name: &str, scale: &LogScale, cx: f64, cy: f64) -> String {
    let mut svg = format!(
        "<path d='{}Z' fill='none' stroke='#38bdf8' stroke-width='1.6' \
         stroke-opacity='0.9'/>",
        orbit_path(a, e, scale, cx, cy)
    );
    let th = (135.0 - 47.0 * idx as f64).to_radians();
    let r_au = a * (1.0 - e * e) / (1.0 + e * th.cos());
    let r_px = scale.px(r_au);
    let x = cx + r_px * th.cos();
    let y = cy - r_px * th.sin();
    let label: String = name.chars().take(14).collect();
    svg.push_str(&format!(
        "<circle cx='{x:.1}' cy='{y:.1}' r='5' fill='#38bdf8' \
         stroke='#0f172a' stroke-width='1.5'/>\
         <text x='{x:.1}' y='{:.1}' font-size='10' fill='#cbd5e1' \
         text-anchor='middle'>{}</text>",
        y - 10.0,
        esc(&label),
    ));
    svg
}

/// Horizontal log-scale strip of one planet's moon system.
fn moon_inset_svg(pm: f64, pr: f64, pd: f64, oa: f64, sm: f64, entries: &[(f64, f64, f64)]) -> String {
    let h_rp = hill_sphere_planet_radii(oa, pm, sm, pr);
    let stab = stable_orbit_limit(h_rp);
    let roche_max = entries
        .iter()
        .map(|(_, d, _)| roche_limit_planet_radii(pd, *d))
        .fold(1.5, f64::max);
    let dist_max = entries.iter().map(|e| e.2).fold(0.0_f64, f64::max);
    let hi = stab.max(roche_max).max(dist_max) * 1.35;
    let scale = LogScale::new(1.0, hi.max(2.0), X0, X1);

    let mut svg = String::new();

    svg.push_str(&format!(
        "<line x1='{X0}' y1='{AXIS_Y}' x2='{X1}' y2='{AXIS_Y}' \
         stroke='#475569' stroke-width='1'/>"
    ));

    // Roche zone: anything orbiting here is torn into rings.
    let rx = scale.px(roche_max);
    svg.push_str(&format!(
        "<rect x='{X0}' y='{:.1}' width='{:.1}' height='20' fill='#f87171' \
         fill-opacity='0.15'/>\
         <line x1='{rx:.1}' y1='{:.1}' x2='{rx:.1}' y2='{:.1}' stroke='#f87171' \
         stroke-width='1.2' stroke-dasharray='3 3' stroke-opacity='0.8'/>\
         <text x='{rx:.1}' y='{:.1}' font-size='9' fill='#f87171' \
         text-anchor='middle' opacity='0.9'>{roche_max:.1}</text>",
        AXIS_Y - 10.0,
        (rx - X0).max(0.0),
        AXIS_Y - 14.0,
        AXIS_Y + 14.0,
        AXIS_Y + 28.0,
    ));

    // Stable orbit limit.
    let sx = scale.px(stab);
    svg.push_str(&format!(
        "<line x1='{sx:.1}' y1='{:.1}' x2='{sx:.1}' y2='{:.1}' stroke='#34d399' \
         stroke-width='1.2' stroke-dasharray='3 3' stroke-opacity='0.8'/>\
         <text x='{sx:.1}' y='{:.1}' font-size='9' fill='#34d399' \
         text-anchor='middle' opacity='0.9'>{stab:.0}</text>",
        AXIS_Y - 14.0,
        AXIS_Y + 14.0,
        AXIS_Y + 28.0,
    ));

    // Parent planet at the left edge (its surface sits at 1 Rp).
    svg.push_str(&format!(
        "<circle cx='{:.1}' cy='{AXIS_Y}' r='15' fill='#38bdf8' fill-opacity='0.9'/>",
        X0 - 11.0
    ));

    // Moons: dot size follows physical radius, index above, distance below.
    for (idx, (mr, _, dist)) in entries.iter().enumerate() {
        let x = scale.px(*dist);
        let r_px = (2.0 + mr * 8.0).clamp(2.0, 7.0);
        svg.push_str(&format!(
            "<circle cx='{x:.1}' cy='{AXIS_Y}' r='{r_px:.1}' fill='#f1f5f9'/>\
             <text x='{x:.1}' y='{:.1}' font-size='9' fill='#cbd5e1' \
             text-anchor='middle'>{}</text>\
             <text x='{x:.1}' y='{:.1}' font-size='9' fill='#64748b' \
             text-anchor='middle'>{dist:.0}</text>",
            AXIS_Y - 18.0,
            idx + 1,
            AXIS_Y + 17.0,
        ));
    }

    // Axis unit.
    svg.push_str(&format!(
        "<text x='{:.1}' y='{:.1}' font-size='9' fill='#64748b'>Rp</text>",
        X1 + 8.0,
        AXIS_Y + 3.0,
    ));

    format!(
        "<svg viewBox='0 0 {INSET_W} {INSET_H}' class='w-full h-auto' role='img'>{svg}</svg>"
    )
}

#[component]
pub fn SystemTab() -> impl IntoView {
    let i18n = use_i18n();

    // ── star configuration ──────────────────────────────────────────────────
    let stars = star_inputs();
    let star_mass = stars.mass_a;
    let mass_b = stars.mass_b;
    let binary_mode = stars.binary;
    let bin_sep = ls_f64("binary_separation", 20.0);
    let bin_ecc = ls_f64("binary_eccentricity", 0.4);

    // ── globals shared by every planet ──────────────────────────────────────
    let system_age = ls_f64("system_age_gyr", 4.6);

    // ── planets & moons ─────────────────────────────────────────────────────
    let planets: Vec<PlanetSigs> =
        load_planet_ids().into_iter().map(PlanetSigs::new).collect();
    let first_planet_id = planets[0].id;

    let moons: Vec<MoonSigs> = load_moon_ids()
        .into_iter()
        .map(|id| MoonSigs {
            parent: ls_u32_dyn(format!("moon_{id}_parent"), first_planet_id),
            name: ls_string_dyn(format!("moon_{id}_name"), String::new()),
            radius: ls_f64_dyn(format!("moon_{id}_radius"), 0.273),
            density: ls_f64_dyn(format!("moon_{id}_density"), 0.606),
            distance: ls_f64_dyn(format!("moon_{id}_dist"), 60.27),
        })
        .collect();

    // Localized fallbacks for unnamed bodies.
    let planet_name = {
        let planets = planets.clone();
        move |id: u32| -> String {
            let idx = planets.iter().position(|p| p.id == id).unwrap_or(0);
            let nm = planets[idx].name.get();
            if nm.is_empty() {
                format!("{} {}", t_string!(i18n, planet), idx + 1)
            } else {
                nm
            }
        }
    };
    let moon_display_name = move |m: &MoonSigs, idx: usize| -> String {
        let nm = m.name.get();
        if nm.is_empty() {
            format!("{} {}", t_string!(i18n, moon), idx + 1)
        } else {
            nm
        }
    };

    // ── system map ──────────────────────────────────────────────────────────
    let map_planets = planets.clone();
    let map_planet_name = planet_name.clone();
    let map_svg = move || -> Result<String, String> {
        let ma = star_mass.get();
        let lum_a = luminosity(ma).map_err(|e| e.to_string())?;
        let temp_a = temperature(ma).map_err(|e| e.to_string())?;
        let binary = binary_mode.get();
        let color_a = star_display_color(spectral_class(temp_a));

        // Clamp e for drawing only — an e ≥ 1 orbit has no closed shape.
        let orbits: Vec<(f64, f64, u32, String)> = map_planets
            .iter()
            .map(|p| {
                (
                    p.semi_major.get().max(1e-4),
                    p.eccentricity.get().clamp(0.0, 0.95),
                    if binary { p.host.get() } else { 0 },
                    map_planet_name(p.id),
                )
            })
            .collect();

        let mut radii: Vec<f64> = Vec::new();
        for (a, e, _, _) in &orbits {
            radii.push(perihelion(*a, *e));
            radii.push(aphelion(*a, *e));
        }

        let mut svg = String::new();

        if binary {
            let mb = mass_b.get();
            let lum_b = luminosity(mb).map_err(|e| e.to_string())?;
            let temp_b = temperature(mb).map_err(|e| e.to_string())?;
            let color_b = star_display_color(spectral_class(temp_b));

            let sep = bin_sep.get().max(1e-4);
            let e_bin = bin_ecc.get();
            let total = ma + mb;
            let (r_a, r_b) = (sep * mb / total, sep * ma / total);
            let hz_a = habitable_zone(lum_a);
            let hz_b = habitable_zone(lum_b);
            let frost = frost_line(lum_a + lum_b);
            // Max stable orbit around each star; roles swap for star B.
            let crit_a = s_type_critical_radius(sep, e_bin, ma, mb).max(0.0);
            let crit_b = s_type_critical_radius(sep, e_bin, mb, ma).max(0.0);
            let any_circumbinary = orbits.iter().any(|(_, _, h, _)| *h == HOST_BOTH);

            radii.extend([r_a, r_b, sep, hz_a.inner, hz_a.outer, hz_b.inner, hz_b.outer, frost]);
            if crit_a > 0.0 { radii.push(crit_a); }
            if crit_b > 0.0 { radii.push(crit_b); }
            let hz_ab = habitable_zone(lum_a + lum_b);
            let p_crit = p_type_critical_radius(sep, e_bin, ma, mb);
            if any_circumbinary {
                radii.extend([hz_ab.inner, hz_ab.outer, p_crit]);
            }

            let lo = radii.iter().copied().fold(f64::INFINITY, f64::min).max(1e-4) * 0.7;
            let hi = radii.iter().copied().fold(0.0_f64, f64::max) * 1.25;
            let scale = LogScale::new(lo, hi, R_MIN, R_MAX);

            let xa = CX - scale.px(r_a);
            let xb = CX + scale.px(r_b);

            // The stars' own orbits around the barycenter.
            svg.push_str(&ring(CX, CY, scale.px(r_a), "#cbd5e1", 1.0, "2 5", 0.2));
            svg.push_str(&ring(CX, CY, scale.px(r_b), "#cbd5e1", 1.0, "2 5", 0.2));

            // Each star's own HZ, split at its stability limit.
            svg.push_str(&hz_annuli(&scale, xa, CY, &hz_a, crit_a, true));
            svg.push_str(&hz_annuli(&scale, xb, CY, &hz_b, crit_b, true));
            if crit_a > 0.0 {
                svg.push_str(&ring(xa, CY, scale.px(crit_a), "#f87171", 1.2, "4 4", 0.5));
            }
            if crit_b > 0.0 {
                svg.push_str(&ring(xb, CY, scale.px(crit_b), "#f87171", 1.2, "4 4", 0.5));
            }

            // Circumbinary HZ and minimum orbit, only when someone lives there.
            if any_circumbinary {
                svg.push_str(&hz_annuli(&scale, CX, CY, &hz_ab, p_crit, false));
                svg.push_str(&ring(CX, CY, scale.px(p_crit), "#f87171", 1.2, "4 4", 0.6));
            }

            // Frost line of the pair.
            svg.push_str(&ring(CX, CY, scale.px(frost), "#7fb3d5", 1.2, "5 5", 0.6));

            // Planet orbits around their hosts.
            for (idx, (a, e, host, name)) in orbits.iter().enumerate() {
                let cx = match *host {
                    HOST_B => xb,
                    HOST_BOTH => CX,
                    _ => xa,
                };
                svg.push_str(&planet_orbit_svg(idx, *a, *e, name, &scale, cx, CY));
            }

            // Stars with A/B tags.
            svg.push_str(&star_glyph(xa, CY, 8.0, color_a));
            svg.push_str(&star_glyph(xb, CY, 6.5, color_b));
            svg.push_str(&format!(
                "<text x='{xa:.1}' y='{y:.1}' font-size='10' fill='#64748b' \
                 text-anchor='middle'>A</text>\
                 <text x='{xb:.1}' y='{y:.1}' font-size='10' fill='#64748b' \
                 text-anchor='middle'>B</text>",
                y = CY + 22.0,
            ));
        } else {
            let hz = habitable_zone(lum_a);
            let frost = frost_line(lum_a);
            radii.extend([hz.inner, hz.outer, frost]);

            let lo = radii.iter().copied().fold(f64::INFINITY, f64::min).max(1e-4) * 0.7;
            let hi = radii.iter().copied().fold(0.0_f64, f64::max) * 1.25;
            let scale = LogScale::new(lo, hi, R_MIN, R_MAX);

            svg.push_str(&annulus(&scale, CX, CY, hz.inner, hz.outer, 0.12, 0.35, ""));
            svg.push_str(&ring(CX, CY, scale.px(frost), "#7fb3d5", 1.2, "5 5", 0.6));

            for (idx, (a, e, _, name)) in orbits.iter().enumerate() {
                svg.push_str(&planet_orbit_svg(idx, *a, *e, name, &scale, CX, CY));
            }

            svg.push_str(&star_glyph(CX, CY, 9.0, color_a));
        }

        Ok(format!(
            "<svg viewBox='0 0 {MAP_W} {MAP_H}' class='w-full h-auto' role='img'>{svg}</svg>"
        ))
    };

    // ── map legend ──────────────────────────────────────────────────────────
    let legend_planets = planets.clone();
    let legend = move || {
        let binary = binary_mode.get();
        let mut chips: Vec<AnyView> = Vec::new();

        if let Ok(temp_a) = temperature(star_mass.get()) {
            let label = if binary {
                view! { {t!(i18n, star_a)} }.into_any()
            } else {
                view! { {t!(i18n, star)} }.into_any()
            };
            chips.push(chip(star_display_color(spectral_class(temp_a)), label).into_any());
        }
        if binary {
            if let Ok(temp_b) = temperature(mass_b.get()) {
                chips.push(
                    chip(
                        star_display_color(spectral_class(temp_b)),
                        view! { {t!(i18n, star_b)} }.into_any(),
                    )
                    .into_any(),
                );
            }
        }

        // HZ chips mirror the map: solid green for dynamically stable parts,
        // dimmed for parts beyond a stability limit.
        let (mut hz_stable_chip, mut hz_unstable_chip) = (true, false);
        let mut any_circumbinary = false;
        if binary {
            let (ma, mb) = (star_mass.get(), mass_b.get());
            if let (Ok(la), Ok(lb)) = (luminosity(ma), luminosity(mb)) {
                let (sep, e_bin) = (bin_sep.get().max(1e-4), bin_ecc.get());
                let hz_a = habitable_zone(la);
                let hz_b = habitable_zone(lb);
                let crit_a = s_type_critical_radius(sep, e_bin, ma, mb).max(0.0);
                let crit_b = s_type_critical_radius(sep, e_bin, mb, ma).max(0.0);
                hz_stable_chip = hz_a.inner < crit_a || hz_b.inner < crit_b;
                hz_unstable_chip = hz_a.outer > crit_a || hz_b.outer > crit_b;
                any_circumbinary =
                    legend_planets.iter().any(|p| p.host.get() == HOST_BOTH);
                if any_circumbinary {
                    let hz_ab = habitable_zone(la + lb);
                    let p_crit = p_type_critical_radius(sep, e_bin, ma, mb);
                    hz_stable_chip |= hz_ab.outer > p_crit;
                    hz_unstable_chip |= hz_ab.inner < p_crit;
                }
            }
        }
        if hz_stable_chip {
            chips.push(chip("#34d399", view! { {t!(i18n, legend_hz)} }.into_any()).into_any());
        }
        if hz_unstable_chip {
            chips.push(
                chip(
                    "rgba(52,211,153,0.35)",
                    view! { {t!(i18n, legend_hz_unstable)} }.into_any(),
                )
                .into_any(),
            );
        }
        chips.push(chip("#7fb3d5", view! { {t!(i18n, legend_frost)} }.into_any()).into_any());
        chips.push(
            chip("#38bdf8", view! { {t!(i18n, legend_planet_orbit)} }.into_any()).into_any(),
        );

        if binary {
            chips.push(
                chip("#f87171", view! { {t!(i18n, legend_s_crit)} }.into_any()).into_any(),
            );
            if any_circumbinary {
                chips.push(
                    chip("#f87171", view! { {t!(i18n, legend_p_crit)} }.into_any()).into_any(),
                );
            }
        }

        chips
    };

    // ── star summary helpers ────────────────────────────────────────────────
    let lum_a = move || luminosity(star_mass.get());
    let temp_a = move || temperature(star_mass.get());
    let temp_a_class = move || match temp_a() {
        Ok(t) => format!("{t:.3}  ({})", spectral_class(t)),
        Err(e) => e.to_string(),
    };
    let rad_a = move || {
        let l = luminosity(star_mass.get())?;
        let t = temperature(star_mass.get())?;
        Ok::<f64, astro_lib::error::StarErr>(radius(l, t))
    };
    let hz_display = move || match lum_a() {
        Ok(l) => {
            let hz = habitable_zone(l);
            format!("{:.2} – {:.2} AU", hz.inner, hz.outer)
        }
        Err(e) => e.to_string(),
    };

    // ── per-planet + moons summary ──────────────────────────────────────────
    let summary_planets = planets.clone();
    let summary_moons = moons.clone();
    let summary_planet_name = planet_name.clone();
    let planet_summary = move || {
        let binary = binary_mode.get();
        let age_yr = system_age.get() * 1e9;

        summary_planets.iter().map(|p| {
            let pname = summary_planet_name(p.id);
            let sp = stars.params_for(p.host.get());
            let sm = sp.kepler_mass;
            let host = p.host.get();
            let m = p.mass.get();
            let r = p.eff_radius();
            let a = p.semi_major.get();
            let e = p.eccentricity.get();
            let tilt = p.axial_tilt.get();
            let ptype = planet_type(m);
            let rocky = has_solid_surface(ptype);
            let period = orbital_period(a, sm);

            let teq = equilibrium_temperature(sp.temp, sp.radius, a, p.albedo.get());
            let press = surface_pressure_estimate(gravity(m, r), p.atmo_mass.get());
            let ghd = greenhouse_effect(press, p.co2_fraction.get());
            let ts = surface_temperature(teq, ghd);
            let lt = planet_lock_time_years(m, r, sm, a, RIGIDITY_ROCKY);

            let in_hz = is_in_habitable_zone(a, &habitable_zone(sp.luminosity));
            let good_tilt = is_habitable_tilt(tilt);
            let free_rot = !is_tidally_locked(lt, age_yr);

            let type_str = match ptype {
                PlanetType::Rocky       => t_string!(i18n, type_rocky),
                PlanetType::SubNeptune  => t_string!(i18n, type_sub_neptune),
                PlanetType::GasGiant    => t_string!(i18n, type_gas_giant),
                PlanetType::SuperJovian => t_string!(i18n, type_super_jovian),
            };

            // Moons of this planet.
            let pd = density(m, r);
            let h_au = hill_sphere_au(a, m, sm);
            let h_rp = hill_sphere_planet_radii(a, m, sm, r);
            let stab_rp = stable_orbit_limit(h_rp);
            let group: Vec<(usize, MoonSigs)> = summary_moons
                .iter()
                .enumerate()
                .filter(|(_, mo)| mo.parent.get() == p.id)
                .map(|(i, mo)| (i, *mo))
                .collect();

            let moon_views: Vec<AnyView> = group.iter().map(|(idx, mo)| {
                let label = moon_display_name(mo, *idx);
                let mr = mo.radius.get();
                let md = mo.density.get();
                let dist = mo.distance.get();
                let mass_val = moon_mass(mr, md);
                let roche = roche_limit_planet_radii(pd, md);
                let orbit_ok = is_moon_orbit_valid(dist, roche, stab_rp);

                view! {
                    <p class="text-[11px] font-semibold text-label pt-3 pb-1 px-3">
                        {label}
                    </p>
                    <ResultRow label=move || t!(i18n, radius_earth)>
                        {format!("{mr:.3}")}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, mass_earth)>
                        {format!("{mass_val:.4}")}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, surface_gravity)>
                        {format!("{:.3}", moon_gravity(mass_val, mr))}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, distance_rp)>
                        {format!("{dist:.1}")}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, orbital_period_days)>
                        {format!("{:.1}", moon_orbital_period_days(dist, m))}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, angular_size)>
                        {fmt_ang(angular_size_arcmin(mr * R_EARTH_KM, dist * R_EARTH_KM))}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, roche_limit)>
                        {format!("{roche:.2}")}
                    </ResultRow>
                    <BoolRow label=move || t!(i18n, orbit_valid)
                        value=Signal::derive(move || orbit_ok) />
                }.into_any()
            }).collect();

            view! {
                <SectionHeader label={
                    let pname = pname.clone();
                    move || pname.clone()
                } />
                <ResultRow label=move || t!(i18n, planet_type_label)>
                    {type_str}
                </ResultRow>
                {binary.then(|| view! {
                    <ResultRow label=move || t!(i18n, host_star)>
                        {host_tag(host)}
                    </ResultRow>
                })}
                <ResultRow label=move || t!(i18n, mass_earth)>
                    {format!("{m:.3}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, radius_earth)>
                    {format!("{r:.3}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, gravity)>
                    {format!("{:.3}", gravity(m, r))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, density)>
                    {format!("{pd:.3}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, escape_velocity)>
                    {format!("{:.3}", escape_velocity(m, r))}
                </ResultRow>
                {rocky.then(|| view! {
                    <ResultRow label=move || t!(i18n, surface_area)>
                        {format!("{:.3}", surface_area(r))}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, volume)>
                        {format!("{:.3}", volume(r))}
                    </ResultRow>
                })}
                <ResultRow label=move || t!(i18n, semi_major_axis_au)>
                    {format!("{a:.3}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, aphelion_au)>
                    {format!("{:.3}", aphelion(a, e))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, perihelion_au)>
                    {format!("{:.3}", perihelion(a, e))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, orbital_period)>
                    {format!("{:.3} yr  ({:.1} days)", period.years, period.days)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, orbital_velocity)>
                    {format!("{:.3}", orbital_velocity(a, sm))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, tropic_latitude)>
                    {format!("{:.1}", tropic_latitude(tilt))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, polar_circle)>
                    {format!("{:.1}", polar_circle(tilt))}
                </ResultRow>
                <ResultRow label=move || t!(i18n, equilibrium_temp)>
                    {format!("{teq:.0}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, greenhouse)>
                    {format!("+{ghd:.0}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, surface_temp)>
                    {format!("{:.0}  ({:.0} °C)", ts, ts - 273.15)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, surface_pressure)>
                    {format!("{press:.2}")}
                </ResultRow>
                <ResultRow label=move || t!(i18n, tidal_lock_time)>
                    {fmt_years(lt)}
                </ResultRow>
                <BoolRow label=move || t!(i18n, in_habitable_zone)
                    value=Signal::derive(move || in_hz) />
                <BoolRow label=move || t!(i18n, habitable_tilt)
                    value=Signal::derive(move || good_tilt) />
                <BoolRow label=move || t!(i18n, avoids_tidal_lock)
                    value=Signal::derive(move || free_rot) />

                {(!moon_views.is_empty()).then(|| view! {
                    <ResultRow label=move || t!(i18n, hill_sphere)>
                        {format!("{h_au:.4} AU  ({h_rp:.0} Rp)")}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, stable_orbit_limit)>
                        {format!(
                            "{:.4} AU  ({stab_rp:.0} Rp)",
                            stable_orbit_limit(h_au),
                        )}
                    </ResultRow>
                    {moon_views}
                })}
            }.into_any()
        }).collect::<Vec<_>>()
    };

    // ── moon insets (one per planet that has moons) ─────────────────────────
    let inset_planets = planets.clone();
    let inset_moons = moons.clone();
    let inset_planet_name = planet_name.clone();
    let moon_insets = move || {
        let insets: Vec<AnyView> = inset_planets
            .iter()
            .filter_map(|p| {
                let entries: Vec<(f64, f64, f64)> = inset_moons
                    .iter()
                    .filter(|mo| mo.parent.get() == p.id)
                    .map(|mo| (mo.radius.get(), mo.density.get(), mo.distance.get()))
                    .collect();
                if entries.is_empty() {
                    return None;
                }
                let sm = stars.params_for(p.host.get()).kepler_mass;
                let pm = p.mass.get();
                let pr = p.eff_radius();
                let svg = moon_inset_svg(pm, pr, density(pm, pr), p.semi_major.get(), sm, &entries);
                let pname = inset_planet_name(p.id);
                Some(view! {
                    <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                        {pname}
                    </p>
                    <div inner_html=svg />
                }.into_any())
            })
            .collect();

        if insets.is_empty() {
            view! {
                <p class="text-xs text-hint py-2">{t!(i18n, no_moons_note)}</p>
            }.into_any()
        } else {
            insets.into_any()
        }
    };

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="flex flex-col gap-6">

            // ── Map card ────────────────────────────────────────────────
            <div class="bg-card/60 border border-edge rounded-2xl p-6">
                <div class="flex items-center gap-2 mb-4">
                    <span class="text-base text-accent">"⊚"</span>
                    <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                        {t!(i18n, system_map)}
                    </h2>
                    <span class="ml-auto text-[10px] text-hint">
                        {t!(i18n, log_scale_note)}
                    </span>
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center gap-4">
                    <div class="flex-1 min-w-0">
                        {move || match map_svg() {
                            Ok(svg) => view! { <div inner_html=svg /> }.into_any(),
                            Err(e) => view! {
                                <p class="text-xs text-err py-4">
                                    {t!(i18n, map_unavailable)} " " {e}
                                </p>
                            }.into_any(),
                        }}
                    </div>
                    <div class="flex flex-wrap sm:flex-col gap-x-5 gap-y-2 sm:w-44 shrink-0">
                        {move || legend()}
                    </div>
                </div>

                <SectionHeader label=move || t!(i18n, moon_system_map) />
                {move || moon_insets()}
            </div>

            // ── Summary card ────────────────────────────────────────────
            <div class="bg-card/60 border border-edge rounded-2xl p-6">
                <div class="flex items-center gap-2 mb-1">
                    <div class="w-1.5 h-1.5 rounded-full bg-accent" />
                    <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                        {t!(i18n, system_summary)}
                    </h2>
                </div>

                // Star(s)
                {move || if binary_mode.get() {
                    view! { <SectionHeader label=move || t!(i18n, star_a_properties) /> }.into_any()
                } else {
                    view! { <SectionHeader label=move || t!(i18n, star_properties) /> }.into_any()
                }}
                <ResultRow label=move || t!(i18n, star_mass)>
                    {move || format!("{:.3} M☉", star_mass.get())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, luminosity)>
                    {move || fmt_result(lum_a(), 3)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, temperature_class)>
                    {temp_a_class}
                </ResultRow>
                <ResultRow label=move || t!(i18n, radius_solar)>
                    {move || fmt_result(rad_a(), 3)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, lifetime)>
                    {move || fmt_result(lifetime(star_mass.get()), 2)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, peak_wavelength)>
                    {move || fmt_result(temp_a().map(peak_wavelength), 1)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, hz_inner_outer)>
                    {hz_display}
                </ResultRow>
                <ResultRow label=move || t!(i18n, frost_line)>
                    {move || fmt_result(lum_a().map(frost_line), 2)}
                </ResultRow>

                {move || if binary_mode.get() {
                    let ma = star_mass.get();
                    let mb = mass_b.get();
                    Some(view! {
                        <SectionHeader label=move || t!(i18n, star_b_properties) />
                        <ResultRow label=move || t!(i18n, star_mass)>
                            {format!("{mb:.3} M☉")}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, luminosity)>
                            {fmt_result(luminosity(mb), 3)}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, temperature_class)>
                            {match temperature(mb) {
                                Ok(t) => format!("{t:.3}  ({})", spectral_class(t)),
                                Err(e) => e.to_string(),
                            }}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, lifetime)>
                            {fmt_result(lifetime(mb), 2)}
                        </ResultRow>

                        <SectionHeader label=move || t!(i18n, binary_system) />
                        <ResultRow label=move || t!(i18n, binary_period)>
                            {format!("{:.1}", binary_orbital_period(bin_sep.get(), ma, mb))}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, combined_luminosity)>
                            {fmt_result(combined_luminosity(ma, mb), 3)}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, combined_hz)>
                            {match combined_luminosity(ma, mb) {
                                Ok(l) => {
                                    let hz = habitable_zone(l);
                                    format!("{:.2} – {:.2}", hz.inner, hz.outer)
                                }
                                Err(e) => e.to_string(),
                            }}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, s_type_max_orbit)>
                            {format!("{:.2}", s_type_critical_radius(bin_sep.get(), bin_ecc.get(), ma, mb))}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, p_type_min_orbit)>
                            {format!("{:.2}", p_type_critical_radius(bin_sep.get(), bin_ecc.get(), ma, mb))}
                        </ResultRow>
                    })
                } else {
                    None
                }}

                // Planets & their moons
                {move || planet_summary()}
            </div>
        </div>
    }
}
