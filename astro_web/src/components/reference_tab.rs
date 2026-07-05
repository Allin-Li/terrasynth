use crate::i18n::*;
use leptos::prelude::*;

// ─── Static illustrations ───────────────────────────────────────────────────
// Each diagram is a self-contained SVG string injected via inner_html, in the
// same style as the climate disc on the planet tab. No text except universal
// symbols and formula fragments, so they need no localization.

const SVG_SPECTRAL: &str = "<svg viewBox='0 0 300 52' width='300' height='52' style='max-width:100%;height:auto' role='img'>\
<rect x='1' y='6' width='40' height='26' rx='4' fill='#9db4ff'/>\
<rect x='43' y='6' width='40' height='26' rx='4' fill='#aabfff'/>\
<rect x='85' y='6' width='40' height='26' rx='4' fill='#cad8ff'/>\
<rect x='127' y='6' width='40' height='26' rx='4' fill='#f6f5ff'/>\
<rect x='169' y='6' width='40' height='26' rx='4' fill='#ffe9c4'/>\
<rect x='211' y='6' width='40' height='26' rx='4' fill='#ffcf9e'/>\
<rect x='253' y='6' width='40' height='26' rx='4' fill='#ff9d66'/>\
<text x='21' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>O</text>\
<text x='63' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>B</text>\
<text x='105' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>A</text>\
<text x='147' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>F</text>\
<text x='189' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>G</text>\
<text x='231' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>K</text>\
<text x='273' y='24' text-anchor='middle' font-size='12' font-weight='600' fill='#1e293b'>M</text>\
<text x='1' y='48' font-size='9' fill='#94a3b8'>&gt; 30 000 K</text>\
<text x='293' y='48' text-anchor='end' font-size='9' fill='#94a3b8'>&lt; 3 800 K</text>\
</svg>";

const SVG_HZ: &str = "<svg viewBox='0 0 300 70' width='300' height='70' style='max-width:100%;height:auto' role='img'>\
<line x1='10' y1='40' x2='292' y2='40' stroke='rgba(148,163,184,0.35)' stroke-width='1'/>\
<circle cx='18' cy='40' r='12' fill='rgba(251,191,36,0.25)'/>\
<circle cx='18' cy='40' r='8' fill='#fbbf24'/>\
<rect x='95' y='26' width='60' height='28' rx='6' fill='rgba(46,204,113,0.22)' stroke='#2ecc71' stroke-width='1'/>\
<circle cx='125' cy='40' r='4' fill='#7fb3d5'/>\
<line x1='230' y1='22' x2='230' y2='58' stroke='#7fb3d5' stroke-width='1.5' stroke-dasharray='3 3'/>\
<text x='230' y='16' text-anchor='middle' font-size='10' fill='#7fb3d5'>&#10052;</text>\
<text x='95' y='66' text-anchor='middle' font-size='8' fill='#94a3b8'>0.95&#183;&#8730;L</text>\
<text x='155' y='66' text-anchor='middle' font-size='8' fill='#94a3b8'>1.37&#183;&#8730;L</text>\
<text x='230' y='66' text-anchor='middle' font-size='8' fill='#94a3b8'>4.85&#183;&#8730;L</text>\
</svg>";

const SVG_BINARY: &str = "<svg viewBox='0 0 300 110' width='300' height='110' style='max-width:100%;height:auto' role='img'>\
<line x1='150' y1='14' x2='150' y2='96' stroke='rgba(148,163,184,0.15)' stroke-width='1'/>\
<text x='75' y='16' text-anchor='middle' font-size='10' font-weight='600' fill='#94a3b8'>S</text>\
<circle cx='60' cy='60' r='7' fill='#fbbf24'/>\
<circle cx='115' cy='60' r='5' fill='#f59e0b'/>\
<circle cx='60' cy='60' r='18' fill='none' stroke='rgba(148,163,184,0.6)' stroke-width='1' stroke-dasharray='3 3'/>\
<circle cx='73' cy='47' r='3' fill='#7fb3d5'/>\
<text x='225' y='16' text-anchor='middle' font-size='10' font-weight='600' fill='#94a3b8'>P</text>\
<circle cx='213' cy='60' r='6' fill='#fbbf24'/>\
<circle cx='237' cy='60' r='5' fill='#f59e0b'/>\
<circle cx='225' cy='60' r='38' fill='none' stroke='rgba(148,163,184,0.6)' stroke-width='1' stroke-dasharray='3 3'/>\
<circle cx='198' cy='33' r='3' fill='#7fb3d5'/>\
</svg>";

const SVG_SIZES: &str = "<svg viewBox='0 0 300 100' width='300' height='100' style='max-width:100%;height:auto' role='img'>\
<line x1='10' y1='70' x2='290' y2='70' stroke='rgba(148,163,184,0.2)' stroke-width='1'/>\
<circle cx='40' cy='64' r='6' fill='#c58f6d'/>\
<circle cx='105' cy='58' r='12' fill='#7fb3d5'/>\
<circle cx='185' cy='46' r='24' fill='#e8b26a'/>\
<circle cx='262' cy='42' r='28' fill='#d98a52'/>\
<text x='40' y='86' text-anchor='middle' font-size='8' fill='#94a3b8'>&#8804; 2 M&#8853;</text>\
<text x='105' y='86' text-anchor='middle' font-size='8' fill='#94a3b8'>2&#8211;10</text>\
<text x='185' y='86' text-anchor='middle' font-size='8' fill='#94a3b8'>10&#8211;300</text>\
<text x='262' y='86' text-anchor='middle' font-size='8' fill='#94a3b8'>&gt; 300</text>\
</svg>";

const SVG_ELLIPSE: &str = "<svg viewBox='0 0 300 110' width='300' height='110' style='max-width:100%;height:auto' role='img'>\
<ellipse cx='150' cy='55' rx='120' ry='42' fill='none' stroke='rgba(148,163,184,0.5)' stroke-width='1' stroke-dasharray='4 3'/>\
<circle cx='262' cy='55' r='12' fill='rgba(251,191,36,0.25)'/>\
<circle cx='262' cy='55' r='8' fill='#fbbf24'/>\
<circle cx='270' cy='55' r='4' fill='#e67e22'/>\
<circle cx='30' cy='55' r='4' fill='#7fb3d5'/>\
</svg>";

const SVG_TILT: &str = "<svg viewBox='0 0 300 120' width='300' height='120' style='max-width:100%;height:auto' role='img'>\
<circle cx='110' cy='60' r='44' fill='rgba(127,179,213,0.12)' stroke='rgba(148,163,184,0.5)' stroke-width='1.5'/>\
<g transform='rotate(-23.4 110 60)'>\
<line x1='110' y1='8' x2='110' y2='112' stroke='#94a3b8' stroke-width='1.5'/>\
<line x1='66' y1='60' x2='154' y2='60' stroke='rgba(148,163,184,0.7)' stroke-width='1'/>\
<line x1='70' y1='42.5' x2='150' y2='42.5' stroke='#e67e22' stroke-width='1' stroke-dasharray='2 2'/>\
<line x1='70' y1='77.5' x2='150' y2='77.5' stroke='#e67e22' stroke-width='1' stroke-dasharray='2 2'/>\
<line x1='93' y1='19.6' x2='127' y2='19.6' stroke='#7fb3d5' stroke-width='1' stroke-dasharray='2 2'/>\
<line x1='93' y1='100.4' x2='127' y2='100.4' stroke='#7fb3d5' stroke-width='1' stroke-dasharray='2 2'/>\
</g>\
<circle cx='292' cy='60' r='10' fill='#fbbf24'/>\
<line x1='282' y1='40' x2='178' y2='40' stroke='#fbbf24' stroke-width='2'/>\
<line x1='282' y1='60' x2='178' y2='60' stroke='#fbbf24' stroke-width='2'/>\
<line x1='282' y1='80' x2='178' y2='80' stroke='#fbbf24' stroke-width='2'/>\
<polygon points='170,40 179,35 179,45' fill='#fbbf24'/>\
<polygon points='170,60 179,55 179,65' fill='#fbbf24'/>\
<polygon points='170,80 179,75 179,85' fill='#fbbf24'/>\
</svg>";

const SVG_GREENHOUSE: &str = "<svg viewBox='0 0 300 110' width='300' height='110' style='max-width:100%;height:auto' role='img'>\
<rect x='0' y='92' width='300' height='18' fill='rgba(148,163,184,0.25)'/>\
<line x1='20' y1='28' x2='280' y2='28' stroke='rgba(127,179,213,0.6)' stroke-width='2' stroke-dasharray='5 4'/>\
<line x1='70' y1='8' x2='70' y2='84' stroke='#fbbf24' stroke-width='2.5'/>\
<polygon points='70,92 64,81 76,81' fill='#fbbf24'/>\
<line x1='150' y1='90' x2='150' y2='16' stroke='#e74c3c' stroke-width='2' stroke-dasharray='4 3'/>\
<polygon points='150,8 144,19 156,19' fill='#e74c3c'/>\
<line x1='210' y1='90' x2='210' y2='34' stroke='#e67e22' stroke-width='2' stroke-dasharray='4 3'/>\
<line x1='222' y1='34' x2='222' y2='82' stroke='#e67e22' stroke-width='2' stroke-dasharray='4 3'/>\
<polygon points='222,90 216,79 228,79' fill='#e67e22'/>\
</svg>";

const SVG_TIDAL: &str = "<svg viewBox='0 0 300 110' width='300' height='110' style='max-width:100%;height:auto' role='img'>\
<circle cx='150' cy='55' r='16' fill='rgba(251,191,36,0.25)'/>\
<circle cx='150' cy='55' r='11' fill='#fbbf24'/>\
<circle cx='150' cy='55' r='40' fill='none' stroke='rgba(148,163,184,0.5)' stroke-width='1' stroke-dasharray='4 3'/>\
<circle cx='190' cy='55' r='9' fill='#7fb3d5'/>\
<circle cx='183' cy='55' r='2.5' fill='#1e293b'/>\
<circle cx='110' cy='55' r='9' fill='#7fb3d5'/>\
<circle cx='117' cy='55' r='2.5' fill='#1e293b'/>\
<circle cx='150' cy='15' r='9' fill='#7fb3d5'/>\
<circle cx='150' cy='22' r='2.5' fill='#1e293b'/>\
</svg>";

const SVG_CLIMATE_MINI: &str = "<svg viewBox='0 0 120 120' width='120' height='120' style='max-width:100%;height:auto' role='img'>\
<defs><clipPath id='ref-climate-disc'><circle cx='60' cy='60' r='50'/></clipPath></defs>\
<g clip-path='url(#ref-climate-disc)'>\
<rect x='6' y='10' width='108' height='8' fill='#dfeefa'/>\
<rect x='6' y='18' width='108' height='10' fill='#7fb3d5'/>\
<rect x='6' y='28' width='108' height='14' fill='#16a085'/>\
<rect x='6' y='42' width='108' height='12' fill='#2ecc71'/>\
<rect x='6' y='54' width='108' height='12' fill='#e67e22'/>\
<rect x='6' y='66' width='108' height='12' fill='#2ecc71'/>\
<rect x='6' y='78' width='108' height='14' fill='#16a085'/>\
<rect x='6' y='92' width='108' height='10' fill='#7fb3d5'/>\
<rect x='6' y='102' width='108' height='8' fill='#dfeefa'/>\
</g>\
<circle cx='60' cy='60' r='50' fill='none' stroke='rgba(148,163,184,0.35)' stroke-width='1.5'/>\
<line x1='10' y1='60' x2='110' y2='60' stroke='rgba(15,23,42,0.6)' stroke-width='1' stroke-dasharray='3 3'/>\
</svg>";

const SVG_HILL: &str = "<svg viewBox='0 0 300 120' width='300' height='120' style='max-width:100%;height:auto' role='img'>\
<circle cx='150' cy='60' r='55' fill='none' stroke='rgba(148,163,184,0.5)' stroke-width='1' stroke-dasharray='4 3'/>\
<circle cx='150' cy='60' r='23' fill='none' stroke='rgba(46,204,113,0.25)' stroke-width='9'/>\
<circle cx='150' cy='60' r='27.5' fill='none' stroke='rgba(46,204,113,0.8)' stroke-width='1' stroke-dasharray='3 2'/>\
<circle cx='150' cy='60' r='18' fill='rgba(231,76,60,0.12)' stroke='#e74c3c' stroke-width='1' stroke-dasharray='3 2'/>\
<circle cx='150' cy='60' r='9' fill='#7fb3d5'/>\
<circle cx='167.6' cy='45.2' r='3' fill='#cbd5e1'/>\
</svg>";

const SVG_ANGULAR: &str = "<svg viewBox='0 0 240 90' width='240' height='90' style='max-width:100%;height:auto' role='img'>\
<circle cx='70' cy='42' r='23' fill='rgba(251,191,36,0.2)'/>\
<circle cx='70' cy='42' r='17' fill='#fbbf24'/>\
<text x='120' y='48' text-anchor='middle' font-size='16' fill='#94a3b8'>&#8776;</text>\
<circle cx='170' cy='42' r='17' fill='#cbd5e1'/>\
<circle cx='163' cy='36' r='3' fill='rgba(100,116,139,0.4)'/>\
<circle cx='175' cy='47' r='2.5' fill='rgba(100,116,139,0.4)'/>\
<circle cx='168' cy='50' r='1.8' fill='rgba(100,116,139,0.4)'/>\
<text x='70' y='78' text-anchor='middle' font-size='9' fill='#94a3b8'>&#8776; 0.5&#176;</text>\
<text x='170' y='78' text-anchor='middle' font-size='9' fill='#94a3b8'>&#8776; 0.5&#176;</text>\
</svg>";

// ─── Building blocks ────────────────────────────────────────────────────────

/// One collapsible glossary article: a header row that toggles the body.
/// The body text comes as a single localized string with '\n' between
/// paragraphs.
#[component]
fn Article(
    #[prop(into)] title: ViewFn,
    #[prop(into)] body: Signal<String>,
    #[prop(optional)] svg: Option<&'static str>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    view! {
        <div class="border-b border-divider/30 last:border-0">
            <button
                class="w-full flex items-center gap-2 py-2.5 px-3 text-left cursor-pointer
                       rounded hover:bg-edge/20"
                on:click=move |_| open.update(|v| *v = !*v)
            >
                <span class="text-[10px] text-hint w-3 shrink-0">
                    {move || if open.get() { "▾" } else { "▸" }}
                </span>
                <span class="text-label text-sm font-medium">{title.run()}</span>
            </button>
            <div class=move || {
                if open.get() { "px-8 pb-4 flex flex-col gap-3" } else { "hidden" }
            }>
                {svg.map(|s| view! { <div class="pt-1 overflow-x-auto" inner_html=s /> })}
                {move || {
                    let text = body.get();
                    text.split('\n')
                        .map(|p| view! {
                            <p class="text-[13px] leading-relaxed text-label">{p.to_string()}</p>
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

/// A card grouping the articles of one object (star / planet / moons).
#[component]
fn Group(
    icon: &'static str,
    #[prop(into)] title: ViewFn,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-card/60 border border-edge rounded-2xl p-6">
            <div class="flex items-center gap-2 mb-3">
                <span class="text-base text-accent">{icon}</span>
                <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                    {title.run()}
                </h2>
            </div>
            {children()}
        </div>
    }
}

/// Shorthand: an Article wired to a title key and a body key (plus an
/// optional illustration).
macro_rules! art {
    ($i18n:ident, $title:ident, $body:ident) => {
        view! {
            <Article
                title=move || t!($i18n, $title)
                body=Signal::derive(move || t_string!($i18n, $body).to_string())
            />
        }
    };
    ($i18n:ident, $title:ident, $body:ident, $svg:expr) => {
        view! {
            <Article
                title=move || t!($i18n, $title)
                body=Signal::derive(move || t_string!($i18n, $body).to_string())
                svg=$svg
            />
        }
    };
}

// ─── Tab ────────────────────────────────────────────────────────────────────

#[component]
pub fn ReferenceTab() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex flex-col gap-6">
            <p class="text-sm text-hint leading-relaxed max-w-3xl">
                {t!(i18n, ref_intro)}
            </p>

            <Group icon="★" title=move || t!(i18n, tab_star)>
                {art!(i18n, ref_star_mass, ref_star_mass_body)}
                {art!(i18n, ref_luminosity, ref_luminosity_body)}
                {art!(i18n, ref_spectral, ref_spectral_body, SVG_SPECTRAL)}
                {art!(i18n, ref_star_radius, ref_star_radius_body)}
                {art!(i18n, ref_lifetime, ref_lifetime_body)}
                {art!(i18n, ref_peak_wl, ref_peak_wl_body)}
                {art!(i18n, ref_hz, ref_hz_body, SVG_HZ)}
                {art!(i18n, ref_frost, ref_frost_body)}
                {art!(i18n, ref_bounds, ref_bounds_body)}
                {art!(i18n, ref_star_hab, ref_star_hab_body)}
                {art!(i18n, ref_binary, ref_binary_body, SVG_BINARY)}
                {art!(i18n, ref_flora, ref_flora_body)}
            </Group>

            <Group icon="◉" title=move || t!(i18n, tab_planet)>
                {art!(i18n, ref_ptype, ref_ptype_body, SVG_SIZES)}
                {art!(i18n, ref_gravity, ref_gravity_body)}
                {art!(i18n, ref_escape, ref_escape_body)}
                {art!(i18n, ref_orbit_a, ref_orbit_a_body)}
                {art!(i18n, ref_ecc, ref_ecc_body, SVG_ELLIPSE)}
                {art!(i18n, ref_peri, ref_peri_body)}
                {art!(i18n, ref_tilt, ref_tilt_body, SVG_TILT)}
                {art!(i18n, ref_temp, ref_temp_body, SVG_GREENHOUSE)}
                {art!(i18n, ref_atmo, ref_atmo_body)}
                {art!(i18n, ref_tidal, ref_tidal_body, SVG_TIDAL)}
                {art!(i18n, ref_climate, ref_climate_body, SVG_CLIMATE_MINI)}
                {art!(i18n, ref_age, ref_age_body)}
            </Group>

            <Group icon="☽" title=move || t!(i18n, tab_moon)>
                {art!(i18n, ref_hill, ref_hill_body, SVG_HILL)}
                {art!(i18n, ref_roche, ref_roche_body)}
                {art!(i18n, ref_moon_props, ref_moon_props_body)}
                {art!(i18n, ref_moon_sky, ref_moon_sky_body, SVG_ANGULAR)}
                {art!(i18n, ref_moon_period, ref_moon_period_body)}
                {art!(i18n, ref_moon_multi, ref_moon_multi_body)}
            </Group>
        </div>
    }
}
