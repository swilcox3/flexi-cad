extern crate uuid;
pub extern crate cgmath;
#[macro_use] extern crate query_interface;
#[macro_use] pub extern crate serde_json;
pub extern crate typetag;

mod geometry_kernel;
mod entities;

use uuid::Uuid;
use query_interface::Object;
use serde::{Serialize, Deserialize};

pub use geometry_kernel::*;
pub use entities::wall::Wall;
pub use entities::door::Door;

#[derive(Debug, PartialEq)]
pub enum DBError
{
    NotFound,
    Locked,
    Overwrite,
    NoUndoEvent,
    ObjLacksTrait,
    TimedOut,
    Other{msg: String}
}

pub fn error_other<T: std::fmt::Debug>(err: T) -> DBError {
    DBError::Other{msg: format!("{:?}", err)}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateMsg {
    Empty,
    Delete{key: RefID},
    Mesh{data: MeshData},
    Other{data: serde_json::Value}
}

#[typetag::serde]
pub trait Data : Object + Send + Sync {
    fn get_id(&self) -> &RefID;
    fn update(&self) -> Result<UpdateMsg, DBError>;
    fn set_data(&mut self, data: &serde_json::Value) -> Result<(), DBError>;
    //Only use this if you know exactly what you're doing.
    fn set_id(&mut self, id: RefID);
}
mopo!(Data);

pub type DataObject = Box<dyn Data>;
pub type RefID = Uuid;
pub type UserID = Uuid;
pub type UndoEventID = Uuid;

pub trait DepStore {
    fn register_sub(&self, publisher: &RefID, sub: RefID);
    fn delete_sub(&self, publisher: &RefID, sub: &RefID);
    fn delete_obj(&self, publisher: &RefID);
}

pub trait HasRefs : Data {
    fn init(&self, deps: &DepStore);
    fn clear_refs(&mut self);
}

pub trait ReferTo {
    fn get_result(&self, which: &RefType) -> Option<RefResult>;
    fn get_results_for_type(&self, which: &RefType) -> Vec<RefResult>;
}

pub trait UpdateFromRefs {
    fn get_refs(&self) -> Vec<Option<Reference>>;
    fn set_ref(&mut self, which_self: &RefType, result: &RefResult, other_ref: Reference);
    fn update_from_refs(&mut self, results: &Vec<Option<RefResult>>) -> Result<UpdateMsg, DBError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
