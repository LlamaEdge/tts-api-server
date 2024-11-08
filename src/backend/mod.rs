pub(crate) mod piper;

use crate::error;
use hyper::{Body, Request, Response};
use piper::audio_speech_handler;

pub(crate) async fn handle_llama_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        "/v1/audio/speech" => audio_speech_handler(req).await,
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
