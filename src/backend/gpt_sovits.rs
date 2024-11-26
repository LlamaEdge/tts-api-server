use crate::error;
use hyper::{body::to_bytes, http::Method, Body, Request, Response};

mod ffi {
    #[link(wasm_import_module = "gpt_sovits")]
    extern "C" {
        pub fn infer(
            speaker_ptr: *const u8,
            speaker_len: usize,
            text_ptr: *const u8,
            text_len: usize,
        ) -> i32;
        pub fn get_output(output_buf: *mut u8, output_len: usize) -> i32;
    }
}

fn infer(speaker: &str, text: &str) -> Result<Vec<u8>, &'static str> {
    unsafe {
        let i = ffi::infer(speaker.as_ptr(), speaker.len(), text.as_ptr(), text.len());
        match i {
            -1 => Err("gpt_sovits infer error"),
            -2 => Err("gpt_sovits runtime error"),
            _ => {
                let mut buf = vec![0u8; i as usize];
                let o = ffi::get_output(buf.as_mut_ptr(), i as usize);
                match o {
                    -2 => Err("gpt_sovits runtime error"),
                    _ => Ok(buf),
                }
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SpeechRequest {
    /// The text to generate audio for.
    pub input: String,
    /// Id of speaker.
    pub speaker: String,
}

fn create_speech(speech_request: SpeechRequest) -> anyhow::Result<Vec<u8>> {
    let result =
        infer(&speech_request.speaker, &speech_request.input).map_err(|e| anyhow::anyhow!(e))?;
    Ok(result)
}

pub(crate) async fn audio_speech_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming audio speech request");

    if req.method().eq(&Method::OPTIONS) {
        let result = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Content-Type", "application/json")
            .body(Body::empty());

        match result {
            Ok(response) => return response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    }

    info!(target: "stdout", "Prepare the chat completion request.");

    // parse request
    let body_bytes = match to_bytes(req.into_body()).await {
        Ok(body_bytes) => body_bytes,
        Err(e) => {
            let err_msg = format!("Fail to read buffer from request body. {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            return error::internal_server_error(err_msg);
        }
    };
    let speech_request: SpeechRequest = match serde_json::from_slice(&body_bytes) {
        Ok(speech_request) => speech_request,
        Err(e) => {
            let err_msg = format!("Fail to deserialize speech request: {msg}", msg = e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            return error::bad_request(err_msg);
        }
    };

    let wav_data = match create_speech(speech_request) {
        Ok(obj) => obj,
        Err(e) => {
            let err_msg = format!("Failed to transcribe the audio. {}", e);

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
        .header(hyper::header::CONTENT_TYPE, "audio/wav")
        .header(
            hyper::header::CONTENT_DISPOSITION,
            "attachment; filename=audio.wav",
        )
        .body(Body::from(wav_data));

    let res = match result {
        Ok(response) => response,
        Err(e) => {
            let err_msg = e.to_string();

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    };

    info!(target: "stdout", "Send the audio speech response");

    res
}
