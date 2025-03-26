use leptos::prelude::*;
use leptos_mview::mview;
use leptos_use::{UseIntervalOptions, UseIntervalReturn, use_interval_with_options};
use lipsum::lipsum;

mod controls;
mod pages;

#[component]
pub(crate) fn Reader() -> impl IntoView {
    let text_offset = RwSignal::new(0u32);

    let text_length = 100u32;

    let text_gen = move |length: u32| lipsum(length.try_into().unwrap_or(text_length as usize));
    let text = text_gen(text_length);
    let text = text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let (content, set_content) = signal(text);

    // let UseIntervalReturn {
    //     counter: new_text, ..
    // } = use_interval(3000);
    // Effect::new(move || {
    //     new_text.get();
    //     let offset = text_offset.get_untracked();
    //     let new = text_gen(text_length + offset)
    //         .split_whitespace()
    //         .skip(offset as usize)
    //         .map(|s| s.to_string())
    //         .collect::<Vec<String>>();
    //     logging::log!("{}", new.join(" "));
    //     set_content.set(new);
    //     text_offset.set(offset + 1);
    // });

    let content_length = RwSignal::new(0u32);

    let progress_write = RwSignal::new(None);
    let progress = progress_write.read_only();

    let UseIntervalReturn {
        counter,
        reset: counter_reset,
        pause: counter_pause,
        resume: counter_resume,
        is_active: counter_is_active,
        ..
    } = use_interval_with_options(550, UseIntervalOptions::default().immediate(false));
    let counter_pause_clone = counter_pause.clone();
    Effect::new(move || {
        counter.get();
        let mut content_length = content_length.get();
        if content_length > 0 {
            content_length -= 1;
        }
        if progress.get_untracked().unwrap_or(0) + 1 > content_length {
            counter_pause_clone();
            progress_write.set(None);
            return;
        }
        if !counter_is_active.get() {
            return;
        }
        progress_write.set(Some(progress.get_untracked().map_or(0, |c| c + 1)));
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

    let playing = RwSignal::new(false);
    Effect::new(move || match (playing.get(), counter_is_active.get()) {
        (true, false) => counter_resume(),
        (false, true) => counter_pause(),
        _ => {}
    });
    Effect::new(move || {
        if progress.get().is_none() {
            playing.set(false)
        }
    });

    mview! {
        div.reader {
            controls::Controls progress={progress_write} {playing} content_length={content_length.read_only()};
            pages::Pages {content} highlight={progress};
        }
    }
}
