use actix::*;
use actix_web::{middleware, web, App, HttpServer};
use clap;
use clickhouse_rs::Pool;
use env_logger;
use log::error;
use std::process;

mod actors;
mod error;
mod model;
mod view;

async fn init_database(pool: Pool) -> bool {
    let query = include_str!("../schema.sql");
    let result = pool.get_handle().await;
    let mut client = match result {
        Ok(client) => client,
        Err(e) => {
            error!("Cannot connect to server: {}", e);
            return false;
        }
    };

    match client.execute(query).await {
        Ok(_) => true,
        Err(e) => {
            error!("Cannot execute query: {}", e);
            false
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let matches = clap::App::new("Logging")
        .author("Ivan Krivosheev <py.krivosheev@gmail.com>")
        .arg(
            clap::Arg::with_name("host")
                .long("host")
                .help("Server host")
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Server port")
                .default_value("8000")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("database-host")
                .short("H")
                .long("database-host")
                .help("Database host")
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("database-port")
                .short("P")
                .long("database-port")
                .help("Database port")
                .default_value("9000")
                .takes_value(true),
        )
        .get_matches();

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();

    let database_host = matches.value_of("database-host").unwrap();
    let database_port = matches.value_of("database-port").unwrap();
    let pool = Pool::new(format!("tcp://{}:{}/logging", database_host, database_port));

    if !init_database(pool.clone()).await {
        process::exit(1);
    }

    let inserter = actors::Inserter::new(pool.clone()).start();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(inserter.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(8192)
                    .error_handler(error::json_handler),
            )
            .app_data(web::QueryConfig::default().error_handler(error::query_handler))
            .wrap(middleware::Logger::default())
            .service(web::scope("/log").service(view::create).service(view::list))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
