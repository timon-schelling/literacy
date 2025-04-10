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
