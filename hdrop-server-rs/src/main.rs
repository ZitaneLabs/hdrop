mod api;
mod core;
mod schema;
mod server;
use warp::Filter;

#[tokio::main]
async fn main() {
    // webserver & api
    let api = routeAggregator();

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

fn routeAggregator() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    api::statusRoute().or(api::updateExpiryRoute())
}
