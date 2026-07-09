use astro_lib::moon::{
    angular_size_arcmin, are_moons_stable, hill_sphere_au, hill_sphere_planet_radii,
    is_moon_orbit_valid, moon_gravity, moon_mass, moon_orbital_period_days, near_resonance,
    roche_limit_planet_radii, stable_orbit_limit,
};
use astro_lib::planet::density;
use crate::i18n::*;
use leptos::prelude::*;

use super::compare::{CompareTable, Snapshot};
use super::planets::{star_inputs, PlanetSigs};
use super::storage::{load_moon_ids, load_planet_ids, ls_f64_dyn, ls_string_dyn, ls_u32_dyn, save_moon_ids};
use super::ui::{filter_numeric, InfoHint, ResultRow, SectionHeader};

const R_EARTH_KM: f64 = 6_371.0;

/// Per-moon reactive state, persisted under `moon_{id}_*`.
#[derive(Clone)]
struct MoonEntry {
    id: u32,
    parent: RwSignal<u32>,
    name: RwSignal<String>,
    radius: RwSignal<f64>,
    density: RwSignal<f64>,
    distance: RwSignal<f64>,
    radius_text: RwSignal<String>,
    density_text: RwSignal<String>,
    distance_text: RwSignal<String>,
}

fn sync_text(val: RwSignal<f64>, text: RwSignal<String>) {
    Effect::new(move |_| {
        let v = val.get();
        let cur = text.get_untracked();
        if cur.replace(',', ".").parse::<f64>().ok() != Some(v) {
            text.set(v.to_string());
        }
    });
}

impl MoonEntry {
    fn new(id: u32, default_parent: u32) -> Self {
        let parent = ls_u32_dyn(format!("moon_{id}_parent"), default_parent);
        let name = ls_string_dyn(format!("moon_{id}_name"), String::new());
        let radius = ls_f64_dyn(format!("moon_{id}_radius"), 0.273);
        let density = ls_f64_dyn(format!("moon_{id}_density"), 0.606);
        let distance = ls_f64_dyn(format!("moon_{id}_dist"), 60.27);
        let radius_text = RwSignal::new(radius.get().to_string());
        let density_text = RwSignal::new(density.get().to_string());
        let distance_text = RwSignal::new(distance.get().to_string());
        sync_text(radius, radius_text);
        sync_text(density, density_text);
        sync_text(distance, distance_text);
        Self {
            id, parent, name, radius, density, distance,
            radius_text, density_text, distance_text,
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

    // ── parent planets & star ───────────────────────────────────────────────
    let planets: Vec<PlanetSigs> =
        load_planet_ids().into_iter().map(PlanetSigs::new).collect();
    let first_planet_id = planets[0].id;
    let stars = star_inputs();

    // Localized fallback for unnamed bodies: "Planet 2" / "Планета 2".
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
    let moon_name = move |m: &MoonEntry, idx: usize| -> String {
        let nm = m.name.get();
        if nm.is_empty() {
            format!("{} {}", t_string!(i18n, moon), idx + 1)
        } else {
            nm
        }
    };

    // ── dynamic moon list (persisted as "moon_ids") ─────────────────────────
    let stored_ids = load_moon_ids();
    let next_id: RwSignal<u32> =
        RwSignal::new(stored_ids.iter().max().map_or(1, |max| max + 1));
    let moons: RwSignal<Vec<MoonEntry>> = RwSignal::new(
        stored_ids
            .into_iter()
            .map(|id| MoonEntry::new(id, first_planet_id))
            .collect(),
    );

    Effect::new(move |_| {
        let ids: Vec<u32> = moons.with(|v| v.iter().map(|m| m.id).collect());
        save_moon_ids(&ids);
    });

    let add_moon = move |_| {
        let id = next_id.get();
        next_id.set(id + 1);
        moons.update(|v| v.push(MoonEntry::new(id, first_planet_id)));
    };

    // ── save / compare ──────────────────────────────────────────────────────
    let snapshots: RwSignal<Vec<Snapshot>> = RwSignal::new(vec![]);
    let world_name = RwSignal::new(String::from("System 1"));
    let save_count = RwSignal::new(1_u32);

    // ── view ────────────────────────────────────────────────────────────────
    let input_planets = planets.clone();
    let results_planets = planets.clone();
    let results_planet_name = planet_name.clone();
    let results_moon_name = moon_name.clone();
    let snap_planets = planets.clone();
    let snap_planet_name = planet_name.clone();
    let snap_moon_name = moon_name.clone();

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

                    // ── Moon entries ────────────────────────────────────────
                    <div class="flex items-center justify-between">
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
                        let planets = input_planets.clone();
                        moon_list.into_iter().enumerate().map(|(idx, entry)| {
                            let entry_id = entry.id;
                            let parent = entry.parent;
                            let name = entry.name;
                            let mr = entry.radius;
                            let md = entry.density;
                            let mdist = entry.distance;
                            let mr_t = entry.radius_text;
                            let md_t = entry.density_text;
                            let mdist_t = entry.distance_text;
                            let planets = planets.clone();
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
                                    <input
                                        type="text"
                                        placeholder=move || t_string!(i18n, moon_name_placeholder)
                                        class="bg-base border border-edge rounded-lg px-3 py-1.5
                                               text-heading text-sm outline-none
                                               focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                        prop:value=move || name.get()
                                        on:input=move |ev| name.set(event_target_value(&ev))
                                    />
                                    <div class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint">
                                            {t!(i18n, parent_planet)}
                                        </span>
                                        <select
                                            class="bg-base border border-edge rounded-lg px-2 py-1.5
                                                   text-heading text-sm outline-none cursor-pointer
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:change=move |ev| {
                                                if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                                    parent.set(v);
                                                }
                                            }
                                        >
                                            {planets.iter().enumerate().map(|(p_idx, p)| {
                                                let p_id = p.id;
                                                let p_name = p.name;
                                                view! {
                                                    <option
                                                        value=p_id.to_string()
                                                        selected=move || parent.get() == p_id
                                                    >
                                                        {move || {
                                                            let nm = p_name.get();
                                                            if nm.is_empty() {
                                                                format!("{} {}", t_string!(i18n, planet), p_idx + 1)
                                                            } else {
                                                                nm
                                                            }
                                                        }}
                                                    </option>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    </div>
                                    <div class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint flex items-center gap-1">
                                            {t!(i18n, radius_earth)}
                                            <InfoHint text=move || t!(i18n, hint_moon_radius) />
                                        </span>
                                        <input type="text" inputmode="decimal"
                                            prop:value=move || mr_t.get()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                let filtered = filter_numeric(&event_target_value(&ev));
                                                mr_t.set(filtered.clone());
                                                if let Ok(v) = filtered.replace(',', ".").parse::<f64>() { mr.set(v); }
                                            }
                                        />
                                    </div>
                                    <div class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint flex items-center gap-1">
                                            {t!(i18n, density)}
                                            <InfoHint text=move || t!(i18n, hint_moon_density) />
                                        </span>
                                        <input type="text" inputmode="decimal"
                                            prop:value=move || md_t.get()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                let filtered = filter_numeric(&event_target_value(&ev));
                                                md_t.set(filtered.clone());
                                                if let Ok(v) = filtered.replace(',', ".").parse::<f64>() { md.set(v); }
                                            }
                                        />
                                    </div>
                                    <div class="flex flex-col gap-1">
                                        <span class="text-[10px] text-hint flex items-center gap-1">
                                            {t!(i18n, distance_rp)}
                                            <InfoHint text=move || t!(i18n, hint_moon_distance) />
                                        </span>
                                        <input type="text" inputmode="decimal"
                                            prop:value=move || mdist_t.get()
                                            class="bg-base border border-edge rounded-lg px-3 py-1.5
                                                   text-heading text-sm font-mono outline-none
                                                   focus:border-accent focus:ring-1 focus:ring-accent/40 w-full"
                                            on:input=move |ev| {
                                                let filtered = filter_numeric(&event_target_value(&ev));
                                                mdist_t.set(filtered.clone());
                                                if let Ok(v) = filtered.replace(',', ".").parse::<f64>() { mdist.set(v); }
                                            }
                                        />
                                    </div>
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
                                let moon_list = moons.get();

                                let ek = Locale::en.get_keys_const();
                                let rk = Locale::ru.get_keys_const();
                                macro_rules! lbl {
                                    ($key:ident) => { [ek.$key().inner().to_owned(), rk.$key().inner().to_owned()] }
                                }

                                let mut rows: Vec<([String; 2], String)> = Vec::new();
                                for p in &snap_planets {
                                    let group: Vec<(usize, &MoonEntry)> = moon_list
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, m)| m.parent.get() == p.id)
                                        .collect();
                                    if group.is_empty() {
                                        continue;
                                    }
                                    let pname = snap_planet_name(p.id);
                                    let sm = stars.params_for(p.host.get()).kepler_mass;
                                    let pm = p.mass.get();
                                    let pr = p.eff_radius();
                                    let pd = density(pm, pr);
                                    let oa = p.semi_major.get();
                                    let h_au_val = hill_sphere_au(oa, pm, sm);
                                    let h_rp_val = hill_sphere_planet_radii(oa, pm, sm, pr);

                                    let with_pname = |l: [String; 2]| -> [String; 2] {
                                        [format!("{pname} — {}", l[0]), format!("{pname} — {}", l[1])]
                                    };
                                    rows.push((
                                        with_pname(lbl!(hill_sphere)),
                                        format!("{:.4} AU  ({:.0} Rp)", h_au_val, h_rp_val),
                                    ));
                                    rows.push((
                                        with_pname(lbl!(stable_orbit_limit)),
                                        format!("{:.4} AU  ({:.0} Rp)",
                                            stable_orbit_limit(h_au_val), stable_orbit_limit(h_rp_val)),
                                    ));

                                    for (idx, m) in group {
                                        let mname = snap_moon_name(m, idx);
                                        let mr = m.radius.get();
                                        let md = m.density.get();
                                        let dst = m.distance.get();
                                        let mass_val = moon_mass(mr, md);
                                        let ang = angular_size_arcmin(mr * R_EARTH_KM, dst * R_EARTH_KM);
                                        let lbl_m = |l: [String; 2]| -> [String; 2] {
                                            [format!("{mname} — {}", l[0]), format!("{mname} — {}", l[1])]
                                        };
                                        rows.push((lbl_m(lbl!(mass_earth)),          format!("{:.4}", mass_val)));
                                        rows.push((lbl_m(lbl!(surface_gravity)),     format!("{:.3}", moon_gravity(mass_val, mr))));
                                        rows.push((lbl_m(lbl!(angular_size)),        format_ang(ang)));
                                        rows.push((lbl_m(lbl!(orbital_period_days)), format!("{:.1}", moon_orbital_period_days(dst, pm))));
                                        rows.push((lbl_m(lbl!(roche_limit)),         format!("{:.2}", roche_limit_planet_radii(pd, md))));
                                    }
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

                    {move || {
                        let moon_list = moons.get();
                        let mut sections: Vec<AnyView> = Vec::new();

                        for p in &results_planets {
                            let sm = stars.params_for(p.host.get()).kepler_mass;
                            let group: Vec<(usize, MoonEntry)> = moon_list
                                .iter()
                                .enumerate()
                                .filter(|(_, m)| m.parent.get() == p.id)
                                .map(|(i, m)| (i, m.clone()))
                                .collect();
                            if group.is_empty() {
                                continue;
                            }

                            let pname = results_planet_name(p.id);
                            let pm = p.mass.get();
                            let pr = p.eff_radius();
                            let pd = density(pm, pr);
                            let oa = p.semi_major.get();
                            let h_au = hill_sphere_au(oa, pm, sm);
                            let h_rp = hill_sphere_planet_radii(oa, pm, sm, pr);
                            let stab_rp = stable_orbit_limit(h_rp);

                            sections.push(view! {
                                <SectionHeader label={
                                    let pname = pname.clone();
                                    move || pname.clone()
                                } />
                                <ResultRow label=move || t!(i18n, hill_sphere)
                                    hint=move || t!(i18n, hint_hill_sphere)>
                                    {format!("{:.4} AU  ({:.0} Rp)", h_au, h_rp)}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, stable_orbit_limit)
                                    hint=move || t!(i18n, hint_stable_orbit)>
                                    {format!("{:.4} AU  ({:.0} Rp)",
                                        stable_orbit_limit(h_au), stab_rp)}
                                </ResultRow>
                            }.into_any());

                            // Per-moon results
                            for (idx, m) in &group {
                                let label = results_moon_name(m, *idx);
                                let mr = m.radius.get();
                                let md = m.density.get();
                                let dst = m.distance.get();
                                let mass_val = moon_mass(mr, md);
                                let roche = roche_limit_planet_radii(pd, md);
                                let orbit_ok = is_moon_orbit_valid(dst, roche, stab_rp);

                                sections.push(view! {
                                    <p class="text-[11px] font-semibold text-label pt-3 pb-1 px-3">
                                        {label}
                                    </p>
                                    <ResultRow label=move || t!(i18n, mass_earth)
                                        hint=move || t!(i18n, hint_moon_mass)>
                                        {format!("{mass_val:.4}")}
                                    </ResultRow>
                                    <ResultRow label=move || t!(i18n, surface_gravity)
                                        hint=move || t!(i18n, hint_moon_gravity)>
                                        {format!("{:.3}", moon_gravity(mass_val, mr))}
                                    </ResultRow>
                                    <ResultRow label=move || t!(i18n, angular_size)
                                        hint=move || t!(i18n, hint_angular_size)>
                                        {format_ang(angular_size_arcmin(mr * R_EARTH_KM, dst * R_EARTH_KM))}
                                    </ResultRow>
                                    <ResultRow label=move || t!(i18n, orbital_period_days)
                                        hint=move || t!(i18n, hint_moon_period)>
                                        {format!("{:.1}", moon_orbital_period_days(dst, pm))}
                                    </ResultRow>
                                    <ResultRow label=move || t!(i18n, roche_limit)
                                        hint=move || t!(i18n, hint_roche_limit)>
                                        {format!("{roche:.2}")}
                                    </ResultRow>
                                    <div class="flex justify-between items-start gap-4 py-2.5 px-3
                                                border-b border-divider/30 rounded hover:bg-edge/20">
                                        <span class="text-label text-sm flex items-center gap-1 flex-1 min-w-0 flex-wrap">
                                            {t!(i18n, orbit_valid)}
                                            <InfoHint text=move || t!(i18n, hint_orbit_valid) />
                                        </span>
                                        <span class=if orbit_ok {
                                            "text-xs font-semibold px-2.5 py-0.5 rounded-full shrink-0 \
                                             bg-ok/15 text-ok ring-1 ring-ok/25"
                                        } else {
                                            "text-xs font-semibold px-2.5 py-0.5 rounded-full shrink-0 \
                                             bg-err/15 text-err ring-1 ring-err/25"
                                        }>
                                            {if orbit_ok { t_string!(i18n, yes_label) } else { t_string!(i18n, no_label) }}
                                        </span>
                                    </div>
                                }.into_any());
                            }

                            // Multi-moon stability inside this planet's group
                            if group.len() >= 2 {
                                let data: Vec<(String, f64, f64)> = group
                                    .iter()
                                    .map(|(idx, m)| {
                                        let d = m.distance.get();
                                        (results_moon_name(m, *idx), d, moon_orbital_period_days(d, pm))
                                    })
                                    .collect();

                                let mut pair_views: Vec<AnyView> = Vec::new();
                                for i in 0..data.len() {
                                    for j in (i + 1)..data.len() {
                                        let (d_inner, p_inner) =
                                            if data[i].1 < data[j].1 { (data[i].1, data[i].2) } else { (data[j].1, data[j].2) };
                                        let (d_outer, p_outer) =
                                            if data[i].1 >= data[j].1 { (data[i].1, data[i].2) } else { (data[j].1, data[j].2) };
                                        let stable_pair = are_moons_stable(d_inner, d_outer);
                                        let resonance = near_resonance(p_inner, p_outer);
                                        let ratio = d_outer / d_inner;
                                        let label = format!("{} <-> {}", data[i].0, data[j].0);

                                        pair_views.push(view! {
                                            <div class="flex flex-wrap items-center gap-2 py-2 px-3
                                                        border-b border-divider/30 rounded hover:bg-edge/20">
                                                <span class="text-label text-sm flex-1">{label}</span>
                                                <span class="text-[10px] font-mono text-hint">
                                                    {format!("{} {ratio:.2}", t_string!(i18n, ratio_label))}
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
                                                {resonance.then(|| view! {
                                                    <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full
                                                                 bg-accent-alt/15 text-accent-alt ring-1 ring-accent-alt/25">
                                                        {t!(i18n, near_resonance)}
                                                    </span>
                                                })}
                                            </div>
                                        }.into_any());
                                    }
                                }

                                sections.push(view! {
                                    <SectionHeader label=move || t!(i18n, multi_moon_stability) />
                                    {pair_views}
                                }.into_any());
                            }
                        }

                        if sections.is_empty() {
                            sections.push(view! {
                                <p class="text-xs text-hint py-3 px-3">{t!(i18n, no_moons_note)}</p>
                            }.into_any());
                        }
                        sections
                    }}
                </div>
            </div>

            <CompareTable snapshots=snapshots />
        </div>
    }
}
