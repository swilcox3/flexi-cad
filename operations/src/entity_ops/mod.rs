#[cfg(test)]
mod tests;
mod snapping;
pub use snapping::*;

use crate::prelude::*;

pub fn move_obj(file: &PathBuf, event: &UndoEventID, id: &RefID, delta: &Vector3f) -> Result<(), DBError> {
    app_state::modify_obj(file, event, id, |obj| {
        match obj.query_mut::<dyn Position>() {
            Some(movable) => {
                movable.move_obj(delta);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })
}

pub fn get_obj_data(file: &PathBuf, id: &RefID, prop_name: &str) -> Result<serde_json::Value, DBError> {
    let mut val = None;
    app_state::get_obj(file, id, |obj| {
        let data = obj.get_data(prop_name)?;
        val = Some(data);
        Ok(())
    })?;
    match val {
        Some(data) => Ok(data),
        None => Err(DBError::PropertyNotFound)
    }
}

pub fn set_obj_data(file: &PathBuf, event: &UndoEventID, id: &RefID, data: &serde_json::Value) -> Result<(), DBError> {
    app_state::modify_obj(file, event, id, |obj| {
        obj.set_data(data)
    })
}

pub fn copy_objs(file: &PathBuf, event: &UndoEventID, ids: HashSet<RefID>) -> Result<(Vec<RefID>, HashMap<RefID, RefID>), DBError> {
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
            if let Some(has_ref) = obj.query_ref::<dyn UpdateFromRefs>() {
                let mut index = 0;
                for ref_opt in has_ref.get_refs() {
                    if let Some(this_ref) = ref_opt {
                        if let Some(ref_copy_id) = orig_to_copy.get(&this_ref.obj) {
                            if let Some(has_ref_res) = obj.query_ref::<dyn ReferTo>() {
                                if let Some(res) = has_ref_res.get_point(this_ref.index) {
                                    let ref_index = index;
                                    let copy_ref = GeometryId {
                                        obj: *ref_copy_id,
                                        index: this_ref.index,
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
                    if let Some(has_ref) = obj.query_mut::<dyn UpdateFromRefs>() {
                        for (index, res, ref_to_set) in &refs_to_set {
                            app_state::add_dep(&file, &ref_to_set, GeometryId{obj: copy_id.clone(), index: *index})?;
                            has_ref.set_ref(*index, *res, ref_to_set.clone());
                        }
                    }
                    Ok(())
                })?;
            }
        }
        to_update.push(id);
    }
    Ok((to_update, orig_to_copy))
}
