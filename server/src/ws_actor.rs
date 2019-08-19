use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::path::PathBuf;
use operations_kernel::*;
use ccl::dhashmap::DHashMap;
use crossbeam_channel::{Receiver};

lazy_static! {
    static ref UPDATES: DHashMap<PathBuf, Receiver<UpdateMsg>> = DHashMap::default();
}
/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

fn error<T: std::fmt::Debug>(err: T) -> String {
    format!("ERROR: {:?}", err)
}

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let res = ws::start(MyWebSocket::new(), &r, stream);
    println!("{:?}", res.as_ref().unwrap());
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(msg) => {
                let cmd: data_model::CmdMsg = serde_json::from_str(&msg).unwrap();
                println!("{:?}", cmd);
                if let Err(e) = MyWebSocket::route(cmd) {
                    ctx.text(e);
                }
            }
            ws::Message::Binary(_) => (),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self { 
            hb: Instant::now(),
        }
    }

    fn route(mut msg: CmdMsg) -> Result<(), String> {
        match msg.func_name.as_ref() {
            "init_file" => {
                let (s, r) = crossbeam_channel::unbounded();
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::init_file(path.clone(), s);
                UPDATES.insert(path, r);
                Ok(())
            }
            "demo_100" => {
                let position: Point3f = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::demo_100(path, position);
                Ok(())
            }
            _ => {
                Err(error("Not Implemented"))
            }
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }
            if let Some(r) = UPDATES.get(&PathBuf::from("defaultNew.flx")) {
                 if r.len() > 0 {
                    for msg in r.try_iter() {
                        ctx.text(serde_json::to_string(&msg).unwrap());
                    }
                }
            }
            ctx.ping("");
        });
    }
}
