#[cfg(test)]
mod tests;
mod snapping;
pub use snapping::*;

use crate::*;

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
                let mut index = 0;
                for ref_opt in has_ref.get_refs() {
                    if let Some(this_ref) = ref_opt {
                        if let Some(ref_copy_id) = orig_to_copy.get(&this_ref.id) {
                            if let Some(has_ref_res) = obj.query_ref::<ReferTo>() {
                                if let Some(res) = has_ref_res.get_result(this_ref.index) {
                                    let ref_index = ReferInd{index: index};
                                    let copy_ref = Reference {
                                        id: *ref_copy_id,
                                        index: this_ref.index,
                                        ref_type: this_ref.ref_type
                                    };
                                    refs_to_set.push((ref_index, res, copy_ref));
                                }
                            }
                        }
                    }
                    index += 1;
                }
            }
            Ok(())
        })?;
        if refs_to_set.len() > 0 {
            if let Some(copy_id) = orig_to_copy.get(&id) {
                app_state::modify_obj(&file, &event, copy_id, |obj| {
                    if let Some(has_ref) = obj.query_mut::<UpdateFromRefs>() {
                        for (index, res, ref_to_set) in &refs_to_set {
                            app_state::add_dep(&file, &ref_to_set.id, copy_id.clone())?;
                            has_ref.set_ref(*index, res.clone(), ref_to_set.clone());
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
