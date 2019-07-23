mod wall_ops;
#[cfg(test)]
mod tests;

pub use wall_ops::*;
use crate::*;
use data_model::cgmath::MetricSpace;

pub fn move_obj(file: PathBuf, event: UndoEventID, id: RefID, delta: Vector3f) -> Result<(), DBError> {
    app_state::modify_obj(&file, &event, &id, |obj| {
        match obj.query_mut::<Position>() {
            Some(movable) => {
                movable.move_obj(&delta);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::update_deps(file, id);
    Ok(())
}

pub fn move_objs(file: PathBuf, event: UndoEventID, ids: HashSet<RefID>, delta: Vector3f) -> Result<(), DBError> {
    for id in &ids {
        app_state::modify_obj(&file, &event, id, |obj| {
            match obj.query_mut::<Position>() {
                Some(movable) => {
                    movable.move_obj(&delta);
                    Ok(())
                }
                None => Err(DBError::ObjLacksTrait)
            }
        })?;
    }
    app_state::update_all_deps(file, ids.into_iter().collect());
    Ok(())
}

pub fn set_obj_data(file: PathBuf, event: UndoEventID, id: RefID, data: serde_json::Value) -> Result<(), DBError> {
    app_state::modify_obj(&file, &event, &id, |obj| {
        obj.set_data(&data)
    })?;
    app_state::update_deps(file, id);
    Ok(())
}

pub fn set_objs_data(file: PathBuf, event: UndoEventID, data: Vec<(RefID, serde_json::Value)>) ->Result<(), DBError> {
    let mut keys = HashSet::new();
    for (id, val) in data {
        app_state::modify_obj(&file, &event, &id, |obj| {
            obj.set_data(&val)
        })?;
        keys.insert(id);
    }
    app_state::update_all_deps(file, keys.into_iter().collect());
    Ok(())
}

pub fn copy_objs(file: PathBuf, event: UndoEventID, ids: HashSet<RefID>, delta: Vector3f) -> Result<HashMap<RefID, RefID>, DBError> {
    let mut orig_to_copy = HashMap::new();
    for id in &ids {
        let copy_id = app_state::copy_obj(&file, &event, id, &delta)?;
        orig_to_copy.insert(id.clone(), copy_id.clone());
    }
    let mut to_update = Vec::new();
    //Reattach dependencies
    for id in ids {
        let mut refs_to_set = Vec::new();
        app_state::get_obj(&file, &id, |obj| {
            if let Some(has_ref) = obj.query_ref::<UpdateFromPoint>() {
                let mut i = 0;
                for this_ref in has_ref.get_refs() {
                    if let Some(this_ref) = this_ref {
                        if let Some(ref_copy) = orig_to_copy.get(&this_ref.id) {
                            if let Some(pt) = has_ref.get_point(i) {
                                let shifted = pt + delta;
                                refs_to_set.push((i, shifted, Reference{id: ref_copy.clone(), which_pt: this_ref.which_pt}));
                            }
                        }
                    }
                    i = i + 1;
                }
            }
            Ok(())
        })?;
        if refs_to_set.len() > 0 {
            if let Some(copy_id) = orig_to_copy.get(&id) {
                app_state::modify_obj(&file, &event, copy_id, |obj| {
                    if let Some(has_ref) = obj.query_mut::<UpdateFromPoint>() {
                        for (which, pt, ref_to_set) in &refs_to_set {
                            app_state::add_dep(&file, &ref_to_set.id, copy_id.clone())?;
                            has_ref.set_point(*which, *pt, ref_to_set.clone());
                        }
                    }
                    Ok(())
                })?;
            }
        }
        to_update.push(id);
    }
    app_state::update_all_deps(file, to_update);
    Ok(orig_to_copy)
}

pub fn join_at_point(file: PathBuf, event: UndoEventID, id_1: RefID, id_2: RefID, mut pt: Point3f) -> Result<(), DBError> {
    let mut which_1 = 0;
    let mut which_2 = 0;
    app_state::get_obj(&file, &id_1, |first| {
        match first.query_ref::<UpdateFromPoint>() {
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
    app_state::get_obj(&file, &id_2, |second| {
        match second.query_ref::<UpdateFromPoint>() {
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
    app_state::modify_obj(&file, &event, &id_1, |first| {
        match first.query_mut::<UpdateFromPoint>() {
            Some(joinable) => {
                joinable.set_point(which_1, pt.clone(), Reference{id: id_2.clone(), which_pt: which_2});
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::modify_obj(&file, &event, &id_2, |second| {
        match second.query_mut::<UpdateFromPoint>() {
            Some(joinable) => {
                joinable.set_point(which_2, pt.clone(), Reference{id: id_1.clone(), which_pt: which_1});
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;

    app_state::add_dep(&file, &id_1, id_2.clone())?;
    app_state::add_dep(&file, &id_2, id_1.clone())?;
    app_state::update_all_deps(file, vec!(id_1, id_2));
    Ok(())
}