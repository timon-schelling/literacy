use std::usize;

use audio::Track;
use common::{Segment, Text, Wav, Word};
use leptos::{logging, prelude::*};
use leptos_mview::mview;
use leptos_use::{UseIntervalReturn, use_interval};

mod helper;

mod audio;

mod controls;
mod pages;

#[component]
pub(crate) fn Reader() -> impl IntoView {
    let text: RwSignal<Option<Text>> = RwSignal::new(None);
    let segment_index = RwSignal::new(0);
    let segment: RwSignal<Option<Segment>> = RwSignal::new(None);
    let words: RwSignal<Vec<Word>> = RwSignal::new(vec![]);
    let content: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let content_length = RwSignal::new(0u32);
    let progress: RwSignal<Option<u32>> = RwSignal::new(None);
    let progress_from_audio: RwSignal<Option<u32>> = RwSignal::new(None);
    let audio: RwSignal<Option<Track>> = RwSignal::new(None);
    let audio_progress: RwSignal<Option<f64>> = RwSignal::new(None);
    let playing = RwSignal::new(false);

    let text_resource: LocalResource<Text> = LocalResource::new(async move || {
        serde_json::from_str(helper::load_text("./text.json").await.as_str())
            .expect("expected segment as json")
    });
    let audio_resource = LocalResource::new(move || {
        (async move |segment: Option<Segment>| {
            if let Some(segment) = segment {
                let bytes = match segment.audio {
                    common::Audio::None => vec![],
                    common::Audio::Wav(Wav::Raw(bytes)) => bytes,
                    common::Audio::Wav(_) => todo!(),
                    common::Audio::Ref(url) => helper::load_bytes(&url).await,
                };
                Some(Track::new(&bytes).await)
            } else {
                None
            }
        })(segment.get())
    });

    let UseIntervalReturn {
        counter: update_tick,
        reset: update_tick_reset,
        ..
    } = use_interval(10);

    // load segment from text
    Effect::new(move || {
        if let Some(t) = text_resource.get() {
            text.set(Some(t.take()));
        }
    });

    // load segment from text
    Effect::new(move || {
        if let Some(t) = text.get() {
            segment.set(t.segments.get(segment_index.get()).cloned());
        }
    });

    // load words from segment
    Effect::new(move || {
        if let Some(s) = segment.get() {
            words.set(s.words);
        }
    });

    // collect content from words
    Effect::new(move || {
        content.set(words.get().iter().map(|w| w.content.clone()).collect());
    });

    // load audio from audio resource
    Effect::new(move || {
        if let Some(r) = audio_resource.get()
            && let Some(t) = r.take()
        {
            audio.set(Some(t));
        }
    });

    // prevent update tick overflow
    Effect::new(move || {
        if update_tick.get() >= u16::MAX as u64 {
            logging::log!("reset update_tick");
            update_tick_reset();
        }
    });

    // update audio progress from playing audio
    Effect::new(move || {
        update_tick.get();
        if let Some(audio) = audio.get_untracked() {
            if let Some(ap) = audio.progress() {
                audio_progress.set(Some(ap));
            }
        }
    });

    // update progress on audio progress change
    Effect::new(move || {
        if let Some(ap) = audio_progress.get() {
            let new_progress = words.get().iter().enumerate().find_map(|e| {
                if e.1.start <= ap && e.1.end >= ap {
                    Some(e.0 as u32)
                } else {
                    None
                }
            });
            if new_progress != progress.get_untracked() && new_progress.is_some() {
                progress_from_audio.set(new_progress);
                progress.set(new_progress);
            }
        }
    });

    // pause on manual progress change
    Effect::new(move || {
        if progress_from_audio.get_untracked() != progress.get() && progress.get().is_some() {
            playing.set(false);
        }
    });

    // pause on manual page change
    Effect::new(move || {
        segment_index.get();
        if audio_progress.get_untracked().is_some() {
            playing.set(false);
        }
    });

    // pause at the end of text
    Effect::new(move || {
        if let Some(t) = text.get()
            && segment_index.get() >= t.segments.len()
        {
            segment_index.set(t.segments.len() - 1);
            playing.set(false);
        }
    });

    // automatically go to the next page
    Effect::new(move || {
        if let Some(ap) = audio_progress.get()
            && let Some(last_word) = words.get_untracked().last()
            && ap >= last_word.end + 0.3
        {
            audio.set(None);
            audio_progress.set(None);
            segment_index.update(|n| {
                if *n < usize::MAX {
                    *n += 1
                }
            });
        }
    });

    // calculate content length
    let content_clone = content.clone();
    Effect::new(move || {
        content_length.set(content_clone.get().len().try_into().unwrap_or(0));
    });

    // reset progress on content change
    let content_clone = content.clone();
    Effect::new(move || {
        content_clone.get();
        progress.update(move |o| *o = None);
    });

    // play or pause audio depending on playing state
    Effect::new(move || match (playing.get(), audio.get()) {
        (true, Some(a)) => {
            if let Some(p) = progress.get_untracked()
                && let Some(w) = words.get_untracked().get(p as usize)
            {
                a.play_at(w.start);
            } else {
                a.play();
            }
        }
        (false, Some(a)) => a.pause(),
        _ => {}
    });

    mview! {
        div.reader {
            controls::Controls page={segment_index} {playing} {progress} content_length={content_length.read_only()};
            pages::Pages content={content.read_only()} highlight={progress.read_only()};
        }
    }
}
