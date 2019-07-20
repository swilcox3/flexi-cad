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

pub trait ObjStore {
    fn get_obj(&self, id: &RefID, callback: &mut FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError>;
    fn modify_obj(&self, event: &UndoEventID, id: &RefID, callback: &mut FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError>;
    fn delete_obj(&self, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError>;
    fn add_object(&self, event: &UndoEventID, obj: DataObject) -> Result<(), DBError>;
    fn get_ref_point(&self, refer: &mut Reference) -> Result<Point3f, DBError> {
        let mut result = Point3f::new(0.0, 0.0, 0.0);
        if let Err(e) = self.get_obj(&refer.id, &mut |obj: &DataObject| {
            match obj.query_ref::<RefPoint>() {
                Some(ref_obj) => {
                    match ref_obj.get_point(refer.which_pt) {
                        Some(pt) => {
                            result = pt.clone();
                            Ok(())
                        }
                        None => {
                            Err(DBError::NotFound)
                        }
                    }
                }
                None => Err(DBError::ObjLacksTrait)
            }
        }) {
            if e == DBError::NotFound {
                refer.id = RefID::nil();
            }
            return Err(e);
        }
        Ok(result)
    }
}

pub trait Update : Data {
    fn init(&self, deps: &DepStore);
    fn clear_refs(&mut self);
    fn get_refs(&self) -> Vec<RefID>;
    fn update_from_refs(&mut self, objs: &ObjStore) -> Result<UpdateMsg, DBError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
