use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::path::PathBuf;
use data_model::*;
use ccl::dhashmap::DHashMap;
use crossbeam_channel::{Receiver};
use serde::Deserialize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

fn error<T: std::fmt::Debug>(err: T) -> String {
    format!("ERROR: {:?}", err)
}

#[derive(Deserialize)]
pub struct User {
    user_id: UserID
}

/// do websocket handshake and start `MyWebSocket` actor
pub fn ws_index(r: HttpRequest, user: web::Query<User>, stream: web::Payload) -> Result<HttpResponse, Error> {
    let res = ws::start(MyWebSocket::new(user.user_id), &r, stream);
    info!("{:?}", res.as_ref().unwrap());
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    id: UserID,
    updates: DHashMap<PathBuf, Receiver<UpdateMsg>>
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
                info!("{:?}", cmd);
                if let Err(e) = self.route(cmd) {
                    error!("{:?}", e);
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
    fn new(id: UserID) -> Self {
        Self { 
            hb: Instant::now(),
            id: id,
            updates: DHashMap::default()
        }
    }

    fn route(&self, mut msg: CmdMsg) -> Result<(), String> {
        match msg.func_name.as_ref() {
            "init_file" => {
                let (s, r) = crossbeam_channel::unbounded();
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::init_file(path.clone(), self.id.clone(), s);
                self.updates.insert(path, r);
                Ok(())
            }
            "begin_undo_event" => {
                let query: QueryID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let desc: String = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::begin_undo_event(&path, &self.id, desc, query).map_err(error)
            }
            "end_undo_event" => {
                let id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::end_undo_event(&path, id).map_err(error)
            }
            "undo_latest" => {
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::undo_latest(&path, &self.id).map_err(error)
            }
            "redo_latest" => {
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::redo_latest(&path, &self.id).map_err(error)
            }
            "suspend_event" => {
                let id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::suspend_event(&path, &id).map_err(error)
            }
            "resume_event" => {
                let id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::resume_event(&path, &id).map_err(error)
            }
            "cancel_event" => {
                let id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::cancel_event(&path, &id).map_err(error)
            }
            "take_undo_snapshot" => {
                let obj_id: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event_id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::take_undo_snapshot(&path, &event_id, &obj_id).map_err(error)
            }
            "join_objs" => {
                let point: Point3f = serde_json::from_value(msg.params.remove(6)).map_err(error)?;
                let type_2: RefType = serde_json::from_value(msg.params.remove(5)).map_err(error)?;
                let type_1: RefType = serde_json::from_value(msg.params.remove(4)).map_err(error)?;
                let id_2: RefID = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let id_1: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event_id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::join_objs(path, &event_id, id_1, id_2, &type_1, &type_2, &point).map_err(error)
            }
            "snap_obj_to_other" => {
                let point: Point3f = serde_json::from_value(msg.params.remove(5)).map_err(error)?;
                let type_1: RefType = serde_json::from_value(msg.params.remove(4)).map_err(error)?;
                let id_2: RefID = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let id_1: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event_id: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::snap_obj_to_other(path, &event_id, id_1, &id_2, &type_1, &point).map_err(error)
            }
            "can_refer_to" => {
                let query: QueryID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let id: RefID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::can_refer_to(&path, &id, query, &self.id).map_err(error)
            }
            "get_closest_result" => {
                let query: QueryID = serde_json::from_value(msg.params.remove(4)).map_err(error)?;
                let point: Point3f = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let type_1: RefType = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let id: RefID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::get_closest_result(&path, &id, &type_1, &point, query, &self.id).map_err(error)
            }
            "add_wall" => {
                let wall: Wall = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::add_obj(&path, &event, Box::new(wall)).map_err(error)
            }
            "add_door" => {
                let door: Door = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::add_obj(&path, &event, Box::new(door)).map_err(error)
            }
            "add_dimension" => {
                let dim: Dimension = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::add_obj(&path, &event, Box::new(dim)).map_err(error)
            }
            "move_obj" => {
                let delta: Vector3f = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let id: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::move_obj(path, &event, id, &delta).map_err(error)
            }
            "delete_obj" => {
                let id: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::delete_obj(&path, &event, &id).map_err(error)
            }
            "get_obj_data" => {
                let query: QueryID = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let prop_name: String = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let id: RefID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::get_obj_data(&path, &id, &prop_name, query, &self.id).map_err(error)
            }
            "set_obj_data" => {
                let data = msg.params.remove(3);
                let id: RefID = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::set_obj_data(path, &event, id, &data).map_err(error)
            }
            "set_objs_data" => {
                let data: Vec<(RefID, serde_json::Value)> = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::set_objs_data(path, &event, data).map_err(error)
            }
            "move_objs" => {
                let delta: Vector3f = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let data: std::collections::HashSet<RefID> = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::move_objs(path, &event, data, &delta).map_err(error)
            }
            "copy_objs" => {
                let query: QueryID = serde_json::from_value(msg.params.remove(3)).map_err(error)?;
                let data: std::collections::HashSet<RefID> = serde_json::from_value(msg.params.remove(2)).map_err(error)?;
                let event: UndoEventID = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::copy_objs(path, &event, data, query, &self.id).map_err(error)
            }
            "demo" => {
                let position: Point3f = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::demo(&path, &self.id, &position).map_err(error)
            }
            "demo_100" => {
                let position: Point3f = serde_json::from_value(msg.params.remove(1)).map_err(error)?;
                let path: PathBuf = serde_json::from_value(msg.params.remove(0)).map_err(error)?;
                operations_kernel::demo_100(path, self.id.clone(), position);
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
            for chunk in act.updates.chunks() {
                for (path, r) in chunk.iter() {
                    for msg in r.try_iter() {
                        info!("Sending msg: {:?} for file {:?}", msg, path);
                        ctx.text(serde_json::to_string(&msg).unwrap());
                    }
                }
            }
            ctx.ping("");
        });
    }
}
