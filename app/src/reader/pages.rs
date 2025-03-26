use leptos::prelude::*;
use leptos_mview::mview;

#[component]
pub(crate) fn Pages(
    #[prop(into)] content: ReadSignal<Vec<String>>,
    #[prop(into)] highlight: ReadSignal<Option<u32>>,
) -> impl IntoView {
    mview! {
        div.pages {
            Page {content} {highlight};
        }
    }
}

#[component]
fn Page(
    #[prop(into)] content: Signal<Vec<String>>,
    #[prop(into)] highlight: Signal<Option<u32>>,
) -> impl IntoView {
    let text = Memo::new(move |_| content.get());

    mview! {
        div.page {
            Paragraph {text} {highlight};
        }
    }
}

#[component]
fn Paragraph(
    #[prop(into)] text: Signal<Vec<String>>,
    #[prop(into)] highlight: Signal<Option<u32>>,
) -> impl IntoView {
    mview! {
        p.paragraph {
            {
                move || text.get()
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
fn Word(#[prop(into)] text: String, highlight: bool) -> impl IntoView {
    mview! {
        span.word class:highlight={move || highlight} {
            { text }
        }
        span {
            " "
        }
    }
}
