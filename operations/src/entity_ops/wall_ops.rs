use crate::*;
use data_model::cgmath::MetricSpace;

pub fn get_temp_wall(id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<UpdateMsg, DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    wall.update()
}

pub fn create_wall(file: &PathBuf, event: &UndoEventID, id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<(), DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    app_state::add_obj(file, event, wall)
}

pub fn join_walls(file: &PathBuf, event: &UndoEventID, id_1: &RefID, id_2: &RefID, mut pt: Point3f) -> Result<(), DBError> {
    let mut which_1 = 0;
    let mut which_2 = 0;
    app_state::get_obj(file, id_1, |first| {
        match first.query_ref::<RefPoint>() {
            Some(joinable) => {
                let dist_0 = joinable.get_point(0).unwrap().distance2(pt);
                let dist_1 = joinable.get_point(1).unwrap().distance2(pt);
                if dist_0 > dist_1 {
                    pt = joinable.get_point(1).unwrap().clone();
                    which_1 = 1;
                }
                else {
                    pt = joinable.get_point(0).unwrap().clone();
                    which_1 = 0;
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::get_obj(file, id_2, |second| {
        match second.query_ref::<RefPoint>() {
            Some(joinable) => {
                let dist_0 = joinable.get_point(0).unwrap().distance2(pt);
                let dist_1 = joinable.get_point(1).unwrap().distance2(pt);
                if dist_0 > dist_1 {
                    which_2 = 1;
                }
                else {
                    which_2 = 0;
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::modify_obj(file, event, id_1, |first| {
        match first.query_mut::<RefPoint>() {
            Some(joinable) => {
                joinable.set_point(which_1, pt.clone(), Reference{id: id_2.clone(), which_pt: which_2});
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::modify_obj(file, event, id_2, |second| {
        match second.query_mut::<RefPoint>() {
            Some(joinable) => {
                joinable.set_point(which_2, pt.clone(), Reference{id: id_1.clone(), which_pt: which_1});
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;

    app_state::add_dep(file, id_1, id_2.clone())?;
    app_state::add_dep(file, id_2, id_1.clone())?;
    app_state::update_all_deps(file.clone(), vec!(id_1.clone(), id_2.clone()));
    Ok(())
}