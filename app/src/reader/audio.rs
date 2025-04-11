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
        Self {
            inner: Arc::new(TrackInner::new(source).await),
        }
    }

    pub fn prepare(&self) {
        self.inner.prepare()
    }

    pub fn play(&self) {
        self.inner.play()
    }

    pub fn play_at(&self, offset: f64) {
        self.inner.play_at(offset)
    }

    pub fn pause(&self) {
        self.inner.pause()
    }

    pub fn stop(&self) {
        self.inner.stop()
    }

    pub fn progress(&self) -> Option<f64> {
        self.inner.progress()
    }
}

struct TrackInner {
    id: [u8; 32],
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

    fn with_player_and_index<T, R>(&self, f: T) -> R
    where
        T: FnOnce(&mut AudioPlayer, usize) -> R,
    {
        AUDIO_PLAYER_INTERNAL.with_borrow_mut(|p| {
            let i = p
                .tracks
                .iter_mut()
                .enumerate()
                .find_map(|t| if t.1.id == self.id { Some(t.0) } else { None })
                .expect("failed to get internal index");
            f(p, i)
        })
    }

    fn with_internal_and_context<T, R>(&self, f: T) -> R
    where
        T: FnOnce(&mut TrackInternal, &mut AudioContext) -> R,
    {
        self.with_player_and_index(|p, i| {
            let mut internal = p.tracks.get_mut(i).expect("internal failure");
            f(&mut internal, &mut p.context)
        })
    }

    fn prepare(&self) {
        self.with_internal_and_context(|i, c| {
            if i.node.is_some() {
                return;
            }
            let node = c.create_buffer_source().expect("failed to create source");
            node.set_buffer(Some(&i.source));
            i.node = Some(node);
            logging::log!("prepare done");
        });
    }

    fn play(&self) {
        self.internal_play(None);
    }

    fn play_at(&self, offset: f64) {
        self.internal_play(Some(offset));
    }

    fn internal_play(&self, offset: Option<f64>) {
        self.prepare();
        self.with_internal_and_context(|i, c| {
            i.node
                .as_ref()
                .expect("failed to play")
                .connect_with_audio_node(&c.destination())
                .expect("failed to play");
            let start_time = c.current_time();
            if let Some(o) = offset {
                i.offset = o;
            }
            i.node
                .as_ref()
                .expect("failed to play")
                .start_with_when_and_grain_offset(start_time, i.offset)
                .expect("failed to play");
            i.start_time = start_time - i.offset;
            i.is_playing = true;
            logging::log!("play done");
        });
    }

    fn pause(&self) {
        self.with_internal_and_context(|i, c| {
            if !i.is_playing {
                return;
            }
            if let Some(node) = &i.node {
                let current_time = c.current_time();
                #[allow(deprecated)]
                node.stop().expect("failed to stop");
                node.disconnect().expect("failed to disconnect");
                i.offset = current_time - i.start_time;
                i.is_playing = false;
                i.node = None;
                logging::log!("pause done");
            }
        });
    }

    fn stop(&self) {
        self.with_internal_and_context(|i, _| {
            if !i.is_playing {
                return;
            }
            if let Some(node) = &i.node {
                #[allow(deprecated)]
                node.stop().expect("failed to stop");
                node.disconnect().expect("failed to disconnect");
                i.offset = 0.0;
                i.is_playing = false;
                i.node = None;
                logging::log!("stop done");
            }
        });
    }

    fn progress(&self) -> Option<f64> {
        self.with_internal_and_context(|i, c| {
            if !i.is_playing {
                return None;
            }
            Some(c.current_time() - i.start_time)
        })
    }

    fn internal_remove(&mut self) {
        self.with_player_and_index(|p, i| {
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
