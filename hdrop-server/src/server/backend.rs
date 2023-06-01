use bytes::BufMut;
use futures::TryStreamExt;
//use tokio::time::error::Elapsed;
use std::convert::Infallible;
use uuid::Uuid;

const UPLOAD_LIMIT: u64 = 5000_000_000;

// default mode = false
const SHORT_MODE: bool = false;
