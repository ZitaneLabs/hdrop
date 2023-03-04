use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use warp::{
    filters::BoxedFilter,
    http::StatusCode,
    multipart::{FormData, Part},
    Error, Filter, Rejection, Reply,
};

const FILE_SIZE: i32 = 256 * 1024 * 1024;

/*pub fn postUploadRoute() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone  {
    warp::path!("/")
    .and(warp::post())
    .map(|| "Status OK")
}*/

pub fn updateExpiryRoute(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "expiry")
        .and(warp::post())
        .and(warp::query::<HashMap<String, String>>())
        //.map(|e| format!("Expiry {}", e))
        .and(json_body())
        .map(|access_token, update_token, expiration_time| {
            update_expiry(access_token, update_token, expiration_time)
        })
}

/* handlers */
#[derive(Debug, Deserialize, Clone)]
pub struct Expiry {
    expiry: u64,
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ErrorData {
    reason: String,
}

impl<T> Response<T>
where
    T: Serialize,
{
    fn success() -> Response<()> {
        Response {
            status: "success".to_string(),
            data: None,
        }
    }

    fn missing_update_token() -> Response<ErrorData> {
        Response {
            status: "error".to_string(),
            data: Some(ErrorData {
                reason: "Missing update token".to_string(),
            }),
        }
    }
}

fn update_expiry(
    access_token: String,
    update_token: HashMap<String, String>,
    json_body: Expiry,
) -> impl warp::Reply {
    /*
    check_if_file_exists(accessToken)
    ->
    DB update expiry
    */
    //println!("arsch {:?}", update_token.get("id"));
    authenticate_update_token(update_token)

    // warp::reply::json(&Response::<()>::success())
}

/*
Handlers
 */

fn authenticate_update_token(update_token: HashMap<String, String>) -> impl warp::Reply {
    if let Some(update_token) = update_token.get("update_token") {
        unimplemented!("ToDo authenticate update_token against db entry")
    } else {
        warp::reply::json(&Response::<ErrorData>::missing_update_token())
    }

    /*
    ToDo:
    authenticate
    */
}
/*
 "Middleware"
*/

fn json_body() -> impl Filter<Extract = (Expiry,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    return warp::body::content_length_limit(1024 * 16).and(warp::body::json());
}
