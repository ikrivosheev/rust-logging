use crate::model;
use actix::prelude::*;
use clickhouse_rs::{types::Enum8, Block, Pool};
use futures_util::{future, Future};
use log::{error, info};
use std::pin::Pin;
use std::time::Duration;

const BUFFER_SIZE: usize = 1000;
const INSERT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct NewRecord {
    level: Enum8,
    time: u64,
    message: String,
}

impl NewRecord {
    pub fn new(level: model::Level, time: u64, message: &str) -> Self {
        NewRecord {
            level: Enum8::of(level as i8),
            time: time,
            message: message.to_string(),
        }
    }
}

pub struct Inserter {
    pool: Pool,
    buffer: Vec<NewRecord>,
}

impl Inserter {
    pub fn new(pool: Pool) -> Self {
        Inserter {
            pool: pool,
            buffer: Vec::with_capacity(2 * BUFFER_SIZE),
        }
    }

    fn insert(&mut self) -> Pin<Box<(dyn Future<Output = Result<(), ()>> + 'static)>> {
        if self.buffer.len() == 0 {
            return Box::pin(future::ready(Ok(())));
        }

        info!("Insert, buffer size: {}", self.buffer.len());
        let pool = self.pool.clone();
        let mut time_buffer: Vec<u64> = Vec::with_capacity(self.buffer.len());
        let mut message_buffer: Vec<String> = Vec::with_capacity(self.buffer.len());
        let mut level_buffer: Vec<Enum8> = Vec::with_capacity(self.buffer.len());

        for msg in self.buffer.iter() {
            time_buffer.push(msg.time);
            message_buffer.push(msg.message.clone());
            level_buffer.push(msg.level);
        }

        Box::pin(async move {
            let mut client = match pool.get_handle().await {
                Ok(client) => client,
                Err(e) => {
                    error!("Connection to Clickhouse error: {}", e);
                    return Err(());
                }
            };
            let block = Block::new()
                .column("level", level_buffer)
                .column("time", time_buffer)
                .column("message", message_buffer);
            match client.insert("logs", block).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Cannot execute query: {}", e);
                    Err(())
                }
            }
        })
    }
}

impl Actor for Inserter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(INSERT_INTERVAL, |this, ctx| {
            ctx.wait(this.insert().into_actor(this).map(|res, act, _ctx| {
                if res.is_err() {
                    error!("Error on insert");
                    return ();
                }
                act.buffer = Vec::with_capacity(2 * BUFFER_SIZE);
            }));
        });
    }
}

impl Handler<NewRecord> for Inserter {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: NewRecord, _: &mut Self::Context) -> Self::Result {
        self.buffer.push(msg);
        info!("Succes push to buffer, current size: {}", self.buffer.len());
        if self.buffer.len() == BUFFER_SIZE {
            info!("Buffer is full, start insert");
            return Box::pin(self.insert().into_actor(self).map(|res, act, _ctx| {
                if res.is_err() {
                    return ();
                }
                info!("Success insert, clean buffer");
                act.buffer = Vec::with_capacity(2 * BUFFER_SIZE);
            }));
        }
        Box::pin(future::ready(()))
    }
}
