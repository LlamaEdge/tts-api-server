#[cfg(feature = "gpt_sovits")]
pub(crate) mod gpt_sovits;
#[cfg(feature = "piper")]
pub(crate) mod piper;

use crate::{error, SERVER_INFO};
use endpoints::models::{ListModelsResponse, Model};
use hyper::{Body, Request, Response};
use std::time::SystemTime;

#[cfg(all(feature = "piper", feature = "gpt_sovits"))]
compile_error!("Only one of the features 'piper' and 'gpt_sovits' can be enabled at a time.");

pub(crate) async fn handle_llama_request(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        #[cfg(feature = "piper")]
        "/v1/audio/speech" => piper::audio_speech_handler(req).await,
        #[cfg(feature = "gpt_sovits")]
        "/v1/audio/speech" => gpt_sovits::audio_speech_handler(req).await,
        #[cfg(feature = "piper")]
        "/v1/files" => piper::files_handler(req).await,
        "/v1/info" => server_info_handler().await,
        "/v1/models" => models_handler().await,
        path => {
            #[cfg(feature = "piper")]
            if path.starts_with("/v1/files/") {
                piper::files_handler(req).await
            } else {
                error::invalid_endpoint(path)
            }
            #[cfg(feature = "gpt_sovits")]
            error::invalid_endpoint(path)
        }
    }
}

async fn server_info_handler() -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming server info request.");

    // get the server info
    let server_info = match SERVER_INFO.get() {
        Some(server_info) => server_info,
        None => {
            let err_msg = "The server info is not set.";

            // log
            error!(target: "stdout", "{}", &err_msg);

            return error::internal_server_error("The server info is not set.");
        }
    };

    // serialize server info
    let s = match serde_json::to_string(&server_info) {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("Fail to serialize server info. {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            return error::internal_server_error(err_msg);
        }
    };

    // return response
    let result = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Content-Type", "application/json")
        .body(Body::from(s));
    let res = match result {
        Ok(response) => response,
        Err(e) => {
            let err_msg = e.to_string();

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    };

    info!(target: "stdout", "Send the server info response.");

    res
}

async fn models_handler() -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming model list request.");

    // get the model name
    let server_info = SERVER_INFO.get().unwrap();
    let model_name = server_info.tts_model.name.clone();

    let list_models_response = ListModelsResponse {
        object: String::from("list"),
        data: vec![Model {
            id: model_name,
            created: SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            object: String::from("model"),
            owned_by: String::from("Not specified"),
        }],
    };

    // serialize response
    let s = match serde_json::to_string(&list_models_response) {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("Failed to serialize the model list result. Reason: {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            return error::internal_server_error(err_msg);
        }
    };

    // return response
    let result = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Content-Type", "application/json")
        .body(Body::from(s));
    let res = match result {
        Ok(response) => response,
        Err(e) => {
            let err_msg = format!("Failed to get model list. Reason: {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    };

    // log
    info!(target: "stdout", "Send the model list response.");

    res
}
