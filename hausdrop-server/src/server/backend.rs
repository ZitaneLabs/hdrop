#![deny(warnings)]

use bytes::BufMut;
use futures::TryStreamExt;
//use tokio::time::error::Elapsed;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

/*
Switch to mpart-async later for multipart upload
*/

const UPLOAD_LIMIT:u64 = 5000_000_000;

// default mode = false
const SHORT_MODE:bool = false;

#[tokio::main]
async fn main() {
    let upload_route = warp::path("upload")
    .and(warp::post())
    .and(warp::multipart::form().max_length(UPLOAD_LIMIT))
    .and_then(upload);
    
    let download_route = warp::path("files").and(warp::fs::dir("./files/"));

    let router = upload_route.or(download_route).recover(handle_rejection);
    println!("Server started at localhost:1220");
    warp::serve(router).run(([0, 0, 0, 0], 1220)).await;
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, format!("Payload too large (max {UPLOAD_LIMIT} bytes)"))
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}

async fn upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        if p.name() == "file" {
            let content_type = p.content_type();
            let file_ending;
            match content_type {
                Some(file_type) => match file_type {
                    "application/pdf" => {
                        file_ending = "pdf";
                    }
                    "image/png" => {
                        file_ending = "png";
                    }
                    v => {
                        //eprintln!("invalid file type found: {}", v);
                        //return Err(warp::reject::reject());
                        file_ending = "heb";
                    }
                },
                None => {
                    eprintln!("file type could not be determined");
                    return Err(warp::reject::reject());
                }
            }

            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let mut file_name = "overwrite";

            if SHORT_MODE {
                todo!("not implemented, TODO");
            } else {
                let file_name = format!("./files/{}.{}", Uuid::new_v4().to_string(), file_ending);
            }

            tokio::fs::write(&file_name, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
            println!("created file: {}", file_name);
        }
    }

    Ok("success")
}