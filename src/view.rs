use actix::Addr;
use actix_web::{get, post, web, Responder};
use serde::Deserialize;

use crate::actors;

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    level: String,
    time: u64,
    message: String,
}

#[post("/")]
pub async fn create(data: web::Json<CreateRequest>, inserter: web::Data<Addr<actors::Inserter>>) -> impl Responder {
    let message = actors::NewRecord::new(&data.level, data.time, &data.message);
    inserter.do_send(message);
    format!("Test: {}, {}, {}", data.level, data.time, data.message)
}

#[derive(Debug, Deserialize)]
pub struct ListRequest {
    level: String,
    time: u64,
}

#[get("/")]
pub async fn list(q: web::Query<ListRequest>) -> impl Responder {
    format!("Test: {}, {}", q.level, q.time)
}
