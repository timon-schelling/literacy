use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos_mview::mview;
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

        Title text="Welcome to Leptos";

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
    let UseIntervalReturn { counter, .. } = use_interval(550);

    mview! {
        div.reader {
            div.page {
                Paragraph text={lipsum(300)} highlight={Some(counter)};
            }
        }
    }
}

#[component]
fn Paragraph(text: String, highlight: Option<Signal<u64>>) -> impl IntoView {
    mview! {
        p.paragraph {
            {
                move || text
                    .split_whitespace()
                    .enumerate()
                    .map(|(i, w)| {
                        let is_highlighted = highlight.as_ref().map_or(false, move |h| h.get() == i as u64);
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
