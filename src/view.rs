use crate::actors;
use crate::model;
use actix::Addr;
use actix_rt::spawn;
use actix_web::{
    error, get, http::header::ContentType, http::StatusCode, post, web, HttpResponse, Responder,
};
use clickhouse_rs::{types::Enum8, Pool};
use futures_channel::mpsc;
use futures_util::{sink::SinkExt, StreamExt};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;

#[post("/")]
pub async fn create(
    data: web::Json<model::LogRecord>,
    inserter: web::Data<Addr<actors::Inserter>>,
) -> impl Responder {
    let message = actors::NewRecord::new(data.level, data.time, &data.message);
    inserter.do_send(message);
    HttpResponse::Ok().json(data)
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ListRequest {
    level: model::Level,
    time_from: u64,
    time_to: u64,
}

#[derive(Debug, Serialize)]
pub struct ListRowResponse {
    level: model::Level,
    time: u64,
    message: String,
}

#[get("/")]
pub async fn list(
    q: web::Query<ListRequest>,
    pool: web::Data<Pool>,
) -> Result<impl Responder, error::Error> {
    let (tx, rx) = mpsc::channel(10);

    let mut client = match pool.get_handle().await {
        Ok(client) => client,
        Err(e) => {
            error!("Connection to Clickhouse error: {}", e);
            return Err(error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into());
        }
    };
    let query = format!(
        "SELECT level, time, message FROM logs WHERE level = {} and time BETWEEN {} AND {}",
        q.level as u8, q.time_from, q.time_to
    );

    spawn(async move {
        client
            .query(query)
            .stream_blocks()
            .for_each(|block_result| {
                let mut tx = tx.clone();
                async move {
                    let block = match block_result {
                        Ok(block) => block,
                        Err(e) => {
                            error!("Error on get block: {}", e);
                            return ();
                        }
                    };

                    for row in block.rows() {
                        let level: Enum8 = row.get("level").unwrap();
                        let level: model::Level = model::Level::try_from(level.internal()).unwrap();
                        let obj = ListRowResponse {
                            level: level,
                            time: row.get("time").unwrap(),
                            message: row.get("message").unwrap(),
                        };
                        let json = serde_json::to_string(&obj).unwrap();
                        let bytes = web::Bytes::from(format!("{}\n", json));
                        let result: Result<web::Bytes, ()> = Ok(bytes);
                        tx.send(result).await.unwrap();
                    }
                }
            })
            .await;
    });

    let mut builder = HttpResponse::Ok();
    builder.content_type(ContentType::octet_stream());
    Ok(builder.streaming(rx))
}
