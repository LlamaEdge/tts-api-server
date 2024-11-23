#[cfg(feature = "gpt_sovits")]
pub(crate) mod gpt_sovits;
#[cfg(feature = "piper")]
pub(crate) mod piper;

use crate::error;

use hyper::{Body, Request, Response};

pub(crate) async fn handle_llama_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        #[cfg(feature = "piper")]
        "/v1/audio/speech" => piper::audio_speech_handler(req).await,
        #[cfg(feature = "gpt_sovits")]
        "/v1/audio/speech_gpt" => gpt_sovits::audio_speech_handler(req).await,
        "/v1/files" => piper::files_handler(req).await,
        path => {
            if path.starts_with("/v1/files/") {
                piper::files_handler(req).await
            } else {
                error::invalid_endpoint(path)
            }
        }
    }
}
