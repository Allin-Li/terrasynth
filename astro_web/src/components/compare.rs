use crate::i18n::*;
use leptos::prelude::*;

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub name: String,
    pub rows: Vec<(&'static str, String)>,
}

// ── Markdown export ───────────────────────────────────────────────────────────

pub fn to_markdown(snaps: &[Snapshot]) -> String {
    if snaps.is_empty() {
        return String::new();
    }
    let mut out = String::new();

    // Header row
    out.push_str("| Property |");
    for s in snaps {
        out.push_str(&format!(" {} |", s.name));
    }
    out.push('\n');

    // Separator
    out.push_str("| --- |");
    for _ in snaps {
        out.push_str(" --- |");
    }
    out.push('\n');

    // Data rows (use first snapshot's labels as row headers)
    if let Some(first) = snaps.first() {
        for (i, (label, _)) in first.rows.iter().enumerate() {
            out.push_str(&format!("| {} |", label));
            for snap in snaps {
                let val = snap.rows.get(i).map(|(_, v)| v.as_str()).unwrap_or("-");
                out.push_str(&format!(" {} |", val));
            }
            out.push('\n');
        }
    }
    out
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Renders a comparison table for a list of saved world snapshots.
/// Shows nothing when the list is empty.
#[component]
pub fn CompareTable(snapshots: RwSignal<Vec<Snapshot>>) -> impl IntoView {
    let i18n = use_i18n();
    let show_md = RwSignal::new(false);

    let md_text = Signal::derive(move || {
        let snaps = snapshots.get();
        to_markdown(&snaps)
    });

    view! {
        <div>
            // ── Compare table (visible only when there are snapshots) ─────────
            {move || {
                if snapshots.get().is_empty() {
                    None
                } else {
                    Some(view! {
                        <div class="bg-card/60 border border-edge rounded-2xl p-6">
                            // Header bar
                            <div class="flex items-center justify-between mb-4">
                                <div class="flex items-center gap-2">
                                    <div class="w-1.5 h-1.5 rounded-full bg-accent-alt" />
                                    <h2 class="text-xs font-semibold text-label uppercase tracking-widest">
                                        {t!(i18n, saved_worlds)}
                                    </h2>
                                </div>
                                <div class="flex gap-2">
                                    <button
                                        class="text-[11px] font-semibold px-3 py-1.5 rounded-lg
                                               cursor-pointer
                                               bg-accent/15 text-accent ring-1 ring-accent/20
                                               hover:bg-accent/25"
                                        on:click=move |_| show_md.set(true)
                                    >
                                        {t!(i18n, copy_markdown)}
                                    </button>
                                    <button
                                        class="text-[11px] font-semibold px-3 py-1.5 rounded-lg
                                               cursor-pointer
                                               bg-edge/40 text-hint ring-1 ring-edge
                                               hover:text-label"
                                        on:click=move |_| snapshots.set(vec![])
                                    >
                                        {t!(i18n, clear_all)}
                                    </button>
                                </div>
                            </div>

                            // Scrollable table
                            <div class="overflow-x-auto rounded-lg border border-edge/50">
                                <table class="w-full text-sm">
                                    // Column headers (one per snapshot)
                                    <thead>
                                        <tr class="bg-edge/20">
                                            <th class="text-left text-hint text-[11px]
                                                       uppercase tracking-wider font-semibold
                                                       py-3 px-4">
                                                {t!(i18n, property)}
                                            </th>
                                            {move || snapshots.get().into_iter().enumerate()
                                                .map(|(i, s)| {
                                                    let name = s.name.clone();
                                                    view! {
                                                        <th class="text-left text-[11px] font-semibold
                                                                   py-3 px-4 whitespace-nowrap">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-accent">{name}</span>
                                                                <button
                                                                    class="text-hint cursor-pointer
                                                                           hover:text-err
                                                                           hover:bg-err/10
                                                                           rounded p-0.5 -m-0.5"
                                                                    on:click=move |_| {
                                                                        snapshots.update(|v| {
                                                                            v.remove(i);
                                                                        })
                                                                    }
                                                                >
                                                                    "✕"
                                                                </button>
                                                            </div>
                                                        </th>
                                                    }
                                                })
                                                .collect_view()
                                            }
                                        </tr>
                                    </thead>

                                    // Data rows
                                    <tbody>
                                        {move || {
                                            let snaps = snapshots.get();
                                            let n_rows = snaps
                                                .first()
                                                .map(|s| s.rows.len())
                                                .unwrap_or(0);
                                            (0..n_rows).map(|row_i| {
                                                let label = snaps[0].rows[row_i].0;
                                                let vals: Vec<String> = snaps
                                                    .iter()
                                                    .map(|s| {
                                                        s.rows
                                                            .get(row_i)
                                                            .map(|(_, v)| v.clone())
                                                            .unwrap_or_default()
                                                    })
                                                    .collect();
                                                view! {
                                                    <tr class="border-t border-edge/30
                                                               hover:bg-edge/10">
                                                        <td class="text-label py-2.5 px-4
                                                                   whitespace-nowrap text-sm">
                                                            {label}
                                                        </td>
                                                        {vals.into_iter()
                                                            .map(|v| view! {
                                                                <td class="text-heading font-mono
                                                                           tabular-nums
                                                                           py-2.5 px-4
                                                                           whitespace-nowrap text-sm">
                                                                    {v}
                                                                </td>
                                                            })
                                                            .collect_view()
                                                        }
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    })
                }
            }}

            // ── Markdown modal (fixed overlay) ───────────────────────────────
            <Show when=move || show_md.get()>
                <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50
                            flex items-center justify-center"
                     on:click=move |_| show_md.set(false)>
                    <div class="bg-card border border-edge rounded-2xl p-6
                                w-full max-w-2xl mx-4 shadow-2xl shadow-black/50"
                         on:click=move |ev| ev.stop_propagation()>
                        <div class="flex justify-between items-center mb-4">
                            <h3 class="text-sm font-medium text-label">
                                {t!(i18n, md_modal_title)}
                            </h3>
                            <button
                                class="text-hint hover:text-heading cursor-pointer
                                       hover:bg-edge/40 rounded-lg
                                       w-7 h-7 flex items-center justify-center"
                                on:click=move |_| show_md.set(false)
                            >
                                "✕"
                            </button>
                        </div>
                        <textarea
                            class="w-full h-64 bg-inset border border-edge
                                   rounded-lg p-4 text-label text-xs font-mono
                                   outline-none resize-none
                                   focus:ring-1 focus:ring-accent/40 focus:border-accent"
                            readonly
                            prop:value=move || md_text.get()
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
}
