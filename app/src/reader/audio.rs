use leptos::logging;
use rand::RngCore;
use std::cell::RefCell;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Uint8Array;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext};

thread_local! {
    static AUDIO_PLAYER_INTERNAL: RefCell<AudioPlayer> = {
        RefCell::new(AudioPlayer::new())
    };
}

#[derive(Debug)]
struct AudioPlayer {
    context: AudioContext,
    tracks: Vec<TrackInternal>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
            context: AudioContext::new().expect("audio not supported"),
            tracks: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct TrackInternal {
    id: [u8; 32],
    source: AudioBuffer,
    node: Option<AudioBufferSourceNode>,
    offset: f64,
    start_time: f64,
    is_playing: bool,
}

impl TrackInternal {
    fn new(id: [u8; 32], source: AudioBuffer) -> Self {
        TrackInternal {
            id,
            node: None,
            source,
            offset: 0.0,
            start_time: 0.0,
            is_playing: false,
        }
    }
}

#[derive(Clone)]
pub struct Track {
    inner: Arc<TrackInner>,
}

impl Track {
    pub async fn new(source: &[u8]) -> Self {
        Self { inner: Arc::new(TrackInner::new(source).await) }
    }

    pub fn prepare(&self) {
        self.inner.prepare();
    }

    pub fn play(&self) {
        self.inner.play();
    }

    pub fn pause(&self) {
        self.inner.pause();
    }

    pub fn stop(&self) {
        self.inner.stop();
    }
}

struct TrackInner {
    id: [u8; 32]
}

impl TrackInner {
    async fn new(source: &[u8]) -> Self {
        let uint8_array = Uint8Array::from(source);
        let promise = AUDIO_PLAYER_INTERNAL
            .with_borrow_mut(|p| p.context.decode_audio_data(&uint8_array.buffer()))
            .expect("failed to decode");
        let buffer = JsFuture::from(promise).await.expect("failed to decode");
        let buffer: AudioBuffer = buffer.dyn_into().expect("failed to decode");

        let mut id = [0; 32];
        rand::rng().fill_bytes(&mut id);

        let track = TrackInternal::new(id, buffer);

        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            p.tracks.push(track);
        });

        Self { id }
    }

    fn prepare(&self) {
        let i = self.internal_index();
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            let internal = p
                .tracks
                .get_mut(i)
                .expect("failed to prepare");
            if internal.node.is_some() {
                return;
            }
            let node = p.context.create_buffer_source().expect("failed to create source");
            node.set_buffer(Some(&internal.source));
            internal.node = Some(node);
            logging::log!("prepare done");
        });
    }

    fn play(&self) {
        self.prepare();
        let i = self.internal_index();
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            let internal = p
                .tracks
                .get_mut(i)
                .expect("failed to play");
            internal
                .node
                .as_ref()
                .expect("failed to play")
                .connect_with_audio_node(&p.context.destination())
                .expect("failed to play");
            let start_time = p.context.current_time();
            internal
                .node
                .as_ref()
                .expect("failed to play")
                .start_with_when_and_grain_offset(start_time, internal.offset)
                .expect("failed to play");
            internal.start_time = start_time - internal.offset;
            internal.is_playing = true;
            logging::log!("play done");
        });
    }

    fn pause(&self) {
        let i = self.internal_index();
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            let internal = p.tracks.get_mut(i).expect("failed to stop");
            if !internal.is_playing {
                return;
            }
            if let Some(node) = &internal.node {
                let current_time = p.context.current_time();
                node.stop().expect("failed to stop");
                node.disconnect().expect("failed to disconnect");
                internal.offset = current_time - internal.start_time;
                internal.is_playing = false;
                internal.node = None;
                logging::log!("pause done");
            }
        });
    }

    fn stop(&self) {
        let i = self.internal_index();
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            let internal = p.tracks.get_mut(i).expect("failed to stop");
            if !internal.is_playing {
                return;
            }
            if let Some(node) = &internal.node {
                node.stop().expect("failed to stop");
                node.disconnect().expect("failed to disconnect");
                internal.offset = 0.0;
                internal.is_playing = false;
                internal.node = None;
                logging::log!("stop done");
            }
        });
    }

    fn internal_index(&self) -> usize {
        AUDIO_PLAYER_INTERNAL
            .with_borrow_mut(|p| {
                p.tracks
                    .iter_mut()
                    .enumerate()
                    .find_map(|t| if t.1.id == self.id { Some(t.0) } else { None })
            })
            .expect("failed to get internal index")
    }

    fn internal_remove(&mut self) {
        self.stop();
        let i = self.internal_index();
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            _ = p.tracks.swap_remove(i);
        });
        logging::log!("remove done");
    }
}

impl Drop for TrackInner {
    fn drop(&mut self) {
        self.internal_remove();
    }
}
