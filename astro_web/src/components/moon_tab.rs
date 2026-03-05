use astro_lib::*;
use crate::i18n::*;
use leptos::prelude::*;

use super::compare::{CompareTable, Snapshot};
use super::storage::{ls_f64, ls_f64_dyn};
use super::ui::{NumberInput, ResultRow, SectionHeader};

const R_EARTH_KM: f64 = 6_371.0;

/// Per-moon reactive state.
#[derive(Clone)]
struct MoonEntry {
    id: u32,
    radius: RwSignal<f64>,
    density: RwSignal<f64>,
    distance: RwSignal<f64>,
}

impl MoonEntry {
    fn new(id: u32) -> Self {
        Self {
            id,
            radius: ls_f64_dyn(format!("moon_{id}_radius"), 0.273),
            density: ls_f64_dyn(format!("moon_{id}_density"), 0.606),
            distance: ls_f64_dyn(format!("moon_{id}_dist"), 60.27),
        }
    }
}

fn format_ang(total_min: f64) -> String {
    let deg = (total_min / 60.0) as u32;
    let min = total_min % 60.0;
    if deg > 0 {
        format!("{deg}\u{b0}  {min:.0}'")
    } else {
        format!("{min:.1}'")
    }
}

#[component]
pub fn MoonTab() -> impl IntoView {
    let i18n = use_i18n();

    // ── shared planet / star inputs ─────────────────────────────────────────
    let planet_mass    = ls_f64("moon_planet_mass",    1.0);
    let planet_radius  = ls_f64("moon_planet_radius",  1.0);
    let planet_density = ls_f64("moon_planet_density", 1.0);
    let planet_orb_a   = ls_f64("moon_planet_orb_a",   1.0);
    let star_mass      = ls_f64("moon_star_mass",       1.0);

    // ── dynamic moon list ───────────────────────────────────────────────────
    let next_id: RwSignal<u32> = RwSignal::new(1);
    let moons: RwSignal<Vec<MoonEntry>> = RwSignal::new(vec![MoonEntry::new(0)]);

    let add_moon = move |_| {
        let id = next_id.get();
        next_id.set(id + 1);
        moons.update(|v| v.push(MoonEntry::new(id)));
    };

    // ── stability (shared) ──────────────────────────────────────────────────
    let h_au  = move || hill_sphere_au(planet_orb_a.get(), planet_mass.get(), star_mass.get());
    let h_rp  = move || hill_sphere_planet_radii(
        planet_orb_a.get(), planet_mass.get(), star_mass.get(), planet_radius.get(),
    );
    let stab_au = move || stable_orbit_limit(h_au());
    let stab_rp = move || stable_orbit_limit(h_rp());

    // ── save / compare ──────────────────────────────────────────────────────
    let snapshots: RwSignal<Vec<Snapshot>> = RwSignal::new(vec![]);
    let world_name = RwSignal::new(String::from("System 1"));
    let save_count = RwSignal::new(1_u32);

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="flex flex-col gap-8">
            <div class="grid grid-cols-1 lg:grid-cols-[300px_1fr] gap-6 items-start">

                // ── Inputs card ─────────────────────────────────────────────
                <div class="bg-card border border-edge rounded-2xl p-6 pb-8 flex flex-col gap-5">
                    <div class="flex items-center gap-2">
                        <span class="text-base text-accent">"☽"</span>
                        <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                            {t!(i18n, inputs)}
                        </h2>
                    </div>

                    <p class="text-[10px] font-semibold text-hint uppercase tracking-widest">
                        {t!(i18n, parent_planet)}
                    </p>
                    <NumberInput label=move || t!(i18n, mass)    value=planet_mass    unit="M⊕" step="0.01" />
                    <NumberInput label=move || t!(i18n, radius)  value=planet_radius  unit="R⊕" step="0.01" />
                    <NumberInput label=move || t!(i18n, density_label) value=planet_density unit="ρ⊕" step="0.01" />

                    <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                        {t!(i18n, star_orbit)}
                    </p>
                    <NumberInput label=move || t!(i18n, planet_semi_major) value=planet_orb_a unit="AU" step="0.01" />
                    <NumberInput label=move || t!(i18n, star_mass) value=star_mass unit="M☉" step="0.01" />

                    // ── Moon entries ────────────────────────────────────────
                    <div class="flex items-center justify-between pt-2">
                        <p class="text-[10px] font-semibold text-hint uppercase tracking-widest">
                            {t!(i18n, moons)}
                        </p>
                        <button
                            class="text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer
                                   bg-accent/15 text-accent ring-1 ring-accent/20 hover:bg-accent/25"
                            on:click=add_moon
                        >
                            {t!(i18n, add_moon)}
                        </button>
                    </div>

                    {move || {
                        let moon_list = moons.get();
                        moon_list.into_iter().enumerate().map(|(idx, entry)| {
                            let entry_id = entry.id;
                            let mr = entry.radius;
                            let md = entry.density;
                            let mdist = entry.distance;
                            view! {
                                <div class="bg-inset border border-edge rounded-xl p-4 flex flex-col gap-3">
                                    <div class="flex items-center justify-between">
                                        <span class="text-xs font-semibold text-label">
                                            {t!(i18n, moon_label, idx = move || idx + 1)}
                                        </span>
                                        <button
                                            class="text-[10px] text-err hover:text-err/80 cursor-pointer px-1"
                                            on:click=move |_| {
                                                moons.update(|v| v.retain(|m| m.id != entry_id));
                                            }
                                        >
                                            "✕"
                                        </button>
                                    </div>
                                    <label class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint">{t!(i18n, radius_earth)}</span>
                                        <input type="number" step="0.001"
                                            prop:value=move || mr.get().to_string()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                if let Ok(v) = event_target_value(&ev).parse::<f64>() { mr.set(v); }
                                            }
                                        />
                                    </label>
                                    <label class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint">{t!(i18n, density)}</span>
                                        <input type="number" step="0.01"
                                            prop:value=move || md.get().to_string()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                if let Ok(v) = event_target_value(&ev).parse::<f64>() { md.set(v); }
                                            }
                                        />
                                    </label>
                                    <label class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint">{t!(i18n, distance_rp)}</span>
                                        <input type="number" step="0.1"
                                            prop:value=move || mdist.get().to_string()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                if let Ok(v) = event_target_value(&ev).parse::<f64>() { mdist.set(v); }
                                            }
                                        />
                                    </label>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}

                    // Save row
                    <div class="flex items-center gap-2 pt-4 border-t border-edge min-w-0">
                        <input
                            type="text"
                            placeholder=move || t_string!(i18n, system_name_placeholder)
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
                                let pm  = planet_mass.get();
                                let pr  = planet_radius.get();
                                let pd  = planet_density.get();
                                let oa  = planet_orb_a.get();
                                let sm  = star_mass.get();
                                let h_au_val = hill_sphere_au(oa, pm, sm);
                                let h_rp_val = hill_sphere_planet_radii(oa, pm, sm, pr);

                                let mut rows: Vec<(&str, String)> = vec![
                                    ("Hill sphere", format!("{:.4} AU  ({:.0} Rp)", h_au_val, h_rp_val)),
                                    ("Stable orbit limit", format!("{:.4} AU  ({:.0} Rp)",
                                        stable_orbit_limit(h_au_val), stable_orbit_limit(h_rp_val))),
                                ];

                                let moon_list = moons.get();
                                for (i, m) in moon_list.iter().enumerate() {
                                    let mr = m.radius.get();
                                    let md = m.density.get();
                                    let dst = m.distance.get();
                                    let mass_val = moon_mass(mr, md);
                                    let ang = angular_size_arcmin(mr * R_EARTH_KM, dst * R_EARTH_KM);
                                    let label_prefix = if moon_list.len() > 1 {
                                        format!("M{} ", i + 1)
                                    } else {
                                        String::new()
                                    };
                                    // Use leaked strings so we get &'static str for Snapshot rows
                                    let lk = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
                                    rows.push((lk(format!("{label_prefix}Mass  M⊕")), format!("{:.4}", mass_val)));
                                    rows.push((lk(format!("{label_prefix}Gravity  g⊕")), format!("{:.3}", moon_gravity(mass_val, mr))));
                                    rows.push((lk(format!("{label_prefix}Angular size")), format_ang(ang)));
                                    rows.push((lk(format!("{label_prefix}Period  days")), format!("{:.1}", moon_orbital_period_days(dst, pm))));
                                    rows.push((lk(format!("{label_prefix}Roche  Rp")), format!("{:.2}", roche_limit_planet_radii(pd, md))));
                                }

                                let snap = Snapshot { name: world_name.get(), rows };
                                snapshots.update(|v| v.push(snap));
                                let n = save_count.get() + 1;
                                save_count.set(n);
                                world_name.set(format!("System {n}"));
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
                    </div>

                    <SectionHeader label=move || t!(i18n, stability_limits) />
                    <ResultRow label=move || t!(i18n, hill_sphere)>
                        {move || format!("{:.4} AU  ({:.0} Rp)", h_au(), h_rp())}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, stable_orbit_limit)>
                        {move || format!("{:.4} AU  ({:.0} Rp)", stab_au(), stab_rp())}
                    </ResultRow>

                    // Per-moon results
                    {move || {
                        let moon_list = moons.get();
                        let pm = planet_mass.get();
                        let pd = planet_density.get();
                        let slrp = stab_rp();

                        moon_list.iter().enumerate().map(|(idx, entry)| {
                            let mr = entry.radius;
                            let md = entry.density;
                            let mdist = entry.distance;
                            let label = format!("Moon {}", idx + 1);

                            let m_mass_val = move || moon_mass(mr.get(), md.get());
                            let m_grav_val = move || moon_gravity(m_mass_val(), mr.get());
                            let m_ang = move || {
                                angular_size_arcmin(mr.get() * R_EARTH_KM, mdist.get() * R_EARTH_KM)
                            };
                            let m_period = move || moon_orbital_period_days(mdist.get(), pm);
                            let m_roche = move || roche_limit_planet_radii(pd, md.get());

                            // orbit validity
                            let orbit_ok = move || is_moon_orbit_valid(mdist.get(), m_roche(), slrp);

                            view! {
                                <SectionHeader label=move || label.clone() />
                                <ResultRow label=move || t!(i18n, mass_earth)>
                                    {move || format!("{:.4}", m_mass_val())}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, surface_gravity)>
                                    {move || format!("{:.3}", m_grav_val())}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, angular_size)>
                                    {move || format_ang(m_ang())}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, orbital_period_days)>
                                    {move || format!("{:.1}", m_period())}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, roche_limit)>
                                    {move || format!("{:.2}", m_roche())}
                                </ResultRow>
                                <div class="flex justify-between items-baseline gap-4 py-2.5 px-3
                                            border-b border-divider/30 rounded hover:bg-edge/20">
                                    <span class="text-label text-sm">{t!(i18n, orbit_valid)}</span>
                                    <span class=move || {
                                        if orbit_ok() {
                                            "text-xs font-semibold px-2.5 py-0.5 rounded-full \
                                             bg-ok/15 text-ok ring-1 ring-ok/25"
                                        } else {
                                            "text-xs font-semibold px-2.5 py-0.5 rounded-full \
                                             bg-err/15 text-err ring-1 ring-err/25"
                                        }
                                    }>
                                        {move || if orbit_ok() { t_string!(i18n, yes_label) } else { t_string!(i18n, no_label) }}
                                    </span>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}

                    // Multi-moon stability analysis
                    {move || {
                        let moon_list = moons.get();
                        if moon_list.len() < 2 {
                            return None;
                        }
                        let pm = planet_mass.get();

                        // gather distances and periods
                        let data: Vec<(f64, f64)> = moon_list.iter().map(|m| {
                            let d = m.distance.get();
                            let p = moon_orbital_period_days(d, pm);
                            (d, p)
                        }).collect();

                        let mut pair_views = Vec::new();
                        for i in 0..data.len() {
                            for j in (i + 1)..data.len() {
                                let (d_inner, p_inner) = if data[i].0 < data[j].0 { data[i] } else { data[j] };
                                let (d_outer, p_outer) = if data[i].0 >= data[j].0 { data[i] } else { data[j] };
                                let stable_pair = are_moons_stable(d_inner, d_outer);
                                let resonance = near_resonance(p_inner, p_outer);
                                let ratio = d_outer / d_inner;
                                let label: &'static str = Box::leak(
                                    format!("Moon {} <-> Moon {}", i + 1, j + 1).into_boxed_str()
                                );

                                pair_views.push(view! {
                                    <div class="flex flex-wrap items-center gap-2 py-2 px-3
                                                border-b border-divider/30 rounded hover:bg-edge/20">
                                        <span class="text-label text-sm flex-1">{label}</span>
                                        <span class="text-[10px] font-mono text-hint">
                                            {format!("ratio {ratio:.2}")}
                                        </span>
                                        <span class=if stable_pair {
                                            "text-[10px] font-semibold px-2 py-0.5 rounded-full \
                                             bg-ok/15 text-ok ring-1 ring-ok/25"
                                        } else {
                                            "text-[10px] font-semibold px-2 py-0.5 rounded-full \
                                             bg-err/15 text-err ring-1 ring-err/25"
                                        }>
                                            {if stable_pair { t_string!(i18n, stable) } else { t_string!(i18n, too_close) }}
                                        </span>
                                        {if resonance {
                                            Some(view! {
                                                <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full
                                                             bg-accent-alt/15 text-accent-alt ring-1 ring-accent-alt/25">
                                                    {t!(i18n, near_resonance)}
                                                </span>
                                            })
                                        } else {
                                            None
                                        }}
                                    </div>
                                });
                            }
                        }

                        Some(view! {
                            <SectionHeader label=move || t!(i18n, multi_moon_stability) />
                            {pair_views}
                        })
                    }}
                </div>
            </div>

            <CompareTable snapshots=snapshots />
        </div>
    }
}
