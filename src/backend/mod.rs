pub(crate) mod piper;

use crate::error;
use hyper::{Body, Request, Response};
use piper::audio_speech_handler;

pub(crate) async fn handle_llama_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        "/v1/audio/speech" => audio_speech_handler(req).await,
        _ => error::invalid_endpoint(req.uri().path()),
    }
}
