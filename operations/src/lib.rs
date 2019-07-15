#![allow(dead_code)]
extern crate ccl;
#[macro_use] extern crate lazy_static;
extern crate tokio;
extern crate futures;
extern crate tokio_threadpool;
extern crate crossbeam_channel;
extern crate data_model;
#[cfg(test)] #[macro_use]
extern crate query_interface;
#[cfg(test)] #[macro_use]
extern crate serde_json;

#[cfg(test)]
mod tests;

mod operation_manager;
mod scheduler;
mod app_state;

pub use data_model::*;

use std::path::PathBuf;
pub use app_state::{init_file, begin_undo_event, end_undo_event, undo_latest, redo_latest, suspend_event, resume_event, cancel_event, take_undo_snapshot, delete_obj};

pub fn move_obj(file: &PathBuf, event: &UndoEventID, id: &RefID, delta: &Vector3f) -> Result<(), DBError> {
    app_state::modify_obj(file, event, id, |obj| {
        match obj.query_mut::<Position>() {
            Some(movable) => {
                movable.move_obj(delta);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::update_deps(file.clone(), id.clone());
    Ok(())
}

pub fn get_temp_wall(id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<UpdateMsg, DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    wall.update()
}

pub fn create_wall(file: &PathBuf, event: &UndoEventID, id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<(), DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    app_state::add_obj(file, event, wall)
}

pub fn join_walls(file: &PathBuf, event: &UndoEventID, id_1: &RefID, id_2: &RefID, pt: &Point3f) -> Result<(), DBError> {
    app_state::modify_obj(file, event, id_1, |first| {
        match first.query_mut::<RefPoint>() {
            Some(joinable) => Ok(joinable.set_point(1, pt.clone(), Reference{id: id_2.clone(), which_pt: 0})),
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::modify_obj(file, event, id_2, |second| {
        match second.query_mut::<RefPoint>() {
            Some(joinable) => Ok(joinable.set_point(0, pt.clone(), Reference{id: id_1.clone(), which_pt: 1})),
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::add_dep(file, id_1, id_2.clone())?;
    app_state::add_dep(file, id_2, id_1.clone())?;
    app_state::update_all_deps(file.clone(), vec!(id_1.clone(), id_2.clone()));
    Ok(())
}



