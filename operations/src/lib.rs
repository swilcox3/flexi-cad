#![allow(dead_code)]
extern crate ccl;
#[macro_use] extern crate lazy_static;
extern crate rayon;
extern crate crossbeam_channel;
extern crate data_model;
#[cfg(test)] #[macro_use]
extern crate query_interface;
#[cfg(test)] #[macro_use]
extern crate serde_json;
extern crate bincode;

#[cfg(test)]
mod tests;

mod operation_manager;
mod app_state;
mod entity_ops;

pub use data_model::*;
pub use entity_ops::*;

pub use std::path::PathBuf;
pub use std::collections::{HashSet, HashMap, VecDeque};
pub use app_state::*;
pub use rayon::prelude::*;

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
    let event = app_state::begin_undo_event(file, &user, String::from("Demo"))?;
    app_state::add_obj(file, &event, Box::new(wall_1))?;
    app_state::add_obj(file, &event, Box::new(wall_2))?;
    app_state::add_obj(file, &event, Box::new(wall_3))?;
    app_state::add_obj(file, &event, Box::new(wall_4))?;
    join_refs(file, &event, &id_1, &id_2, &RefType::Point, &RefType::Point, &position_2)?;
    join_refs(file, &event, &id_2, &id_3, &RefType::Point, &RefType::Point, &position_3)?;
    join_refs(file, &event, &id_3, &id_4, &RefType::Point, &RefType::Point, &position_4)?;
    join_refs(file, &event, &id_4, &id_1, &RefType::Point, &RefType::Point, position)?;
    let door_pos = position + Vector3f::new(side_length / 2.0, 0.0, 0.0);
    let door = Door::new(RefID::new_v4(), door_pos, door_pos + Vector3f::new(5.0, 0.0, 0.0), width / 2.0, height - 1.0);
    let door_id = door.get_id().clone();
    app_state::add_obj(file, &event, Box::new(door))?;
    join_refs(file, &event, &door_id, &id_1, &RefType::Line, &RefType::Rect, &door_pos)?;
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
    snap_to_ref(file, &event, &dim_id_1, &id_1, &RefType::Point, position)?;
    snap_to_ref(file, &event, &dim_id_1, &id_1, &RefType::Point, &position_2)?;
    snap_to_ref(file, &event, &dim_id_2, &id_2, &RefType::Point, &position_2)?;
    snap_to_ref(file, &event, &dim_id_2, &id_2, &RefType::Point, &position_3)?;
    snap_to_ref(file, &event, &dim_id_3, &id_3, &RefType::Point, &position_3)?;
    snap_to_ref(file, &event, &dim_id_3, &id_3, &RefType::Point, &position_4)?;
    snap_to_ref(file, &event, &dim_id_4, &id_4, &RefType::Point, &position_4)?;
    snap_to_ref(file, &event, &dim_id_4, &id_4, &RefType::Point, position)?;
    app_state::end_undo_event(file, event)
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