use actix::*;
use actix_web::{middleware, web, App, HttpServer};
use clap;
use env_logger;

mod error;
mod view;
mod actors;


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
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Server port")
                .default_value("8000")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("database-login")
                .short("l")
                .long("database-login")
                .help("Database user:password")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("database-host")
                .short("H")
                .long("database-host")
                .help("Database host")
                .default_value("localhost")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("database-port")
                .short("P")
                .long("database-port")
                .help("Database port")
                .default_value("9000")
                .takes_value(true)
        )
        .get_matches();

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();
    
    // let url: String;
    // let database_host = matches.value_of("database-host").unwrap();
    // let database_port = matches.value_of("database-port").unwrap();
    // if let Some(login) = matches.value_of("database-login") {
        // url = format!("tcp://{}@{}:{}/logging?keepalive=10", login, database_host, database_port);
    // }
    // else {
        // url = format!("tcp://{}:{}/logging?keepalive=10", database_host, database_port);
    // }
    let inserter = actors::Inserter::new().start();
    
    HttpServer::new(move || {
        App::new()
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
