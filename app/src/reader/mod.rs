use std::usize;

use audio::Track;
use common::{Segment, Wav, Word};
use leptos::{logging, prelude::*};
use leptos_mview::mview;
use leptos_use::{UseIntervalOptions, UseIntervalReturn, use_interval_with_options};

mod audio;
mod controls;
mod pages;

mod helper {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        Response,
        js_sys::{JsString, Uint8Array},
        window,
    };

    async fn request(url: &str) -> Response {
        let response = JsFuture::from(window().expect("request failed").fetch_with_str(url))
            .await
            .expect("request failed");
        response.dyn_into().expect("request failed")
    }

    pub(super) async fn load_bytes(url: &str) -> Vec<u8> {
        let response = request(url).await;
        let buffer = JsFuture::from(response.array_buffer().expect("loading bytes failed"))
            .await
            .expect("loading bytes failed");
        let u8_array = Uint8Array::new(&buffer);
        u8_array.to_vec()
    }

    pub(super) async fn load_text(url: &str) -> String {
        let response = request(url).await;
        let text = JsFuture::from(response.text().expect("loading bytes text"))
            .await
            .expect("loading bytes text")
            .dyn_into::<JsString>()
            .expect("loading bytes text");
        text.into()
    }
}

#[component]
pub(crate) fn Reader() -> impl IntoView {
    let segment: LocalResource<Segment> = LocalResource::new(async move || {
        serde_json::from_str(helper::load_text("./segment.json").await.as_str()).expect("expected segment as json")
    });

    let words_resource = LocalResource::new(move || {
        segment.get();
        (async move |segment: LocalResource<Segment>| {
            segment.await
        })(segment)
    });
    let words: RwSignal<Vec<Word>> = RwSignal::new(vec![]);
    Effect::new(move || {
        if let Some(r) = words_resource.get() {
            words.set(r.take().words);
        }
    });
    let content: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    Effect::new(move || {
        content.set(words.get().iter().map(|w| w.content.clone()).collect());
    });

    let audio_resource = LocalResource::new(move || {
        segment.get();
        (async move |segment: LocalResource<Segment>| {
            let bytes = match segment.await.audio {
                common::Audio::None => vec![],
                common::Audio::Wav(Wav::Raw(bytes)) => bytes,
                common::Audio::Wav(_) => todo!(),
                common::Audio::Ref(url) => helper::load_bytes(&url).await,
            };
            Track::new(&bytes).await
        })(segment)
    });
    let audio: RwSignal<Option<Track>> = RwSignal::new(None);
    Effect::new(move || {
        if let Some(r) = audio_resource.get() {
            audio.set(Some(r.take()));
        }
    });

    let page = RwSignal::new(0);

    let content_length = RwSignal::new(0u32);

    let audio_progress: RwSignal<Option<f64>> = RwSignal::new(None);
    let UseIntervalReturn {
        counter,
        reset: counter_reset,
        ..
    } = use_interval_with_options(10, UseIntervalOptions::default().immediate(true));
    Effect::new(move || {
        if counter.get() >= u16::MAX as u64 {
            logging::log!("reset counter");
            counter_reset();
        }
        if let Some(audio) = audio.get_untracked() {
            if let Some(ap) = audio.progress() {
                audio_progress.set(Some(ap));
            }
        }
    });

    let progress_write: RwSignal<Option<u32>> = RwSignal::new(None);
    let progress_from_audio: RwSignal<Option<u32>> = RwSignal::new(None);
    let progress = progress_write.read_only();
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
                progress_write.set(new_progress);
            }
        }
    });

    let content_clone = content.clone();
    Effect::new(move || {
        content_length.set(content_clone.get().len().try_into().unwrap_or(0));
    });
    let content_clone = content.clone();
    Effect::new(move || {
        content_clone.get();
        progress_write.update(move |o| *o = None);
    });

    let playing = RwSignal::new(false);
    Effect::new(move || {
        if progress_from_audio.get_untracked() != progress.get() {
            playing.set(false);
        }
    });
    // Effect::new(move || match (playing.get(), counter_is_active.get()) {
    //     (true, false) => counter_resume(),
    //     (false, true) => counter_pause(),
    //     _ => {}
    // });
    // Effect::new(move || {
    //     if progress.get().is_none() {
    //         playing.set(false)
    //     }
    // });

    Effect::new(move || match (playing.get(), audio.get_untracked()) {
        (true, Some(a)) => {
            if let Some(p) = progress.get_untracked() && let Some(w) = words.get_untracked().get(p as usize) {
                a.play_at(w.start);
            } else {
                a.play();
            }
        },
        (false, Some(a)) => a.pause(),
        _ => {}
    });

    mview! {
        div.reader {
            controls::Controls {page} {playing} progress={progress_write} content_length={content_length.read_only()};
            pages::Pages content={content.read_only()} highlight={progress};
        }
    }
}
