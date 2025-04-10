use leptos::prelude::*;
use leptos_mview::mview;

#[component]
pub(crate) fn Segment(
    #[prop(into)] text: Signal<Vec<String>>,
    #[prop(into)] highlight: Signal<Option<u32>>,
    #[prop(into)] active: Signal<bool>,
) -> impl IntoView {
    mview! {
        span.segment class:active={active} {
        {
            move || text.get()
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let is_active = if active.get() {
                        highlight.get().map_or(false, move |h| h == i as u32)
                    } else {
                        false
                    };
                    mview! {
                        Word
                            text={w.to_string()}
                            active={is_active};
                    }
                }).collect_view()
        }
        }
    }
}

#[component]
fn Word(#[prop(into)] text: String, active: bool) -> impl IntoView {
    mview! {
        span.word class:active={move || active} {
            { text }
        }
        span {
            " "
        }
    }
}
