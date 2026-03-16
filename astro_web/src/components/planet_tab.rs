use astro_lib::atmosphere::{
    atmosphere_retention, equilibrium_temperature, exosphere_temperature_estimate,
    greenhouse_effect, scale_height, surface_pressure_estimate, surface_temperature,
};
use astro_lib::habitability::{is_habitable_tilt, is_in_habitable_zone};
use astro_lib::orbit::{aphelion, orbital_period, orbital_velocity, perihelion, polar_circle, tropic_latitude};
use astro_lib::planet::{
    density, escape_velocity, gravity, has_solid_surface, planet_radius_auto, planet_type,
    surface_area, volume,
};
use astro_lib::star::{habitable_zone, luminosity, radius, temperature};
use crate::i18n::*;
use leptos::prelude::*;

use super::compare::{CompareTable, Snapshot};
use super::storage::{ls_bool, ls_f64};
use super::ui::{filter_numeric, BoolRow, NumberInput, ResultRow, SectionHeader};

#[component]
pub fn PlanetTab() -> impl IntoView {
    let i18n = use_i18n();

    // ── inputs ────────────────────────────────────────────────────────────────
    let planet_mass   = ls_f64("planet_mass",   1.0);
    let use_manual_r  = ls_bool("planet_use_manual_r", false);
    let manual_radius = ls_f64("planet_manual_radius", 1.0);
    let semi_major    = ls_f64("planet_semi_major", 1.0);
    let eccentricity  = ls_f64("planet_eccentricity", 0.017);
    let axial_tilt    = ls_f64("planet_axial_tilt", 23.4);

    // star mass: linked from star tab by default, or custom override
    let shared_star_mass  = ls_f64("star_mass", 1.0);
    let custom_star       = ls_bool("planet_custom_star", false);
    let custom_star_mass  = ls_f64("planet_custom_star_mass", 1.0);
    let star_mass = Signal::derive(move || {
        if custom_star.get() { custom_star_mass.get() } else { shared_star_mass.get() }
    });

    // atmosphere inputs
    let albedo         = ls_f64("planet_albedo", 0.3);
    let co2_fraction   = ls_f64("planet_co2_fraction", 0.0004);
    let atmo_mass      = ls_f64("planet_atmo_mass", 1.0);

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
    let period  = move || orbital_period(semi_major.get(), star_mass.get());
    let v_orb   = move || orbital_velocity(semi_major.get(), star_mass.get());

    let period_display = move || {
        let p = period();
        format!("{:.3} yr  ({:.1} days)", p.years, p.days)
    };

    // ── axial tilt ──────────────────────────────────────────────────────────
    let tropic  = move || tropic_latitude(axial_tilt.get());
    let polar   = move || polar_circle(axial_tilt.get());

    // ── atmosphere ──────────────────────────────────────────────────────────
    let star_temp_rel = move || temperature(star_mass.get()).unwrap_or(1.0);
    let star_rad_rel  = move || {
        let l = luminosity(star_mass.get()).unwrap_or(1.0);
        let t = star_temp_rel();
        radius(l, t)
    };

    let t_eq = move || equilibrium_temperature(
        star_temp_rel(), star_rad_rel(), semi_major.get(), albedo.get(),
    );
    let t_exo = move || exosphere_temperature_estimate(t_eq());
    let s_press = move || surface_pressure_estimate(grav(), atmo_mass.get());
    let gh_delta = move || greenhouse_effect(s_press(), co2_fraction.get());
    let t_surf = move || surface_temperature(t_eq(), gh_delta());
    let sh = move || scale_height(grav());

    // ── habitability ────────────────────────────────────────────────────────
    let in_hz = Signal::derive(move || {
        luminosity(star_mass.get())
            .map(|l| is_in_habitable_zone(semi_major.get(), &habitable_zone(l)))
            .unwrap_or(false)
    });
    let good_tilt = Signal::derive(move || is_habitable_tilt(axial_tilt.get()));

    // ── save / compare ──────────────────────────────────────────────────────
    let snapshots: RwSignal<Vec<Snapshot>> = RwSignal::new(vec![]);
    let world_name = RwSignal::new(String::from("Planet 1"));
    let save_count = RwSignal::new(1_u32);

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="flex flex-col gap-8">
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
                                    {move || format!("{:.3}", shared_star_mass.get())}
                                    <span class="text-[10px] text-hint ml-1">"M☉"</span>
                                </div>
                            }.into_any()
                        }}
                    </div>

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
                                let sm = star_mass.get();
                                let t  = axial_tilt.get();
                                let p  = orbital_period(a, sm);
                                let hz_bool = luminosity(sm)
                                    .map(|l| is_in_habitable_zone(a, &habitable_zone(l)))
                                    .unwrap_or(false);

                                let mut rows = vec![
                                    (t_string!(i18n, planet_type_label).to_owned(), match planet_type(m) {
                                        astro_lib::planet::PlanetType::Rocky       => t_string!(i18n, type_rocky).to_owned(),
                                        astro_lib::planet::PlanetType::SubNeptune  => t_string!(i18n, type_sub_neptune).to_owned(),
                                        astro_lib::planet::PlanetType::GasGiant    => t_string!(i18n, type_gas_giant).to_owned(),
                                        astro_lib::planet::PlanetType::SuperJovian => t_string!(i18n, type_super_jovian).to_owned(),
                                    }),
                                    (t_string!(i18n, radius_earth).to_owned(),      format!("{:.3}", r)),
                                    (t_string!(i18n, gravity).to_owned(),           format!("{:.3}", gravity(m, r))),
                                    (t_string!(i18n, density).to_owned(),           format!("{:.3}", density(m, r))),
                                    (t_string!(i18n, escape_velocity).to_owned(),   format!("{:.3}", escape_velocity(m, r))),
                                    (t_string!(i18n, surface_area).to_owned(),      format!("{:.3}", surface_area(r))),
                                    (t_string!(i18n, volume).to_owned(),            format!("{:.3}", volume(r))),
                                    (t_string!(i18n, semi_major_axis_au).to_owned(), format!("{:.3}", a)),
                                    (t_string!(i18n, aphelion_au).to_owned(),       format!("{:.3}", aphelion(a, e))),
                                    (t_string!(i18n, perihelion_au).to_owned(),     format!("{:.3}", perihelion(a, e))),
                                    (t_string!(i18n, orbital_period).to_owned(),    format!("{:.3} yr  ({:.1} days)", p.years, p.days)),
                                    (t_string!(i18n, orbital_velocity).to_owned(),  format!("{:.3}", orbital_velocity(a, sm))),
                                    (t_string!(i18n, tropic_latitude).to_owned(),   format!("{:.1}", tropic_latitude(t))),
                                    (t_string!(i18n, polar_circle).to_owned(),      format!("{:.1}", polar_circle(t))),
                                    (t_string!(i18n, in_habitable_zone).to_owned(), if hz_bool { "✓" } else { "✗" }.to_string()),
                                    (t_string!(i18n, habitable_tilt).to_owned(),    if is_habitable_tilt(t) { "✓" } else { "✗" }.to_string()),
                                ];

                                // atmosphere snapshot rows
                                let g = gravity(m, r);
                                let ve = escape_velocity(m, r);
                                let st = temperature(sm).unwrap_or(1.0);
                                let l = luminosity(sm).unwrap_or(1.0);
                                let sr = radius(l, st);
                                let al = albedo.get();
                                let teq = equilibrium_temperature(st, sr, a, al);
                                let texo = exosphere_temperature_estimate(teq);
                                let sp = surface_pressure_estimate(g, atmo_mass.get());
                                let ghd = greenhouse_effect(sp, co2_fraction.get());
                                let ts = surface_temperature(teq, ghd);
                                rows.push((t_string!(i18n, equilibrium_temp).to_owned(), format!("{:.0}", teq)));
                                rows.push((t_string!(i18n, surface_temp).to_owned(),     format!("{:.0}", ts)));
                                rows.push((t_string!(i18n, surface_pressure).to_owned(), format!("{:.2}", sp)));

                                let ret = atmosphere_retention(ve, texo);
                                let gases = [
                                    ("H₂", ret.hydrogen), ("He", ret.helium),
                                    ("N₂", ret.nitrogen), ("O₂", ret.oxygen),
                                    ("H₂O", ret.water_vapor), ("CO₂", ret.co2),
                                ];
                                let retained: Vec<&str> = gases.iter()
                                    .filter(|(_, ok)| *ok).map(|(n, _)| *n).collect();
                                rows.push((t_string!(i18n, gas_retention).to_owned(), retained.join(", ")));

                                let snap = Snapshot { name: world_name.get(), rows };
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

                    <SectionHeader label=move || t!(i18n, habitability) />
                    <BoolRow label=move || t!(i18n, in_habitable_zone) value=in_hz
                        hint=move || t!(i18n, hint_in_hz) />
                    <BoolRow label=move || t!(i18n, habitable_tilt) value=good_tilt
                        hint=move || t!(i18n, hint_habitable_tilt) />
                </div>
            </div>

            <CompareTable snapshots=snapshots />
        </div>
    }
}
