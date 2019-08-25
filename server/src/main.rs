extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
extern crate serde;
extern crate serde_json;
extern crate operations_kernel;
extern crate data_model;
extern crate fern;
extern crate crossbeam_channel;
#[macro_use] extern crate log;
extern crate structopt;
#[macro_use] extern crate lazy_static;
extern crate ccl;

use actix_web::{web, App, HttpServer};
use structopt::StructOpt;
use log::LevelFilter;

mod ws_actor;

pub fn start(url: &str, ws_port: u16) {
    let ws_url = format!("{}:{}", url, ws_port);
    HttpServer::new(move || {
        App::new()
            .service(
                web::resource("/ws")
                    .route(web::get().to(ws_actor::ws_index))
            )
    })
    .bind(&ws_url).unwrap()
    .run().unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name="flexi-server", about = "A server for FlexiCAD")]
struct Opt {
    ///URL to run the server from
    #[structopt(name="url", default_value="127.0.0.1")]
    url: String,

    ///Port to run websockets from
    #[structopt(name="ws_port", default_value="80")]
    ws_port: u16,

    ///Logging level, from 0-5 where 0 is off
    #[structopt(short="l", long = "log", default_value="3")]
    log: u8,
}

fn main() {
    let opt = Opt::from_args();
    let log_level = match opt.log {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level_for("tokio_reactor", LevelFilter::Error)
        .level_for("mio", LevelFilter::Error)
        .level(log_level)
        .chain(std::io::stdout())
        .apply().unwrap();    

    start(&opt.url, opt.ws_port);
}
