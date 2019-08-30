#![allow(dead_code)]
extern crate ccl;
#[macro_use] extern crate lazy_static;
extern crate rayon;
extern crate crossbeam_channel;
extern crate data_model;
#[cfg(test)] #[macro_use]
extern crate query_interface;
#[macro_use] extern crate serde_json;
extern crate bincode;
#[macro_use] extern crate log;

#[cfg(test)]
mod tests;

mod operation_manager;
pub mod app_state;
pub mod entity_ops;

mod prelude {
    pub use data_model::*;
    pub use crate::entity_ops;
    pub use std::path::PathBuf;
    pub use crate::app_state;
    pub use rayon::prelude::*;
    pub use std::collections::{HashSet, HashMap, VecDeque};
}

use prelude::*;

///This is the only value that can be returned from a library interface.
type LibResult = Result<(), DBError>;

pub use app_state::{open_file, init_file, save_file, save_as_file, begin_undo_event, end_undo_event, undo_latest, redo_latest, suspend_event, resume_event,
    cancel_event, take_undo_snapshot, add_obj, copy_obj};

pub fn get_obj(file: &PathBuf, obj_id: &RefID, query_id: QueryID, user_id: &UserID) -> LibResult {
    app_state::get_obj(file, obj_id, |obj| {
        app_state::send_read_result(file, query_id, user_id, json!(obj))
    })
}

pub fn delete_obj(file: &PathBuf, event: &UndoEventID, obj_id: &RefID) -> LibResult {
    app_state::delete_obj(file, event, obj_id)?;
    Ok(())
}

pub fn move_obj(file: PathBuf, event: &UndoEventID, obj_id: RefID, delta: &Vector3f) -> LibResult {
    entity_ops::move_obj(&file, event, &obj_id, delta)?;
    app_state::update_deps(file, obj_id);
    Ok(())
}

pub fn move_objs(file: PathBuf, event: &UndoEventID, ids: HashSet<RefID>, delta: &Vector3f) -> LibResult {
    for id in &ids {
        entity_ops::move_obj(&file, event, id, delta)?;
    }
    app_state::update_all_deps(file, ids.into_iter().collect());
    Ok(())
}

pub fn get_obj_data(file: &PathBuf, obj_id: &RefID, prop_name: &str, query_id: QueryID, user_id: &UserID) -> LibResult {
    let data = entity_ops::get_obj_data(file, obj_id, prop_name)?;
    app_state::send_read_result(file, query_id, user_id, data)
}

pub fn set_obj_data(file: PathBuf, event: &UndoEventID, obj_id: RefID, data: &serde_json::Value) -> LibResult {
    entity_ops::set_obj_data(&file, event, &obj_id, data)?;
    app_state::update_deps(file, obj_id);
    Ok(())
}

pub fn set_objs_data(file: PathBuf, event: &UndoEventID, data: Vec<(RefID, serde_json::Value)>) -> LibResult {
    let mut keys = HashSet::new();
    for (id, val) in data {
        entity_ops::set_obj_data(&file, event, &id, &val)?;
        keys.insert(id);
    }
    app_state::update_all_deps(file, keys.into_iter().collect());
    Ok(())
}

pub fn copy_objs(file: PathBuf, event: &UndoEventID, ids: HashSet<RefID>, query_id: QueryID, user_id: &UserID) -> LibResult {
    let (to_update, copied) = entity_ops::copy_objs(&file, event, ids)?;
    app_state::send_read_result(&file, query_id, user_id, json!(copied))?;
    app_state::update_all_deps(file, to_update);
    Ok(())
}

pub fn snap_obj_to_other(file: PathBuf, event: &UndoEventID, obj: RefID, other_obj: &RefID, only_match: &RefType, guess: &Point3f) -> LibResult {
    entity_ops::snap_to_ref(&file, event, &obj, other_obj, only_match, guess)?;
    app_state::update_deps(file, obj);
    Ok(())
}

pub fn join_objs(file: PathBuf, event: &UndoEventID, first: RefID, second: RefID, first_wants: &RefType, second_wants: &RefType, guess: &Point3f) -> LibResult {
    entity_ops::join_refs(&file, event, &first, &second, first_wants, second_wants, guess)?;
    app_state::update_all_deps(file, vec![first, second]);
    Ok(())
}

pub fn can_refer_to(file: &PathBuf, obj_id: &RefID, query_id: QueryID, user_id: &UserID) -> LibResult {
    let can_refer = entity_ops::can_refer_to(file, obj_id)?;
    app_state::send_read_result(file, query_id, user_id, json!(can_refer))
}

pub fn get_closest_result(file: &PathBuf, obj_id: &RefID, only_match: &RefType, guess: &Point3f, query_id: QueryID, user_id: &UserID) -> LibResult {
    let res = entity_ops::get_closest_result(file, obj_id, only_match, guess)?;
    app_state::send_read_result(file, query_id, user_id, json!(res))
}

pub fn demo(file: &PathBuf, user: &UserID, position: &Point3f) -> Result<(), DBError> {
    let side_length = 50.0;
    let width = 1.0;
    let height = 5.0;
    let position_2 = position + Vector3f::new(side_length, 0.0, 0.0);
    let position_3 = position + Vector3f::new(side_length, side_length, 0.0);
    let position_4 = position + Vector3f::new(0.0, side_length, 0.0);
    let wall_1 = Wall::new(RefID::new_v4(), position.clone(), position_2.clone(), width, height);
    let id_1 = wall_1.get_id().clone();
    let wall_2 = Wall::new(RefID::new_v4(), position_2.clone(), position_3.clone(), width, height);
    let id_2 = wall_2.get_id().clone();
    let wall_3 = Wall::new(RefID::new_v4(), position_3.clone(), position_4.clone(), width, height);
    let id_3 = wall_3.get_id().clone();
    let wall_4 = Wall::new(RefID::new_v4(), position_4.clone(), position.clone(), width, height);
    let id_4 = wall_4.get_id().clone();
    let event = UndoEventID::new_v4();
    app_state::begin_undo_event(file, &user, event.clone(), String::from("Demo"))?;
    app_state::add_obj(file, &event, Box::new(wall_1))?;
    app_state::add_obj(file, &event, Box::new(wall_2))?;
    app_state::add_obj(file, &event, Box::new(wall_3))?;
    app_state::add_obj(file, &event, Box::new(wall_4))?;
    entity_ops::join_refs(file, &event, &id_1, &id_2, &RefType::Point, &RefType::Point, &position_2)?;
    entity_ops::join_refs(file, &event, &id_2, &id_3, &RefType::Point, &RefType::Point, &position_3)?;
    entity_ops::join_refs(file, &event, &id_3, &id_4, &RefType::Point, &RefType::Point, &position_4)?;
    entity_ops::join_refs(file, &event, &id_4, &id_1, &RefType::Point, &RefType::Point, position)?;
    let door_pos = position + Vector3f::new(side_length / 2.0, 0.0, 0.0);
    let door = Door::new(RefID::new_v4(), door_pos, door_pos + Vector3f::new(5.0, 0.0, 0.0), width / 2.0, height - 1.0);
    let door_id = door.get_id().clone();
    app_state::add_obj(file, &event, Box::new(door))?;
    entity_ops::join_refs(file, &event, &door_id, &id_1, &RefType::Line, &RefType::Rect, &door_pos)?;
    let offset = 5.0;
    let dim_1 = Dimension::new(RefID::new_v4(), position.clone(), position_2.clone(), offset);
    let dim_2 = Dimension::new(RefID::new_v4(), position_2.clone(), position_3.clone(), offset);
    let dim_3 = Dimension::new(RefID::new_v4(), position_3.clone(), position_4.clone(), offset);
    let dim_4 = Dimension::new(RefID::new_v4(), position_4.clone(), position.clone(), offset);
    let dim_id_1 = dim_1.get_id().clone();
    let dim_id_2 = dim_2.get_id().clone();
    let dim_id_3 = dim_3.get_id().clone();
    let dim_id_4 = dim_4.get_id().clone();
    app_state::add_obj(file, &event, Box::new(dim_1))?;
    app_state::add_obj(file, &event, Box::new(dim_2))?;
    app_state::add_obj(file, &event, Box::new(dim_3))?;
    app_state::add_obj(file, &event, Box::new(dim_4))?;
    entity_ops::snap_to_ref(file, &event, &dim_id_1, &id_1, &RefType::Point, position)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_1, &id_1, &RefType::Point, &position_2)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_2, &id_2, &RefType::Point, &position_2)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_2, &id_2, &RefType::Point, &position_3)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_3, &id_3, &RefType::Point, &position_3)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_3, &id_3, &RefType::Point, &position_4)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_4, &id_4, &RefType::Point, &position_4)?;
    entity_ops::snap_to_ref(file, &event, &dim_id_4, &id_4, &RefType::Point, position)?;
    app_state::end_undo_event(file, event)?;
    app_state::update_all_deps(file.clone(), vec![id_1, id_2, id_3, id_4, door_id, dim_id_1, dim_id_2, dim_id_3, dim_id_4]);
    Ok(())
}

pub fn demo_100(file: PathBuf, user: UserID, position: Point3f) {
    rayon::ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();
    rayon::spawn(move || {
        let i_s: Vec<u64> = (0..10).collect();
        let j_s: Vec<u64> = (0..10).collect();
        i_s.par_iter().for_each(|i| {
            j_s.par_iter().for_each(|j| {
                demo(&file, &user, &(position + Vector3f::new(75.0 * (*i as f64), 75.0 * (*j as f64), 0.0))).unwrap();
            });
        });
    });
}