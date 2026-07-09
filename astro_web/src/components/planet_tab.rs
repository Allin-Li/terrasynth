use astro_lib::atmosphere::{
    atmosphere_retention, equilibrium_temperature, exosphere_temperature_estimate,
    greenhouse_effect, scale_height, surface_pressure_estimate, surface_temperature,
};
use astro_lib::climate::{
    climate_bands, latitude_climate, zone_display_color, ClimateZone, HEAT_TRANSPORT_EARTH,
    SEASONAL_DAMPING_EARTH,
};
use astro_lib::habitability::{is_habitable_tilt, is_in_habitable_zone};
use astro_lib::orbit::{aphelion, orbital_period, orbital_velocity, perihelion, polar_circle, tropic_latitude};
use astro_lib::planet::{
    density, escape_velocity, gravity, has_solid_surface, planet_radius_auto, planet_type,
    surface_area, volume,
};
use astro_lib::star::habitable_zone;
use astro_lib::tidal::{is_tidally_locked, planet_lock_time_years, RIGIDITY_ROCKY};
use crate::i18n::*;
use leptos::prelude::*;

use super::compare::{CompareTable, Snapshot};
use super::planets::{star_inputs, PlanetSigs, StarInputs, HOST_A, HOST_B, HOST_BOTH};
use super::storage::{load_planet_ids, ls_f64, ls_string_dyn, reassign_moon_parents, save_planet_ids};
use super::ui::{filter_numeric, fmt_years, BoolRow, NumberInput, ResultRow, SectionHeader};

#[component]
pub fn PlanetTab() -> impl IntoView {
    let i18n = use_i18n();

    // ── planet list ─────────────────────────────────────────────────────────
    let stored_ids = load_planet_ids();
    let next_id: RwSignal<u32> =
        RwSignal::new(stored_ids.iter().max().map_or(1, |max| max + 1));
    let current: RwSignal<u32> = RwSignal::new(stored_ids[0]);
    let planet_ids: RwSignal<Vec<u32>> = RwSignal::new(stored_ids);

    Effect::new(move |_| save_planet_ids(&planet_ids.get()));

    let add_planet = move |_| {
        let id = next_id.get();
        next_id.set(id + 1);
        planet_ids.update(|v| v.push(id));
        current.set(id);
    };

    // Deleting a planet re-homes its moons onto the first remaining planet.
    let delete_planet = move |id: u32| {
        let ids = planet_ids.get();
        if ids.len() <= 1 {
            return;
        }
        let remaining: Vec<u32> = ids.into_iter().filter(|&p| p != id).collect();
        let fallback = remaining[0];
        reassign_moon_parents(id, fallback);
        if current.get() == id {
            current.set(fallback);
        }
        planet_ids.set(remaining);
    };

    // ── shared (per-system) inputs ──────────────────────────────────────────
    let system_age = ls_f64("system_age_gyr", 4.6);
    let stars = star_inputs();

    // ── save / compare ──────────────────────────────────────────────────────
    let snapshots: RwSignal<Vec<Snapshot>> = RwSignal::new(vec![]);
    let world_name = RwSignal::new(String::from("Planet 1"));
    let save_count = RwSignal::new(1_u32);

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="flex flex-col gap-6">
            // Planet switcher
            <div class="flex flex-wrap items-center gap-1.5">
                {move || {
                    let ids = planet_ids.get();
                    let n = ids.len();
                    ids.into_iter().enumerate().map(|(idx, id)| {
                        let name = ls_string_dyn(format!("planet_{id}_name"), String::new());
                        view! {
                            <button
                                class=move || {
                                    let base = "flex items-center gap-1.5 px-3 py-1.5 \
                                                text-xs font-medium rounded-lg cursor-pointer";
                                    if current.get() == id {
                                        format!("{base} bg-accent/20 text-accent ring-1 ring-accent/30")
                                    } else {
                                        format!("{base} text-hint hover:text-label bg-card \
                                                 ring-1 ring-edge/60")
                                    }
                                }
                                on:click=move |_| current.set(id)
                            >
                                <span>{move || {
                                    let nm = name.get();
                                    if nm.is_empty() {
                                        format!("{} {}", t_string!(i18n, planet), idx + 1)
                                    } else {
                                        nm
                                    }
                                }}</span>
                                {(n > 1).then(|| view! {
                                    <span
                                        class="text-hint hover:text-err leading-none"
                                        on:click=move |ev| {
                                            ev.stop_propagation();
                                            delete_planet(id);
                                        }
                                    >
                                        "✕"
                                    </span>
                                })}
                            </button>
                        }
                    }).collect::<Vec<_>>()
                }}
                <button
                    class="px-3 py-1.5 text-xs font-medium rounded-lg cursor-pointer
                           bg-accent/15 text-accent ring-1 ring-accent/20 hover:bg-accent/25"
                    on:click=add_planet
                >
                    {t!(i18n, add_planet)}
                </button>
            </div>

            {move || {
                let sigs = PlanetSigs::new(current.get());
                view! {
                    <PlanetBody
                        sigs=sigs
                        system_age=system_age
                        stars=stars
                        snapshots=snapshots
                        world_name=world_name
                        save_count=save_count
                    />
                }
            }}

            <CompareTable snapshots=snapshots />
        </div>
    }
}

/// Inputs & results for one planet. Recreated whenever the selected planet
/// changes; all state lives in `sigs` (localStorage-backed) or in the parent.
#[component]
fn PlanetBody(
    sigs: PlanetSigs,
    system_age: RwSignal<f64>,
    stars: StarInputs,
    snapshots: RwSignal<Vec<Snapshot>>,
    world_name: RwSignal<String>,
    save_count: RwSignal<u32>,
) -> impl IntoView {
    let i18n = use_i18n();

    // ── inputs ────────────────────────────────────────────────────────────────
    let planet_name   = sigs.name;
    let host          = sigs.host;
    let planet_mass   = sigs.mass;
    let use_manual_r  = sigs.use_manual_r;
    let manual_radius = sigs.manual_radius;
    let semi_major    = sigs.semi_major;
    let eccentricity  = sigs.eccentricity;
    let axial_tilt    = sigs.axial_tilt;
    let peri_long     = sigs.peri_long;

    // What the planet orbits: the host star (A / B / both for circumbinary)
    // from the Star tab, or the custom override mass.
    let custom_star      = stars.custom;
    let custom_star_mass = stars.custom_mass;
    let star_params = move || stars.params_for(host.get());

    // atmosphere inputs
    let albedo       = sigs.albedo;
    let co2_fraction = sigs.co2_fraction;
    let atmo_mass    = sigs.atmo_mass;

    // Local text signals for inline inputs (prevent prop:value clobbering "1." → "1")
    let manual_radius_text = RwSignal::new(manual_radius.get().to_string());
    Effect::new(move |_| {
        let v = manual_radius.get();
        let cur = manual_radius_text.get_untracked();
        if cur.replace(',', ".").parse::<f64>().ok() != Some(v) {
            manual_radius_text.set(v.to_string());
        }
    });
    let custom_star_text = RwSignal::new(custom_star_mass.get().to_string());
    Effect::new(move |_| {
        let v = custom_star_mass.get();
        let cur = custom_star_text.get_untracked();
        if cur.replace(',', ".").parse::<f64>().ok() != Some(v) {
            custom_star_text.set(v.to_string());
        }
    });

    // ── derived ─────────────────────────────────────────────────────────────
    let ptype = Signal::derive(move || planet_type(planet_mass.get()));

    let eff_radius = Signal::derive(move || {
        if use_manual_r.get() {
            manual_radius.get()
        } else {
            planet_radius_auto(planet_mass.get())
        }
    });

    let is_rocky = Signal::derive(move || has_solid_surface(ptype.get()));

    // ── planet properties ─────────────────────────────────────────────────────
    let grav    = move || gravity(planet_mass.get(), eff_radius.get());
    let dens    = move || density(planet_mass.get(), eff_radius.get());
    let v_esc   = move || escape_velocity(planet_mass.get(), eff_radius.get());
    let s_area  = move || surface_area(eff_radius.get());
    let vol     = move || volume(eff_radius.get());

    // ── orbit ───────────────────────────────────────────────────────────────
    let aph     = move || aphelion(semi_major.get(), eccentricity.get());
    let peri    = move || perihelion(semi_major.get(), eccentricity.get());
    let period  = move || orbital_period(semi_major.get(), star_params().kepler_mass);
    let v_orb   = move || orbital_velocity(semi_major.get(), star_params().kepler_mass);

    let period_display = move || {
        let p = period();
        format!("{:.3} yr  ({:.1} days)", p.years, p.days)
    };

    // ── axial tilt ──────────────────────────────────────────────────────────
    let tropic  = move || tropic_latitude(axial_tilt.get());
    let polar   = move || polar_circle(axial_tilt.get());

    // ── tidal locking ───────────────────────────────────────────────────────
    let lock_time = move || planet_lock_time_years(
        planet_mass.get(), eff_radius.get(), star_params().kepler_mass, semi_major.get(),
        RIGIDITY_ROCKY,
    );
    let free_rotation = Signal::derive(move || {
        !is_tidally_locked(lock_time(), system_age.get() * 1e9)
    });

    // ── atmosphere ──────────────────────────────────────────────────────────
    let star_temp_rel = move || star_params().temp;
    let star_rad_rel  = move || star_params().radius;

    let t_eq = move || equilibrium_temperature(
        star_temp_rel(), star_rad_rel(), semi_major.get(), albedo.get(),
    );
    let t_exo = move || exosphere_temperature_estimate(t_eq());
    let s_press = move || surface_pressure_estimate(grav(), atmo_mass.get());
    let gh_delta = move || greenhouse_effect(s_press(), co2_fraction.get());
    let t_surf = move || surface_temperature(t_eq(), gh_delta());
    let sh = move || scale_height(grav());

    // ── climate by latitude ─────────────────────────────────────────────────
    // Bands from −90° (south) to +90° (north); with an eccentric orbit the
    // hemispheres differ, so the table has a hemisphere switch. The disc
    // always shows both hemispheres.
    let show_south = RwSignal::new(false);
    let climate_rows = move || {
        let rows = climate_bands(axial_tilt.get(), t_surf(), eccentricity.get(), peri_long.get());
        if show_south.get() {
            rows[..7].iter().rev().copied().collect::<Vec<_>>()
        } else {
            rows[6..].to_vec()
        }
    };

    // Planet disc colored by climate zone bands (computed at band midpoints).
    // Built as an SVG string: only numbers and palette hex colors go in.
    let climate_svg = move || {
        let tilt = axial_tilt.get();
        let tm = t_surf();
        let (ecc, peri) = (eccentricity.get(), peri_long.get());
        let mut rects = String::new();
        for i in 0..12u32 {
            let top = 90.0 - i as f64 * 15.0;
            let c = latitude_climate(
                top - 7.5, tilt, tm, ecc, peri, HEAT_TRANSPORT_EARTH, SEASONAL_DAMPING_EARTH,
            );
            let color = zone_display_color(c.zone);
            let y1 = 60.0 - top.to_radians().sin() * 56.0;
            let y2 = 60.0 - (top - 15.0).to_radians().sin() * 56.0;
            let h = y2 - y1;
            rects.push_str(&format!(
                "<rect x='4' y='{y1:.2}' width='112' height='{h:.2}' fill='{color}'/>"
            ));
        }
        format!(
            "<svg viewBox='0 0 120 120' width='120' height='120' role='img'>\
             <defs><clipPath id='climate-disc'><circle cx='60' cy='60' r='56'/></clipPath></defs>\
             <g clip-path='url(#climate-disc)'>{rects}</g>\
             <circle cx='60' cy='60' r='56' fill='none' stroke='rgba(148,163,184,0.35)' stroke-width='1.5'/>\
             <line x1='4' y1='60' x2='116' y2='60' stroke='rgba(15,23,42,0.6)' stroke-width='1' stroke-dasharray='3 3'/>\
             </svg>"
        )
    };

    // ── habitability ────────────────────────────────────────────────────────
    let in_hz = Signal::derive(move || {
        is_in_habitable_zone(semi_major.get(), &habitable_zone(star_params().luminosity))
    });
    let good_tilt = Signal::derive(move || is_habitable_tilt(axial_tilt.get()));

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-[300px_1fr] gap-6 items-start">

            // ── Inputs card ─────────────────────────────────────────────
            <div class="bg-card border border-edge rounded-2xl p-6 pb-8 flex flex-col gap-5">
                <div class="flex items-center gap-2">
                    <span class="text-base text-accent">"◉"</span>
                    <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                        {t!(i18n, inputs)}
                    </h2>
                    // planet type badge
                    <span class="ml-auto text-[10px] font-semibold px-2 py-0.5 rounded-full
                                 bg-accent/15 text-accent ring-1 ring-accent/20">
                        {move || match ptype.get() {
                            astro_lib::planet::PlanetType::Rocky       => t_string!(i18n, type_rocky),
                            astro_lib::planet::PlanetType::SubNeptune  => t_string!(i18n, type_sub_neptune),
                            astro_lib::planet::PlanetType::GasGiant    => t_string!(i18n, type_gas_giant),
                            astro_lib::planet::PlanetType::SuperJovian => t_string!(i18n, type_super_jovian),
                        }}
                    </span>
                </div>

                <p class="text-[10px] font-semibold text-hint uppercase tracking-widest">
                    {t!(i18n, planet)}
                </p>
                <input
                    type="text"
                    placeholder=move || t_string!(i18n, planet_name_placeholder)
                    class="bg-inset border border-edge rounded-lg
                           px-3 py-2 text-heading text-sm outline-none
                           focus:border-accent focus:ring-1 focus:ring-accent/40
                           hover:border-divider w-full"
                    prop:value=move || planet_name.get()
                    on:input=move |ev| planet_name.set(event_target_value(&ev))
                />
                <NumberInput label=move || t!(i18n, mass) value=planet_mass unit="M⊕" step="0.01"
                    hint=move || t!(i18n, hint_mass) />

                // Radius toggle: auto or manual
                <div class="flex flex-col gap-1.5">
                    <div class="flex items-baseline justify-between">
                        <span class="text-xs font-medium text-label">{t!(i18n, radius_earth)}</span>
                        <button
                            class=move || {
                                if use_manual_r.get() {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     cursor-pointer \
                                     bg-accent/15 text-accent ring-1 ring-accent/20"
                                } else {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     cursor-pointer \
                                     bg-edge/40 text-hint ring-1 ring-edge \
                                     hover:text-label"
                                }
                            }
                            on:click=move |_| use_manual_r.update(|v| *v = !*v)
                        >
                            {move || if use_manual_r.get() { t_string!(i18n, manual) } else { t_string!(i18n, auto) }}
                        </button>
                    </div>
                    {move || if use_manual_r.get() {
                        view! {
                            <input
                                type="text" inputmode="decimal"
                                prop:value=move || manual_radius_text.get()
                                class="bg-inset border border-edge rounded-lg
                                       px-3 py-2 text-heading text-sm font-mono
                                       outline-none
                                       focus:border-accent focus:ring-1 focus:ring-accent/40
                                       hover:border-divider w-full"
                                on:input=move |ev| {
                                    let filtered = filter_numeric(&event_target_value(&ev));
                                    manual_radius_text.set(filtered.clone());
                                    let sanitized = filtered.replace(',', ".");
                                    if let Ok(v) = sanitized.parse::<f64>() {
                                        manual_radius.set(v);
                                    }
                                }
                            />
                        }.into_any()
                    } else {
                        view! {
                            <div class="bg-inset border border-edge rounded-lg
                                        px-3 py-2 text-accent text-sm font-mono">
                                {move || format!("{:.3}", planet_radius_auto(planet_mass.get()))}
                            </div>
                        }.into_any()
                    }}
                </div>

                <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                    {t!(i18n, orbit)}
                </p>
                <NumberInput label=move || t!(i18n, semi_major_axis) value=semi_major unit="AU" step="0.01"
                    hint=move || t!(i18n, hint_semi_major) />
                <NumberInput label=move || t!(i18n, eccentricity) value=eccentricity step="0.001"
                    hint=move || t!(i18n, hint_eccentricity) />
                <NumberInput label=move || t!(i18n, axial_tilt) value=axial_tilt unit="°" step="0.1"
                    hint=move || t!(i18n, hint_axial_tilt) />
                <NumberInput label=move || t!(i18n, perihelion_long) value=peri_long unit="°" step="1"
                    hint=move || t!(i18n, hint_perihelion_long) />
                <NumberInput label=move || t!(i18n, system_age) value=system_age unit="Gyr" step="0.1"
                    hint=move || t!(i18n, hint_system_age) />

                // Star mass: linked from Star tab or custom
                <div class="flex flex-col gap-1.5">
                    <div class="flex items-baseline justify-between">
                        <span class="text-xs font-medium text-label flex items-center gap-1">
                            {t!(i18n, star_mass)}
                            <super::ui::InfoHint text=move || t!(i18n, hint_star_mass) />
                        </span>
                        <button
                            class=move || {
                                if custom_star.get() {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     cursor-pointer \
                                     bg-accent/15 text-accent ring-1 ring-accent/20"
                                } else {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     cursor-pointer \
                                     bg-edge/40 text-hint ring-1 ring-edge \
                                     hover:text-label"
                                }
                            }
                            on:click=move |_| custom_star.update(|v| *v = !*v)
                        >
                            {move || if custom_star.get() { t_string!(i18n, custom) } else { t_string!(i18n, from_star) }}
                        </button>
                    </div>
                    {move || if custom_star.get() {
                        view! {
                            <input
                                type="text" inputmode="decimal"
                                prop:value=move || custom_star_text.get()
                                class="bg-inset border border-edge rounded-lg
                                       px-3 py-2 text-heading text-sm font-mono
                                       outline-none
                                       focus:border-accent focus:ring-1 focus:ring-accent/40
                                       hover:border-divider w-full"
                                on:input=move |ev| {
                                    let filtered = filter_numeric(&event_target_value(&ev));
                                    custom_star_text.set(filtered.clone());
                                    let sanitized = filtered.replace(',', ".");
                                    if let Ok(v) = sanitized.parse::<f64>() {
                                        custom_star_mass.set(v);
                                    }
                                }
                            />
                        }.into_any()
                    } else {
                        view! {
                            <div class="bg-inset border border-edge rounded-lg
                                        px-3 py-2 text-accent text-sm font-mono">
                                {move || format!("{:.3}", star_params().kepler_mass)}
                                <span class="text-[10px] text-hint ml-1">"M☉"</span>
                            </div>
                        }.into_any()
                    }}
                </div>

                // Host star: which star of the binary this planet orbits
                {move || stars.binary.get().then(|| view! {
                    <div class="flex items-center gap-2 flex-wrap">
                        <span class="text-xs font-medium text-label">
                            {t!(i18n, host_star)}
                        </span>
                        {[
                            (HOST_A, "A"),
                            (HOST_B, "B"),
                            (HOST_BOTH, "A+B"),
                        ].into_iter().map(|(h, lbl)| view! {
                            <button
                                class=move || {
                                    if host.get() == h {
                                        "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                         cursor-pointer \
                                         bg-accent/15 text-accent ring-1 ring-accent/20"
                                    } else {
                                        "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                         cursor-pointer \
                                         bg-edge/40 text-hint ring-1 ring-edge hover:text-label"
                                    }
                                }
                                on:click=move |_| host.set(h)
                            >
                                {lbl}
                            </button>
                        }).collect::<Vec<_>>()}
                    </div>
                })}

                <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                    {t!(i18n, atmosphere)}
                </p>
                <NumberInput label=move || t!(i18n, albedo) value=albedo step="0.01"
                    hint=move || t!(i18n, hint_albedo) />
                <NumberInput label=move || t!(i18n, co2_fraction) value=co2_fraction step="0.0001"
                    hint=move || t!(i18n, hint_co2) />
                <NumberInput label=move || t!(i18n, atmo_mass_factor) value=atmo_mass step="0.1"
                    hint=move || t!(i18n, hint_atmo_mass) />

                // Save row
                <div class="flex items-center gap-2 pt-4 border-t border-edge min-w-0">
                    <input
                        type="text"
                        placeholder=move || t_string!(i18n, world_name_placeholder)
                        class="flex-1 min-w-0 bg-inset border border-edge rounded-lg
                               px-3 py-2 text-heading text-sm outline-none
                               focus:border-accent focus:ring-1 focus:ring-accent/40
                               hover:border-divider"
                        prop:value=move || world_name.get()
                        on:input=move |ev| world_name.set(event_target_value(&ev))
                    />
                    <button
                        class="px-4 py-2 text-xs font-semibold rounded-lg cursor-pointer
                               bg-accent/20 text-accent ring-1 ring-accent/30
                               hover:bg-accent/30 whitespace-nowrap"
                        on:click=move |_| {
                            let m  = planet_mass.get();
                            let r  = eff_radius.get();
                            let a  = semi_major.get();
                            let e  = eccentricity.get();
                            let sp = star_params();
                            let sm = sp.kepler_mass;
                            let t  = axial_tilt.get();
                            let p  = orbital_period(a, sm);
                            let hz_bool = is_in_habitable_zone(a, &habitable_zone(sp.luminosity));

                            let ek = Locale::en.get_keys_const();
                            let rk = Locale::ru.get_keys_const();
                            let ck = i18n.get_locale_untracked().get_keys_const();
                            macro_rules! lbl {
                                ($key:ident) => { [ek.$key().inner().to_owned(), rk.$key().inner().to_owned()] }
                            }

                            let type_str = match planet_type(m) {
                                astro_lib::planet::PlanetType::Rocky       => ck.type_rocky().inner().to_owned(),
                                astro_lib::planet::PlanetType::SubNeptune  => ck.type_sub_neptune().inner().to_owned(),
                                astro_lib::planet::PlanetType::GasGiant    => ck.type_gas_giant().inner().to_owned(),
                                astro_lib::planet::PlanetType::SuperJovian => ck.type_super_jovian().inner().to_owned(),
                            };

                            let mut rows: Vec<([String; 2], String)> = vec![
                                (lbl!(planet_type_label), type_str),
                                (lbl!(radius_earth),      format!("{:.3}", r)),
                                (lbl!(gravity),           format!("{:.3}", gravity(m, r))),
                                (lbl!(density),           format!("{:.3}", density(m, r))),
                                (lbl!(escape_velocity),   format!("{:.3}", escape_velocity(m, r))),
                                (lbl!(surface_area),      format!("{:.3}", surface_area(r))),
                                (lbl!(volume),            format!("{:.3}", volume(r))),
                                (lbl!(semi_major_axis_au), format!("{:.3}", a)),
                                (lbl!(aphelion_au),       format!("{:.3}", aphelion(a, e))),
                                (lbl!(perihelion_au),     format!("{:.3}", perihelion(a, e))),
                                (lbl!(orbital_period),    format!("{:.3} yr  ({:.1} days)", p.years, p.days)),
                                (lbl!(orbital_velocity),  format!("{:.3}", orbital_velocity(a, sm))),
                                (lbl!(tropic_latitude),   format!("{:.1}", tropic_latitude(t))),
                                (lbl!(polar_circle),      format!("{:.1}", polar_circle(t))),
                                (lbl!(in_habitable_zone), if hz_bool { "✓" } else { "✗" }.to_string()),
                                (lbl!(habitable_tilt),    if is_habitable_tilt(t) { "✓" } else { "✗" }.to_string()),
                            ];

                            // tidal locking snapshot rows
                            let lt = planet_lock_time_years(m, r, sm, a, RIGIDITY_ROCKY);
                            let free = !is_tidally_locked(lt, system_age.get() * 1e9);
                            rows.push((lbl!(tidal_lock_time),  fmt_years(lt)));
                            rows.push((lbl!(avoids_tidal_lock), if free { "✓" } else { "✗" }.to_string()));

                            // atmosphere snapshot rows
                            let g = gravity(m, r);
                            let ve = escape_velocity(m, r);
                            let al = albedo.get();
                            let teq = equilibrium_temperature(sp.temp, sp.radius, a, al);
                            let texo = exosphere_temperature_estimate(teq);
                            let sp = surface_pressure_estimate(g, atmo_mass.get());
                            let ghd = greenhouse_effect(sp, co2_fraction.get());
                            let ts = surface_temperature(teq, ghd);
                            rows.push((lbl!(equilibrium_temp), format!("{:.0}", teq)));
                            rows.push((lbl!(surface_temp),     format!("{:.0}", ts)));
                            rows.push((lbl!(surface_pressure), format!("{:.2}", sp)));

                            let ret = atmosphere_retention(ve, texo);
                            let gases = [
                                ("H₂", ret.hydrogen), ("He", ret.helium),
                                ("N₂", ret.nitrogen), ("O₂", ret.oxygen),
                                ("H₂O", ret.water_vapor), ("CO₂", ret.co2),
                            ];
                            let retained: Vec<&str> = gases.iter()
                                .filter(|(_, ok)| *ok).map(|(n, _)| *n).collect();
                            rows.push((lbl!(gas_retention), retained.join(", ")));

                            let default_name = {
                                let nm = planet_name.get();
                                if nm.is_empty() { world_name.get() } else { nm }
                            };
                            let snap = Snapshot { name: default_name, rows };
                            snapshots.update(|v| v.push(snap));
                            let n = save_count.get() + 1;
                            save_count.set(n);
                            world_name.set(format!("Planet {n}"));
                        }
                    >
                        {t!(i18n, save)}
                    </button>
                </div>
            </div>

            // ── Results card ────────────────────────────────────────────
            <div class="bg-card/60 border border-edge rounded-2xl p-6">
                <div class="flex items-center gap-2 mb-1">
                    <div class="w-1.5 h-1.5 rounded-full bg-accent" />
                    <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                        {t!(i18n, results)}
                    </h2>
                    <span class="ml-auto text-[10px] font-medium px-2 py-0.5 rounded-full
                                 bg-edge/40 text-hint ring-1 ring-edge">
                        {move || match ptype.get() {
                            astro_lib::planet::PlanetType::Rocky       => t_string!(i18n, type_rocky),
                            astro_lib::planet::PlanetType::SubNeptune  => t_string!(i18n, type_sub_neptune),
                            astro_lib::planet::PlanetType::GasGiant    => t_string!(i18n, type_gas_giant),
                            astro_lib::planet::PlanetType::SuperJovian => t_string!(i18n, type_super_jovian),
                        }}
                    </span>
                </div>

                <SectionHeader label=move || t!(i18n, planet_properties) />
                <ResultRow label=move || t!(i18n, radius_earth)>
                    {move || format!("{:.3}", eff_radius.get())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, gravity)
                    hint=move || t!(i18n, hint_gravity)>
                    {move || format!("{:.3}", grav())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, density)
                    hint=move || t!(i18n, hint_density)>
                    {move || format!("{:.3}", dens())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, escape_velocity)
                    hint=move || t!(i18n, hint_escape_velocity)>
                    {move || format!("{:.3}", v_esc())}
                </ResultRow>
                {move || if is_rocky.get() {
                    Some(view! {
                        <ResultRow label=move || t!(i18n, surface_area)
                            hint=move || t!(i18n, hint_surface_area)>
                            {move || format!("{:.3}", s_area())}
                        </ResultRow>
                        <ResultRow label=move || t!(i18n, volume)
                            hint=move || t!(i18n, hint_volume)>
                            {move || format!("{:.3}", vol())}
                        </ResultRow>
                    })
                } else {
                    None
                }}

                <SectionHeader label=move || t!(i18n, orbit) />
                <ResultRow label=move || t!(i18n, semi_major_axis_au)>
                    {move || format!("{:.3}", semi_major.get())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, aphelion_au)
                    hint=move || t!(i18n, hint_aphelion)>
                    {move || format!("{:.3}", aph())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, perihelion_au)
                    hint=move || t!(i18n, hint_perihelion)>
                    {move || format!("{:.3}", peri())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, orbital_period)
                    hint=move || t!(i18n, hint_orbital_period)>
                    {period_display}
                </ResultRow>
                <ResultRow label=move || t!(i18n, orbital_velocity)
                    hint=move || t!(i18n, hint_orbital_velocity)>
                    {move || format!("{:.3}", v_orb())}
                </ResultRow>

                <SectionHeader label=move || t!(i18n, axial_tilt_section) />
                <ResultRow label=move || t!(i18n, tropic_latitude)
                    hint=move || t!(i18n, hint_tropic)>
                    {move || format!("{:.1}", tropic())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, polar_circle)
                    hint=move || t!(i18n, hint_polar_circle)>
                    {move || format!("{:.1}", polar())}
                </ResultRow>

                <SectionHeader label=move || t!(i18n, tidal_locking) />
                <ResultRow label=move || t!(i18n, tidal_lock_time)
                    hint=move || t!(i18n, hint_tidal_lock_time)>
                    {move || fmt_years(lock_time())}
                </ResultRow>

                <SectionHeader label=move || t!(i18n, atmosphere) />
                <ResultRow label=move || t!(i18n, equilibrium_temp)
                    hint=move || t!(i18n, hint_eq_temp)>
                    {move || format!("{:.0}", t_eq())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, greenhouse)
                    hint=move || t!(i18n, hint_greenhouse)>
                    {move || format!("+{:.0}", gh_delta())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, surface_temp)
                    hint=move || t!(i18n, hint_surface_temp)>
                    {move || format!("{:.0}  ({:.0} °C)", t_surf(), t_surf() - 273.15)}
                </ResultRow>
                <ResultRow label=move || t!(i18n, surface_pressure)
                    hint=move || t!(i18n, hint_surface_pressure)>
                    {move || format!("{:.2}", s_press())}
                </ResultRow>
                <ResultRow label=move || t!(i18n, scale_height)
                    hint=move || t!(i18n, hint_scale_height)>
                    {move || format!("{:.0}", sh())}
                </ResultRow>

                // gas retention table
                <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-3 pb-1">
                    {t!(i18n, gas_retention)}
                </p>
                {move || {
                    let ve = v_esc();
                    let texo = t_exo();
                    let ret = atmosphere_retention(ve, texo);
                    let gases: Vec<(&str, bool)> = vec![
                        ("H₂",  ret.hydrogen),
                        ("He",   ret.helium),
                        ("CH₄",  ret.methane),
                        ("NH₃",  ret.ammonia),
                        ("H₂O",  ret.water_vapor),
                        ("N₂",   ret.nitrogen),
                        ("O₂",   ret.oxygen),
                        ("CO₂",  ret.co2),
                    ];
                    view! {
                        <div class="flex flex-wrap gap-1.5">
                            {gases.into_iter().map(|(name, ok)| {
                                let cls = if ok {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     bg-ok/15 text-ok ring-1 ring-ok/20"
                                } else {
                                    "text-[10px] font-medium px-2 py-0.5 rounded-full \
                                     bg-err/15 text-err ring-1 ring-err/20"
                                };
                                view! { <span class=cls>{name}</span> }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }}

                // climate by latitude (solid-surface planets only)
                {move || if is_rocky.get() {
                    Some(view! {
                        <SectionHeader label=move || t!(i18n, climate_section) />
                        {move || if free_rotation.get() {
                            view! {
                                <div class="flex flex-col sm:flex-row gap-5 items-center pt-1 pb-2">
                                    <div class="shrink-0" inner_html=climate_svg() />
                                    <div class="flex-1 w-full overflow-x-auto">
                                        <div class="flex items-center gap-1.5 pb-1.5">
                                            <button
                                                class=move || {
                                                    if !show_south.get() {
                                                        "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                                         bg-accent/15 text-accent ring-1 ring-accent/20"
                                                    } else {
                                                        "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                                         bg-edge/40 text-hint ring-1 ring-edge hover:text-label"
                                                    }
                                                }
                                                on:click=move |_| show_south.set(false)
                                            >
                                                {t!(i18n, hemisphere_north)}
                                            </button>
                                            <button
                                                class=move || {
                                                    if show_south.get() {
                                                        "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                                         bg-accent/15 text-accent ring-1 ring-accent/20"
                                                    } else {
                                                        "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                                         bg-edge/40 text-hint ring-1 ring-edge hover:text-label"
                                                    }
                                                }
                                                on:click=move |_| show_south.set(true)
                                            >
                                                {t!(i18n, hemisphere_south)}
                                            </button>
                                        </div>
                                        <table class="w-full text-xs">
                                            <thead>
                                                <tr class="text-hint text-[10px] uppercase tracking-wider">
                                                    <th class="text-left font-semibold py-1.5 pr-2 flex items-center gap-1">
                                                        {t!(i18n, latitude_col)}
                                                        <super::ui::InfoHint text=move || t!(i18n, hint_climate) />
                                                    </th>
                                                    <th class="text-left font-semibold py-1.5 px-2">{t!(i18n, zone_col)}</th>
                                                    <th class="text-right font-semibold py-1.5 px-2">{t!(i18n, annual_col)}</th>
                                                    <th class="text-right font-semibold py-1.5 px-2">{t!(i18n, summer_col)}</th>
                                                    <th class="text-right font-semibold py-1.5 pl-2">{t!(i18n, winter_col)}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {move || climate_rows().into_iter().map(|c| {
                                                    let color = zone_display_color(c.zone);
                                                    let lat_label = format!("{:.0}°", c.latitude_deg.abs());
                                                    // round first so −0.4 °C prints as 0, not "-0"
                                                    let fmt_c = |k: f64| {
                                                        let deg = (k - 273.15).round() + 0.0;
                                                        format!("{deg:.0}")
                                                    };
                                                    let zone_name = match c.zone {
                                                        ClimateZone::IceCap    => t_string!(i18n, zone_ice_cap),
                                                        ClimateZone::Tundra    => t_string!(i18n, zone_tundra),
                                                        ClimateZone::Boreal    => t_string!(i18n, zone_boreal),
                                                        ClimateZone::Temperate => t_string!(i18n, zone_temperate),
                                                        ClimateZone::Tropical  => t_string!(i18n, zone_tropical),
                                                        ClimateZone::Scorched  => t_string!(i18n, zone_scorched),
                                                    };
                                                    view! {
                                                        <tr class="border-t border-divider/30 hover:bg-edge/20">
                                                            <td class="text-label font-mono py-1.5 pr-2 whitespace-nowrap">
                                                                {lat_label}
                                                            </td>
                                                            <td class="py-1.5 px-2">
                                                                <span class="inline-flex items-center gap-1.5 whitespace-nowrap">
                                                                    <span
                                                                        class="w-2 h-2 rounded-full inline-block shrink-0"
                                                                        style=format!("background:{color}")
                                                                    />
                                                                    <span class="text-label">{zone_name}</span>
                                                                </span>
                                                            </td>
                                                            <td class="text-heading font-mono tabular-nums text-right py-1.5 px-2">
                                                                {fmt_c(c.annual_k)}
                                                            </td>
                                                            <td class="text-heading font-mono tabular-nums text-right py-1.5 px-2">
                                                                {fmt_c(c.summer_k)}
                                                            </td>
                                                            <td class="text-heading font-mono tabular-nums text-right py-1.5 pl-2">
                                                                {fmt_c(c.winter_k)}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                        <p class="text-[10px] text-hint pt-1">"°C"</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <p class="text-xs text-hint leading-relaxed py-2 px-3">
                                    {t!(i18n, climate_locked_note)}
                                </p>
                            }.into_any()
                        }}
                    })
                } else {
                    None
                }}

                <SectionHeader label=move || t!(i18n, habitability) />
                <BoolRow label=move || t!(i18n, in_habitable_zone) value=in_hz
                    hint=move || t!(i18n, hint_in_hz) />
                <BoolRow label=move || t!(i18n, habitable_tilt) value=good_tilt
                    hint=move || t!(i18n, hint_habitable_tilt) />
                <BoolRow label=move || t!(i18n, avoids_tidal_lock) value=free_rotation
                    hint=move || t!(i18n, hint_avoids_tidal_lock) />
            </div>
        </div>
    }
}
