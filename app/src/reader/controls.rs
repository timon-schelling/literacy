use icondata as icons;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_mview::mview;

#[component]
pub(crate) fn Controls(
    progress: RwSignal<Option<u32>>,
    playing: RwSignal<bool>,
    content_length: ReadSignal<u32>,
) -> impl IntoView {
    mview! {
        div.controls {
            Progress {progress} {content_length};
            Bar {playing};
        }
    }
}

#[component]
fn Progress(progress: RwSignal<Option<u32>>, content_length: ReadSignal<u32>) -> impl IntoView {
    let internal = RwSignal::new("".to_string());
    Effect::new(move || internal.set(progress.get().unwrap_or(0).to_string()));
    Effect::new(move || {
        let new = internal.get().parse().unwrap_or(0);
        if new == progress.get().unwrap_or(0) {
            return;
        }
        progress.set(Some(new))
    });

    let range = move || (0..content_length.get()).into_iter();

    mview! {
        div.progress {
            input
                type="range"
                min={move || range().min()}
                max={move || range().max()}
                bind:value={internal};
        }
    }
}

#[component]
fn Bar(playing: RwSignal<bool>) -> impl IntoView {
    mview! {
        div.bar {
            button {
                Icon icon={icons::FaBackwardStepSolid};
            }
            PlayPauseButton {playing};
            button {
                Icon icon={icons::FaForwardStepSolid};
            }
        }
    }
}

#[component]
fn PlayPauseButton(playing: RwSignal<bool>) -> impl IntoView {
    mview! {
        button on:click={move |_| playing.set(!playing.get_untracked())} {
            PlayPauseIcon {playing};
        }
    }
}

#[component]
fn PlayPauseIcon(#[prop(into)] playing: Signal<bool>) -> impl IntoView {
    let icon = RwSignal::new(icons::FaPlaySolid);
    Effect::new(move || {
        icon.set(if playing.get() {
            icons::FaPauseSolid
        } else {
            icons::FaPlaySolid
        });
    });

    mview! {
        Icon {icon};
    }
}
