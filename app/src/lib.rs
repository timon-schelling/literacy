use leptos::{logging, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_mview::mview;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos_use::{use_interval, UseIntervalReturn};
use lipsum::lipsum;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    mview! {
        {view!{<!DOCTYPE html>}}
        Root options={options.clone()};
    }
}

#[component]
pub fn Root(options: LeptosOptions) -> impl IntoView {
    mview! {
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                AutoReload options={options.clone()};
                HydrationScripts options={options.clone()};
                MetaTags;
            }
            body {
                App;
            }
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    mview! {
        Stylesheet href="/pkg/app.css";
        Title text="Literacy";
        Router {
            main {
                Routes fallback={|| "Page not found.".into_view()} {
                    Route path={StaticSegment("")} view={ReadingPage};
                }
            }
        }
    }
}

#[component]
fn ReadingPage() -> impl IntoView {
    mview! {
        Reader;
    }
}

#[component]
fn Reader() -> impl IntoView {
    let text_offset = RwSignal::new(0u32);

    let text_length = 100u32;

    let text_gen = move |length: u32| {
        lipsum(length.try_into().unwrap_or(text_length as usize))
    };
    let text = text_gen(text_length);
    let text = text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let (content, set_content) = signal(text);

    let UseIntervalReturn { counter: new_text, .. } = use_interval(3000);
    Effect::new(move || {
        new_text.get();
        let offset = text_offset.get_untracked();
        let new = text_gen(text_length + offset)
            .split_whitespace()
            .skip(offset as usize)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        logging::log!("{}", new.join(" "));
        set_content.set(new);
        text_offset.set(offset + 1);
    });


    let content_length = RwSignal::new(0u32);

    let progress_write = RwSignal::new(Some(0u32));
    let progress = progress_write.read_only();

    let UseIntervalReturn { counter, reset: counter_reset ,.. } = use_interval(550);
    Effect::new(move || {
        counter.get();
        if progress.get_untracked().unwrap_or(0) > content_length.get() {
            return;
        }
        progress_write.set(Some(progress.get_untracked().unwrap_or(0) + 1));
    });

    let content_clone = content.clone();
    Effect::new(move || {
        content_length.set(content_clone.get().len().try_into().unwrap_or(0));
    });
    let content_clone = content.clone();
    Effect::new(move || {
        content_clone.get();
        progress_write.set(None);
        counter_reset();
    });

    mview! {
        div.reader {
            Controls progress={progress_write} content_length={content_length.read_only()};
            div.pages {
                Page content={content} highlight={progress};
            }
        }
    }
}

#[component]
fn Controls(progress: RwSignal<Option<u32>>, content_length: ReadSignal<u32>) -> impl IntoView {
    let internal = RwSignal::new("".to_string());
    Effect::new(move || internal.set(progress.get().unwrap_or(0).to_string()));
    Effect::new(move || {
        let new = internal.get().parse().unwrap_or(0);
        if new == progress.get().unwrap_or(0) {
            return;
        }
        progress.set(Some(new))
    });

    mview! {
        div.controls {
            div.progress {
                input
                    type="range"
                    min="0"
                    max={content_length}
                    bind:value={internal};
            }
        }
    }
}

#[component]
fn Page(content: ReadSignal<Vec<String>>, highlight: ReadSignal<Option<u32>>) -> impl IntoView {
    mview! {
        div.page {
            Paragraph text={content()} {highlight};
        }
    }
}

#[component]
fn Paragraph(text: Vec<String>, highlight: ReadSignal<Option<u32>>) -> impl IntoView {
    mview! {
        p.paragraph {
            {
                move || text
                    .iter()
                    .enumerate()
                    .map(|(i, w)| {
                        let is_highlighted = highlight.get().map_or(false, move |h| h == i as u32);
                        mview! {
                            Word
                                text={w.to_string()}
                                highlight={is_highlighted};
                        }
                    }).collect_view()
            }
        }
    }
}

#[component]
fn Word(text: String, highlight: bool) -> impl IntoView {
    mview! {
        span.word class:highlight={move || highlight} {
            { text }
        }
        span {
            ""
        }
    }
}
