use crate::*;

fn get_result(file: &PathBuf, obj: &RefID, index: ResultInd) -> Result<Option<RefGeometry>, DBError> {
    let mut res_opt = None;
    app_state::get_obj(file, obj, |read| {
        match read.query_ref::<ReferTo>() {
            Some(refer) => {
                res_opt = refer.get_result(index);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(res_opt)
}

fn get_closest_result(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<Option<(Reference, RefGeometry)>, DBError> {
    let mut result = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<ReferTo>() {
            Some(joinable) => {
                let results = joinable.get_all_results();
                let mut dist = std::f64::MAX;
                let mut index = 0;
                for ref_res in results {
                    if only_match.type_equals(&ref_res) {
                        let (cur_dist, ref_type) = ref_res.distance2(&guess);
                        if cur_dist < dist {
                            let which = Reference{id: *obj, index: ResultInd{index: index}, ref_type: ref_type};
                            result = Some((which, ref_res));
                            dist = cur_dist;
                        }
                    }
                    index += 1;
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(result)
}

fn get_closest_ref(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<(Option<ReferInd>), DBError> {
    let mut refer_ind = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<UpdateFromRefs>() {
            Some(joinable) => {
                let ref_num = joinable.get_num_refs();
                let mut dist = std::f64::MAX;
                for index in 0..ref_num {
                    if let Some(ref_geom) = joinable.get_associated_geom(ReferInd{index: index}) {
                        if only_match.type_equals(&ref_geom) {
                            let (cur_dist, _) = ref_geom.distance2(&guess);
                            if cur_dist < dist {
                                refer_ind = Some(ReferInd{index: index});
                                dist = cur_dist;
                            }
                        }
                    }
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(refer_ind)
}

fn snap_to_ref(file: &PathBuf, event: &UndoEventID, obj: &RefID, other_obj: &RefID, own_type: &RefType, only_match: &RefType, guess: &Point3f) -> Result<(), DBError> {
    let res_opt = get_closest_result(file, other_obj, only_match, guess)?;
    if let Some((which, calc_res)) = res_opt {
        let which_opt = get_closest_ref(file, obj, own_type, guess)?;
        if let Some(index) = which_opt {
            app_state::set_ref(file, event, obj, index, calc_res, which)?;
            return Ok(());
        }
    }
    return Err(DBError::NotFound);
}

pub fn snap_obj_to_other(file: PathBuf, event: &UndoEventID, obj: RefID, other_obj: &RefID, own_type: &RefType, only_match: &RefType, guess: &Point3f) -> Result<(), DBError> {
    snap_to_ref(&file, event, &obj, other_obj, own_type, only_match, guess)?;
    app_state::update_deps(file, obj);
    Ok(())
}

pub fn join_objs(file: PathBuf, event: &UndoEventID, first: RefID, second: RefID, first_type: &RefType, second_type: &RefType, guess: &Point3f) -> Result<(), DBError> {
    snap_to_ref(&file, event, &second, &first, second_type, first_type, guess)?;
    snap_to_ref(&file, event, &first, &second, first_type, second_type, guess)?;
    Ok(())
}
