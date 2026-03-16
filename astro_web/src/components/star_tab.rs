use astro_lib::binary::{binary_orbital_period, combined_luminosity, p_type_critical_radius, s_type_critical_radius};
use astro_lib::error::StarErr;
use astro_lib::flora::{pigment_display_color, predict_flora_pigment};
use astro_lib::habitability::is_habitable_star_mass;
use astro_lib::star::{
    frost_line, habitable_zone, lifetime, luminosity, peak_wavelength, radius, spectral_class,
    system_boundaries, temperature, SpectralClass,
};
use crate::i18n::*;
use leptos::prelude::*;

use super::compare::{CompareTable, Snapshot};
use super::storage::{ls_bool, ls_f64};
use super::ui::{BoolRow, InfoHint, NumberInput, ResultRow, SectionHeader, fmt_result};

#[component]
pub fn StarTab() -> impl IntoView {
    let i18n = use_i18n();
    let mass = ls_f64("star_mass", 1.0);

    // ── binary mode ─────────────────────────────────────────────────────────
    let binary_mode   = ls_bool("star_binary_mode", false);
    let mass_b        = ls_f64("star_b_mass", 0.8);
    let bin_sep       = ls_f64("binary_separation", 20.0);
    let bin_ecc       = ls_f64("binary_eccentricity", 0.4);
    let bin_p_type    = ls_bool("binary_p_type", false);

    // ── computed (Star A) ───────────────────────────────────────────────────
    let lum  = move || luminosity(mass.get());
    let temp = move || temperature(mass.get());
    let rad  = move || {
        let l = luminosity(mass.get())?;
        let t = temperature(mass.get())?;
        Ok::<f64, StarErr>(radius(l, t))
    };
    let life = move || lifetime(mass.get());

    let temp_with_class = move || match temp() {
        Ok(t) => format!("{t:.3}  ({})", spectral_class(t)),
        Err(e) => e.to_string(),
    };

    let hz_display = move || match lum() {
        Ok(l) => {
            let hz = habitable_zone(l);
            format!("{:.2} – {:.2} AU", hz.inner, hz.outer)
        }
        Err(e) => e.to_string(),
    };

    let frost_display  = move || fmt_result(lum().map(frost_line), 2);
    let sys_inner      = move || fmt_result(system_boundaries(mass.get()).map(|s| s.inner), 2);
    let sys_outer      = move || fmt_result(system_boundaries(mass.get()).map(|s| s.outer), 1);
    let peak_display   = move || fmt_result(temp().map(peak_wavelength), 1);

    let habitable = Signal::derive(move || is_habitable_star_mass(mass.get()));

    // ── computed (Star B) ───────────────────────────────────────────────────
    let lum_b  = move || luminosity(mass_b.get());
    let temp_b = move || temperature(mass_b.get());
    let rad_b  = move || {
        let l = luminosity(mass_b.get())?;
        let t = temperature(mass_b.get())?;
        Ok::<f64, StarErr>(radius(l, t))
    };
    let life_b = move || lifetime(mass_b.get());

    let temp_b_with_class = move || match temp_b() {
        Ok(t) => format!("{t:.3}  ({})", spectral_class(t)),
        Err(e) => e.to_string(),
    };
    let peak_b_display = move || fmt_result(temp_b().map(peak_wavelength), 1);

    // ── binary computed ─────────────────────────────────────────────────────
    let bin_period = move || binary_orbital_period(bin_sep.get(), mass.get(), mass_b.get());
    let comb_lum   = move || combined_luminosity(mass.get(), mass_b.get());
    let bin_hz_display = move || match comb_lum() {
        Ok(l) => {
            let hz = habitable_zone(l);
            format!("{:.2} – {:.2} AU", hz.inner, hz.outer)
        }
        Err(e) => e.to_string(),
    };
    let s_crit = move || s_type_critical_radius(
        bin_sep.get(), bin_ecc.get(), mass.get(), mass_b.get(),
    );
    let p_crit = move || p_type_critical_radius(
        bin_sep.get(), bin_ecc.get(), mass.get(), mass_b.get(),
    );

    // ── save / compare ──────────────────────────────────────────────────────
    let snapshots: RwSignal<Vec<Snapshot>> = RwSignal::new(vec![]);
    let world_name  = RwSignal::new(String::from("Star 1"));
    let save_count  = RwSignal::new(1_u32);

    // ── view ────────────────────────────────────────────────────────────────
    view! {
        <div class="flex flex-col gap-8">
            <div class="grid grid-cols-1 lg:grid-cols-[300px_1fr] gap-6 items-start">

                // ── Inputs card ─────────────────────────────────────────────
                <div class="bg-card border border-edge rounded-2xl p-6 pb-8 flex flex-col gap-5">
                    <div class="flex items-center gap-2">
                        <span class="text-base text-accent">"★"</span>
                        <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                            {t!(i18n, inputs)}
                        </h2>
                    </div>

                    <p class="text-[10px] font-semibold text-hint uppercase tracking-widest">
                        {move || if binary_mode.get() { t_string!(i18n, star_a) } else { t_string!(i18n, star) }}
                    </p>
                    <NumberInput label=move || t!(i18n, star_mass) value=mass unit="M☉" step="0.01"
                        hint=move || t!(i18n, hint_star_mass_input) />

                    // Binary mode toggle
                    <button
                        class=move || {
                            if binary_mode.get() {
                                "text-[10px] font-medium px-3 py-1 rounded-full cursor-pointer \
                                 bg-accent/15 text-accent ring-1 ring-accent/20 self-start"
                            } else {
                                "text-[10px] font-medium px-3 py-1 rounded-full cursor-pointer \
                                 bg-edge/40 text-hint ring-1 ring-edge hover:text-label self-start"
                            }
                        }
                        on:click=move |_| binary_mode.update(|v| *v = !*v)
                    >
                        {move || if binary_mode.get() { t_string!(i18n, binary_on) } else { t_string!(i18n, binary_off) }}
                    </button>

                    // Binary inputs (conditional)
                    {move || if binary_mode.get() {
                        Some(view! {
                            <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                                {t!(i18n, star_b)}
                            </p>
                            <NumberInput label=move || t!(i18n, star_b_mass) value=mass_b unit="M☉" step="0.01"
                                hint=move || t!(i18n, hint_star_b_mass) />

                            <p class="text-[10px] font-semibold text-hint uppercase tracking-widest pt-2">
                                {t!(i18n, binary_orbit)}
                            </p>
                            <NumberInput label=move || t!(i18n, separation) value=bin_sep unit="AU" step="0.5"
                                hint=move || t!(i18n, hint_separation) />
                            <NumberInput label=move || t!(i18n, eccentricity) value=bin_ecc step="0.01"
                                hint=move || t!(i18n, hint_bin_eccentricity) />

                            <div class="flex items-center gap-2">
                                <span class="text-xs text-label">{t!(i18n, orbit_type)}</span>
                                <button
                                    class=move || {
                                        if !bin_p_type.get() {
                                            "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                             bg-accent/15 text-accent ring-1 ring-accent/20"
                                        } else {
                                            "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                             bg-edge/40 text-hint ring-1 ring-edge hover:text-label"
                                        }
                                    }
                                    on:click=move |_| bin_p_type.set(false)
                                >
                                    {t!(i18n, s_type)}
                                </button>
                                <button
                                    class=move || {
                                        if bin_p_type.get() {
                                            "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                             bg-accent/15 text-accent ring-1 ring-accent/20"
                                        } else {
                                            "text-[10px] font-medium px-2 py-0.5 rounded-full cursor-pointer \
                                             bg-edge/40 text-hint ring-1 ring-edge hover:text-label"
                                        }
                                    }
                                    on:click=move |_| bin_p_type.set(true)
                                >
                                    {t!(i18n, p_type)}
                                </button>
                            </div>
                        })
                    } else {
                        None
                    }}

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
                                let m = mass.get();
                                let ek = Locale::en.get_keys_const();
                                let rk = Locale::ru.get_keys_const();
                                let ck = i18n.get_locale_untracked().get_keys_const();
                                // Helper: bilingual label [en, ru]
                                macro_rules! lbl {
                                    ($key:ident) => { [ek.$key().inner().to_owned(), rk.$key().inner().to_owned()] }
                                }
                                let mut rows = vec![
                                    (lbl!(luminosity),        fmt_result(luminosity(m), 3)),
                                    (lbl!(temperature_class), match temperature(m) {
                                        Ok(t) => format!("{t:.3}  ({})", spectral_class(t)),
                                        Err(e) => e.to_string(),
                                    }),
                                    (lbl!(radius_solar), fmt_result(
                                        luminosity(m).and_then(|l|
                                            temperature(m).map(|t| radius(l, t))),
                                        3
                                    )),
                                    (lbl!(lifetime),        fmt_result(lifetime(m), 2)),
                                    (lbl!(peak_wavelength), fmt_result(temperature(m).map(peak_wavelength), 1)),
                                    (lbl!(hz_inner_outer), match luminosity(m) {
                                        Ok(l) => {
                                            let hz = habitable_zone(l);
                                            format!("{:.2} – {:.2}", hz.inner, hz.outer)
                                        }
                                        Err(e) => e.to_string(),
                                    }),
                                    (lbl!(frost_line), fmt_result(luminosity(m).map(frost_line), 2)),
                                    (lbl!(predicted_pigment), match temperature(m) {
                                        Ok(t) => {
                                            let pred = predict_flora_pigment(spectral_class(t), peak_wavelength(t));
                                            match pred.pigment {
                                                astro_lib::flora::FloraPigment::Green     => ck.pigment_green().inner().to_owned(),
                                                astro_lib::flora::FloraPigment::Red       => ck.pigment_red().inner().to_owned(),
                                                astro_lib::flora::FloraPigment::Orange    => ck.pigment_orange().inner().to_owned(),
                                                astro_lib::flora::FloraPigment::Blue      => ck.pigment_blue().inner().to_owned(),
                                                astro_lib::flora::FloraPigment::Black     => ck.pigment_black().inner().to_owned(),
                                                astro_lib::flora::FloraPigment::UvHostile => ck.pigment_uv_hostile().inner().to_owned(),
                                            }
                                        }
                                        Err(e) => e.to_string(),
                                    }),
                                ];

                                if binary_mode.get() {
                                    let mb = mass_b.get();
                                    rows.push((lbl!(star_b_lum),          fmt_result(luminosity(mb), 3)));
                                    rows.push((lbl!(binary_period),        format!("{:.1}", binary_orbital_period(bin_sep.get(), m, mb))));
                                    rows.push((lbl!(combined_luminosity),  fmt_result(combined_luminosity(m, mb), 3)));
                                    rows.push((lbl!(s_type_max_orbit),     format!("{:.2}", s_type_critical_radius(bin_sep.get(), bin_ecc.get(), m, mb))));
                                    rows.push((lbl!(p_type_min_orbit),     format!("{:.2}", p_type_critical_radius(bin_sep.get(), bin_ecc.get(), m, mb))));
                                }

                                let snap = Snapshot { name: world_name.get(), rows };
                                snapshots.update(|v| v.push(snap));
                                let n = save_count.get() + 1;
                                save_count.set(n);
                                world_name.set(format!("Star {n}"));
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

                    {move || if binary_mode.get() {
                        view! { <SectionHeader label=move || t!(i18n, star_a_properties) /> }.into_any()
                    } else {
                        view! { <SectionHeader label=move || t!(i18n, star_properties) /> }.into_any()
                    }}
                    <ResultRow label=move || t!(i18n, luminosity)
                        hint=move || t!(i18n, hint_luminosity)>
                        {move || fmt_result(lum(), 3)}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, temperature_class)
                        hint=move || t!(i18n, hint_temperature)>
                        {temp_with_class}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, radius_solar)
                        hint=move || t!(i18n, hint_radius_solar)>
                        {move || fmt_result(rad(), 3)}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, lifetime)
                        hint=move || t!(i18n, hint_lifetime)>
                        {move || fmt_result(life(), 2)}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, peak_wavelength)
                        hint=move || t!(i18n, hint_peak_wavelength)>
                        {peak_display}
                    </ResultRow>

                    // Star B results (conditional)
                    {move || if binary_mode.get() {
                        Some(view! {
                            <SectionHeader label=move || t!(i18n, star_b_properties) />
                            <ResultRow label=move || t!(i18n, luminosity)
                                hint=move || t!(i18n, hint_luminosity)>
                                {move || fmt_result(lum_b(), 3)}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, temperature_class)
                                hint=move || t!(i18n, hint_temperature)>
                                {temp_b_with_class}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, radius_solar)
                                hint=move || t!(i18n, hint_radius_solar)>
                                {move || fmt_result(rad_b(), 3)}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, lifetime)
                                hint=move || t!(i18n, hint_lifetime)>
                                {move || fmt_result(life_b(), 2)}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, peak_wavelength)
                                hint=move || t!(i18n, hint_peak_wavelength)>
                                {peak_b_display}
                            </ResultRow>

                            <SectionHeader label=move || t!(i18n, binary_system) />
                            <ResultRow label=move || t!(i18n, binary_period)
                                hint=move || t!(i18n, hint_bin_period)>
                                {move || format!("{:.1}", bin_period())}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, combined_luminosity)
                                hint=move || t!(i18n, hint_combined_lum)>
                                {move || fmt_result(comb_lum(), 3)}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, combined_hz)
                                hint=move || t!(i18n, hint_combined_hz)>
                                {bin_hz_display}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, s_type_max_orbit)
                                hint=move || t!(i18n, hint_s_type_max)>
                                {move || format!("{:.2}", s_crit())}
                            </ResultRow>
                            <ResultRow label=move || t!(i18n, p_type_min_orbit)
                                hint=move || t!(i18n, hint_p_type_min)>
                                {move || format!("{:.2}", p_crit())}
                            </ResultRow>
                        })
                    } else {
                        None
                    }}

                    <SectionHeader label=move || t!(i18n, habitable_zone) />
                    <ResultRow label=move || t!(i18n, hz_inner_outer)
                        hint=move || t!(i18n, hint_hz)>
                        {hz_display}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, frost_line)
                        hint=move || t!(i18n, hint_frost_line)>
                        {frost_display}
                    </ResultRow>

                    <SectionHeader label=move || t!(i18n, system_boundaries) />
                    <ResultRow label=move || t!(i18n, inner_au)
                        hint=move || t!(i18n, hint_sys_inner)>
                        {sys_inner}
                    </ResultRow>
                    <ResultRow label=move || t!(i18n, outer_au)
                        hint=move || t!(i18n, hint_sys_outer)>
                        {sys_outer}
                    </ResultRow>

                    <SectionHeader label=move || t!(i18n, habitability) />
                    <BoolRow label=move || t!(i18n, optimal_star_mass) value=habitable
                        hint=move || t!(i18n, hint_optimal_star) />

                    <SectionHeader label=move || t!(i18n, flora_prediction) />
                    {move || match temp() {
                        Ok(t) => {
                            let sc = spectral_class(t);
                            let pw = peak_wavelength(t);
                            let pred = predict_flora_pigment(sc, pw);
                            let hex = pigment_display_color(pred.pigment);
                            Some(view! {
                                <div class="flex justify-between items-center gap-4 py-2.5 px-3
                                            border-b border-divider/30 rounded hover:bg-edge/20">
                                    <span class="text-label text-sm flex items-center gap-1">
                                        {t!(i18n, predicted_pigment)}
                                        <InfoHint text=move || t!(i18n, hint_predicted_pigment) />
                                    </span>
                                    <div class="flex items-center gap-2">
                                        <span
                                            class="w-4 h-4 rounded-full ring-1 ring-edge"
                                            style:background-color=hex
                                        />
                                        <span class="text-heading text-sm font-mono">
                                            {match pred.pigment {
                                                astro_lib::flora::FloraPigment::Green     => t_string!(i18n, pigment_green),
                                                astro_lib::flora::FloraPigment::Red       => t_string!(i18n, pigment_red),
                                                astro_lib::flora::FloraPigment::Orange    => t_string!(i18n, pigment_orange),
                                                astro_lib::flora::FloraPigment::Blue      => t_string!(i18n, pigment_blue),
                                                astro_lib::flora::FloraPigment::Black     => t_string!(i18n, pigment_black),
                                                astro_lib::flora::FloraPigment::UvHostile => t_string!(i18n, pigment_uv_hostile),
                                            }}
                                        </span>
                                    </div>
                                </div>
                                <ResultRow label=move || t!(i18n, absorbed_range)
                                    hint=move || t!(i18n, hint_absorbed_range)>
                                    {match sc {
                                        SpectralClass::O | SpectralClass::B => t_string!(i18n, flora_absorbed_ob),
                                        SpectralClass::A => t_string!(i18n, flora_absorbed_a),
                                        SpectralClass::F => t_string!(i18n, flora_absorbed_f),
                                        SpectralClass::G => t_string!(i18n, flora_absorbed_g),
                                        SpectralClass::K => t_string!(i18n, flora_absorbed_k),
                                        SpectralClass::M => t_string!(i18n, flora_absorbed_m),
                                    }}
                                </ResultRow>
                                <ResultRow label=move || t!(i18n, reflected_color)
                                    hint=move || t!(i18n, hint_reflected_color)>
                                    {match sc {
                                        SpectralClass::O | SpectralClass::B => t_string!(i18n, flora_reflected_ob),
                                        SpectralClass::A => t_string!(i18n, flora_reflected_a),
                                        SpectralClass::F => t_string!(i18n, flora_reflected_f),
                                        SpectralClass::G => t_string!(i18n, flora_reflected_g),
                                        SpectralClass::K => t_string!(i18n, flora_reflected_k),
                                        SpectralClass::M => t_string!(i18n, flora_reflected_m),
                                    }}
                                </ResultRow>
                                <p class="text-xs text-hint leading-relaxed px-3 py-2">
                                    {match sc {
                                        SpectralClass::O | SpectralClass::B => t_string!(i18n, flora_reasoning_ob),
                                        SpectralClass::A => t_string!(i18n, flora_reasoning_a),
                                        SpectralClass::F => t_string!(i18n, flora_reasoning_f),
                                        SpectralClass::G => t_string!(i18n, flora_reasoning_g),
                                        SpectralClass::K => t_string!(i18n, flora_reasoning_k),
                                        SpectralClass::M => t_string!(i18n, flora_reasoning_m),
                                    }}
                                </p>
                            })
                        }
                        Err(_) => None,
                    }}
                </div>
            </div>

            <CompareTable snapshots=snapshots />
        </div>
    }
}
