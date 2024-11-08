use crate::error;
use endpoints::{audio::speech::SpeechRequest, files::DeleteFileStatus};
use hyper::{body::to_bytes, http::Method, Body, Request, Response};

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

    let audio_buffer = match llama_core::audio::create_speech(speech_request).await {
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
        .header("Content-Type", "audio/wav")
        .header("Content-Disposition", "attachment; filename=audio.wav")
        .body(Body::from(audio_buffer));

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

/// Upload, download, retrieve and delete a file, or list all files.
///
/// - `POST /v1/files`: Upload a file.
/// - `GET /v1/files`: List all files.
/// - `GET /v1/files/{file_id}`: Retrieve a file by id.
/// - `GET /v1/files/{file_id}/content`: Retrieve the content of a file by id.
/// - `GET /v1/files/download/{file_id}`: Download a file by id.
/// - `DELETE /v1/files/{file_id}`: Delete a file by id.
///
pub(crate) async fn files_handler(req: Request<Body>) -> Response<Body> {
    // log
    info!(target: "stdout", "Handling the coming files request");

    let res = if req.method() == Method::POST {
        match llama_core::files::upload_file(req).await {
            Ok(fo) => {
                // serialize chat completion object
                let s = match serde_json::to_string(&fo) {
                    Ok(s) => s,
                    Err(e) => {
                        let err_msg = format!("Failed to serialize file object. {}", e);

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

                match result {
                    Ok(response) => response,
                    Err(e) => {
                        let err_msg = e.to_string();

                        // log
                        error!(target: "stdout", "{}", &err_msg);

                        error::internal_server_error(err_msg)
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("{}", e);

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::GET {
        let uri_path = req.uri().path().trim_end_matches('/').to_lowercase();

        // Split the path into segments
        let segments: Vec<&str> = uri_path.split('/').collect();

        match segments.as_slice() {
            ["", "v1", "files"] => list_files(),
            ["", "v1", "files", file_id, "content"] => {
                if !file_id.starts_with("file_") {
                    let err_msg = format!("unsupported uri path: {}", uri_path);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }

                retrieve_file_content(file_id)
            }
            ["", "v1", "files", file_id] => {
                if !file_id.starts_with("file_") {
                    let err_msg = format!("unsupported uri path: {}", uri_path);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }

                retrieve_file(file_id)
            }
            ["", "v1", "files", "download", file_id] => download_file(file_id),
            _ => {
                let err_msg = format!("unsupported uri path: {}", uri_path);

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::DELETE {
        let id = req.uri().path().trim_start_matches("/v1/files/");
        let status = match llama_core::files::remove_file(id) {
            Ok(status) => status,
            Err(e) => {
                let err_msg = format!("Failed to delete the target file with id {}. {}", id, e);

                // log
                error!(target: "stdout", "{}", &err_msg);

                DeleteFileStatus {
                    id: id.into(),
                    object: "file".to_string(),
                    deleted: false,
                }
            }
        };

        // serialize status
        let s = match serde_json::to_string(&status) {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!(
                    "Failed to serialize the status of the file deletion operation. {}",
                    e
                );

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

        match result {
            Ok(response) => response,
            Err(e) => {
                let err_msg = e.to_string();

                // log
                error!(target: "stdout", "{}", &err_msg);

                error::internal_server_error(err_msg)
            }
        }
    } else if req.method() == Method::OPTIONS {
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
                error!(target: "files_handler", "{}", &err_msg);

                return error::internal_server_error(err_msg);
            }
        }
    } else {
        let err_msg = "Invalid HTTP Method.";

        // log
        error!(target: "stdout", "{}", &err_msg);

        error::internal_server_error(err_msg)
    };

    info!(target: "stdout", "Send the files response");

    res
}

fn list_files() -> Response<Body> {
    match llama_core::files::list_files() {
        Ok(file_objects) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&file_objects) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file list. {}", e);

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

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to list all files. {}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn retrieve_file(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::retrieve_file(id) {
        Ok(fo) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&fo) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file object. {}", e);

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

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn retrieve_file_content(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::retrieve_file_content(id) {
        Ok(content) => {
            // serialize chat completion object
            let s = match serde_json::to_string(&content) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to serialize file content. {}", e);

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

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}

fn download_file(id: impl AsRef<str>) -> Response<Body> {
    match llama_core::files::download_file(id) {
        Ok((filename, buffer)) => {
            // get the extension of the file
            let extension = filename.split('.').last().unwrap_or("unknown");
            let content_type = match extension {
                "txt" => "text/plain",
                "json" => "application/json",
                "png" => "image/png",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "wav" => "audio/wav",
                "mp3" => "audio/mpeg",
                "mp4" => "video/mp4",
                "md" => "text/markdown",
                _ => {
                    let err_msg = format!("Unsupported file extension: {}", extension);

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    return error::internal_server_error(err_msg);
                }
            };
            let content_disposition = format!("attachment; filename={}", filename);

            // return response
            let result = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Content-Type", content_type)
                .header("Content-Disposition", content_disposition)
                .body(Body::from(buffer));

            match result {
                Ok(response) => response,
                Err(e) => {
                    let err_msg = e.to_string();

                    // log
                    error!(target: "stdout", "{}", &err_msg);

                    error::internal_server_error(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);

            // log
            error!(target: "stdout", "{}", &err_msg);

            error::internal_server_error(err_msg)
        }
    }
}
