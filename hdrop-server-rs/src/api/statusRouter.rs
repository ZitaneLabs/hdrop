use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

pub fn statusRoute() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    //warp::path!("status").and(warp::get()).map(|| "Status OK")
    warp::path!("status").and(warp::get()).map(|| warp::reply())
}
