use leptos::prelude::*;
use leptos_mview::mview;

#[component]
pub(crate) fn Pages(
    content: ReadSignal<Vec<String>>,
    highlight: ReadSignal<Option<u32>>,
) -> impl IntoView {
    mview! {
        div.pages {
            Page {content} {highlight};
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
