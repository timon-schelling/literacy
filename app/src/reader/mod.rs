use std::usize;

use audio::Track;
use common::{Segment, Wav};
use leptos::prelude::*;
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

    let text = include_str!("../../../assets/text.txt");
    let text = text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let word_number = text.len();
    let words_per_page = 100;
    let pages_number = (word_number / words_per_page) + 1;

    let audio_resource: LocalResource<Track> = LocalResource::new(move || {
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
    let text_offset = RwSignal::new(0);
    Effect::new(move || {
        let current_page = page.get();
        if current_page >= pages_number {
            page.set(pages_number - 1);
        }
        text_offset.set(current_page * words_per_page);
    });

    let text = RwSignal::new(text);

    let content: RwSignal<Vec<String>> = RwSignal::new(vec![]);

    Effect::new(move || {
        let offset = text_offset.get();
        let new = text
            .get()
            .into_iter()
            .skip(offset)
            .take(words_per_page)
            .to_owned()
            .collect::<Vec<String>>();
        content.set(new);
    });

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
    } = use_interval_with_options(350, UseIntervalOptions::default().immediate(false));
    let counter_pause_clone = counter_pause.clone();
    Effect::new(move || {
        counter.get();
        let mut content_length = content_length.get();
        if content_length > 0 {
            content_length -= 1;
        }
        if progress.get_untracked().unwrap_or(0) + 1 > content_length && content_length > 0 {
            if page.get_untracked() >= pages_number - 1 {
                counter_pause_clone();
                progress_write.set(None);
                return;
            }
            page.update(|n| {
                if *n < usize::MAX {
                    *n += 1
                }
            });
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
        progress_write.update(move |o| *o = None);
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

    Effect::new(move || match (playing.get(), audio.get_untracked()) {
        (true, Some(a)) => a.play(),
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
