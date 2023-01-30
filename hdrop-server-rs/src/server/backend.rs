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

const UPLOAD_LIMIT: u64 = 5000_000_000;

// default mode = false
const SHORT_MODE: bool = false;

#[derive(serde::Deserialize)]
struct FileData {
    file_data: String,
    file_name_data: String,
    file_name_hash: String,
    salt: String,
    iv: String,
}

#[tokio::main]
async fn main() {
    let api_base_v1 = warp::path("v1");
    let files_route_base = api_base_v1.and(warp::path("files"));

    let files_route_get = files_route_base
        .and(warp::path::param())
        .and(warp::get())
        .and_then(get_file);

    let files_route_post = files_route_base
        .and(warp::path::param())
        .and(warp::post())
        .and(json_body::<FileData>())
        .and_then(post_file);

    let router = files_route_get;

    println!("Server started at localhost:1220");
    warp::serve(router).run(([0, 0, 0, 0], 1220)).await;
}

fn json_body<'a, T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: serde::Deserialize<'a> + Sync + Send,
{
    warp::body::json()
}

async fn get_file(_access_token: String) -> Result<impl Reply, Rejection> {
    Err(warp::reject())?;
    Ok(StatusCode::OK)
}

async fn post_file(_access_token: String, file_data: FileData) -> Result<impl Reply, Rejection> {
    Err(warp::reject())?;
    Ok(StatusCode::OK)
}

fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            format!("Payload too large (max {UPLOAD_LIMIT} bytes)"),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
