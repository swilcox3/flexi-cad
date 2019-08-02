#[cfg(test)]
mod tests;

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

pub fn get_obj_data(file: &PathBuf, id: &RefID, prop_name: &String) -> Result<serde_json::Value, DBError> {
    let mut val = None;
    app_state::get_obj(file, id, |obj| {
        let data = obj.get_data(prop_name)?;
        val = Some(data);
        Ok(())
    })?;
    match val {
        Some(data) => Ok(data),
        None => Err(DBError::NotFound)
    }
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

pub fn copy_objs(file: PathBuf, event: UndoEventID, ids: HashSet<RefID>) -> Result<HashMap<RefID, RefID>, DBError> {
    let mut orig_to_copy = HashMap::new();
    for id in &ids {
        let copy_id = app_state::copy_obj(&file, &event, id)?;
        orig_to_copy.insert(id.clone(), copy_id.clone());
    }
    let mut to_update = Vec::new();
    //Reattach dependencies
    for id in ids {
        let mut refs_to_set = Vec::new();
        app_state::get_obj(&file, &id, |obj| {
            if let Some(has_ref) = obj.query_ref::<UpdateFromRefs>() {
                for ref_opt in has_ref.get_refs() {
                    if let Some(this_ref) = ref_opt {
                        if let Some(ref_copy_id) = orig_to_copy.get(&this_ref.id) {
                            if let Some(has_ref_res) = obj.query_ref::<ReferTo>() {
                                if let Some(res) = has_ref_res.get_result(&this_ref.ref_type) {
                                    refs_to_set.push((res, Reference {
                                        id: *ref_copy_id,
                                        ref_type: this_ref.ref_type.clone()
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        })?;
        if refs_to_set.len() > 0 {
            if let Some(copy_id) = orig_to_copy.get(&id) {
                app_state::modify_obj(&file, &event, copy_id, |obj| {
                    if let Some(has_ref) = obj.query_mut::<UpdateFromRefs>() {
                        for (res, ref_to_set) in &refs_to_set {
                            app_state::add_dep(&file, &ref_to_set.id, copy_id.clone())?;
                            has_ref.set_ref(&ref_to_set.ref_type, &res, ref_to_set.clone());
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

pub fn snap_ref_to_result(file: PathBuf, event: UndoEventID, own_ref: Reference, other_ref: Reference, guess: RefResult) -> Result<Option<RefResult>, DBError> {
    let mut which_opt = None;
    let mut res_opt = None;
    app_state::get_obj(&file, &other_ref.id, |refer_obj| {
        match refer_obj.query_ref::<ReferTo>() {
            Some(joinable) => {
                let results = joinable.get_results_for_type(&other_ref.ref_type);
                let mut dist = std::f64::MAX;
                let mut index = 0;
                for ref_res in results {
                    match ref_res {
                        RefResult::Point{pt} => {
                            let refer_pt = pt;
                            if let RefResult::Point{pt} = guess {
                                let cur_dist = refer_pt.distance2(pt);
                                if cur_dist < dist {
                                    res_opt = Some(ref_res);
                                    which_opt = Some(RefType::Point{which_pt: index});
                                    dist = cur_dist;
                                }
                            }
                        }
                        RefResult::Line{pt, dir} => {
                            let refer_pt = pt;
                            let refer_dir = dir;
                            if let RefResult::Point{pt} = guess {
                                let cur_dist = refer_pt.distance2(pt);
                                if cur_dist < dist {
                                    res_opt = Some(ref_res);
                                    which_opt = Some(RefType::)Line
                                }
                            }

                        }
                        _ => ()
                    }
                    index += 1;
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    if let Some(which) = which_opt {
        if let Some(calc_res) = &res_opt {
            app_state::modify_obj(&file, &event, &own_ref.id, |owner| {
                match owner.query_mut::<UpdateFromRefs>() {
                    Some(joinable) => {
                        joinable.set_ref(&own_ref.ref_type, calc_res, Reference{id: other_ref.id, ref_type: which.clone()});
                        Ok(())
                    }
                    None => Err(DBError::ObjLacksTrait)
                }
            })?;
            app_state::add_dep(&file, &own_ref.id, other_ref.id.clone())?;
            app_state::update_deps(file, own_ref.id);
        }
    }
    Ok(res_opt)
}