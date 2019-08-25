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
pub use entities::dimension::Dimension;
pub use cgmath::prelude::*;

#[derive(Debug, PartialEq)]
pub enum DBError
{
    NotFound(String),
    Locked(String),
    Overwrite,
    NoUndoEvent,
    ObjNotFound,
    FileNotFound,
    PropertyNotFound,
    ObjLacksTrait,
    TimedOut,
    Other(String)
}

pub fn error_other<T: std::fmt::Debug>(err: T) -> DBError {
    DBError::Other(format!("{:?}", err))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateMsg {
    Empty,
    Delete{key: RefID},
    Mesh{data: MeshData},
    Read{query_id: QueryID, data: serde_json::Value},
    Other{data: serde_json::Value}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CmdMsg {
    pub func_name: String,
    pub params: Vec<serde_json::Value>
}

#[typetag::serde]
pub trait Data : Object + Send + Sync {
    fn get_id(&self) -> &RefID;
    fn update(&self) -> Result<UpdateMsg, DBError>;
    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError>;
    fn set_data(&mut self, data: &serde_json::Value) -> Result<(), DBError>;
    //Only use this if you know exactly what you're doing.
    fn set_id(&mut self, id: RefID);
}
mopo!(Data);

pub type DataObject = Box<dyn Data>;
pub type RefID = Uuid;
pub type UserID = Uuid;
pub type UndoEventID = Uuid;
pub type QueryID = Uuid;

pub trait ReferTo {
    fn get_result(&self, index: ResultInd) -> Option<RefGeometry>;
    fn get_all_results(&self) -> Vec<RefGeometry>;
}

pub trait UpdateFromRefs {
    fn clear_refs(&mut self);
    fn get_refs(&self) -> Vec<Option<Reference>>;
    fn set_ref(&mut self, index: ReferInd, result: &RefGeometry, other_ref: Reference, snap_pt: &Option<Point3f>);
    fn add_ref(&mut self, result: &RefGeometry, other_ref: Reference, snap_pt: &Option<Point3f>) -> bool;
    fn delete_ref(&mut self, index: ReferInd);
    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry>;
    fn update_from_refs(&mut self, results: &Vec<Option<RefGeometry>>); 
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
