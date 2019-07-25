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

pub fn join_references(file: PathBuf, event: UndoEventID, ref_1: Reference, ref_2: Reference, mut res: RefResult) -> Result<(), DBError> {
    let mut which_opt_1 = None;
    let mut which_opt_2 = None;
    app_state::get_obj(&file, &ref_1.id, |first| {
        match first.query_ref::<ReferTo>() {
            Some(joinable) => {
                let res_opt_1 = joinable.get_result(&ref_1.ref_type);
                let res_opt_2 = joinable.get_result(&ref_2.ref_type);
                if let Some(RefResult::Point{pt}) = res_opt_1 {
                    let pt_1 = pt;
                    if let Some(RefResult::Point{pt}) = res_opt_2 {
                        let pt_2 = pt;
                        if let RefResult::Point{pt} = res {
                            let dist_1 = pt_1.distance2(pt);
                            let dist_2 = pt_2.distance2(pt);
                            if dist_1 > dist_2 {
                                res = RefResult::Point{pt: pt_2};
                                which_opt_1 = Some(RefType::Point{which_pt: 1});
                            }
                            else {
                                res = RefResult::Point{pt: pt_1};
                                which_opt_1 = Some(RefType::Point{which_pt: 0});
                            }
                        }
                    }
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::get_obj(&file, &ref_2.id, |second| {
        match second.query_ref::<ReferTo>() {
            Some(joinable) => {
                let res_opt_1 = joinable.get_result(&ref_1.ref_type);
                let res_opt_2 = joinable.get_result(&ref_2.ref_type);
                if let Some(RefResult::Point{pt}) = res_opt_1 {
                    let pt_1 = pt;
                    if let Some(RefResult::Point{pt}) = res_opt_2 {
                        let pt_2 = pt;
                        if let RefResult::Point{pt} = res {
                            let dist_1 = pt_1.distance2(pt);
                            let dist_2 = pt_2.distance2(pt);
                            if dist_1 > dist_2 {
                                which_opt_2 = Some(RefType::Point{which_pt: 1});
                            }
                            else {
                                which_opt_2 = Some(RefType::Point{which_pt: 0});
                            }
                        }
                    }
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    if let Some(which_1) = which_opt_1 {
        if let Some(which_2) = which_opt_2 {
            app_state::modify_obj(&file, &event, &ref_1.id, |first| {
                match first.query_mut::<UpdateFromRefs>() {
                    Some(joinable) => {
                        joinable.set_ref(&which_1, &res, Reference{id: ref_2.id, ref_type: which_2.clone()});
                        Ok(())
                    }
                    None => Err(DBError::ObjLacksTrait)
                }
            })?;
            app_state::modify_obj(&file, &event, &ref_2.id, |second| {
                match second.query_mut::<UpdateFromRefs>() {
                    Some(joinable) => {
                        joinable.set_ref(&which_2, &res, Reference{id: ref_1.id, ref_type: which_1.clone()});
                        Ok(())
                    }
                    None => Err(DBError::ObjLacksTrait)
                }
            })?;
            app_state::add_dep(&file, &ref_1.id, ref_2.id.clone())?;
            app_state::add_dep(&file, &ref_2.id, ref_1.id.clone())?;
            app_state::update_all_deps(file, vec!(ref_1.id, ref_2.id));
        }
    }
    Ok(())
}