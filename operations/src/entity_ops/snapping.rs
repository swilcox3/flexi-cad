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

fn get_closest_result(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<(Option<Reference>, Option<RefGeometry>), DBError> {
    let mut which_opt = None;
    let mut res_opt = None;
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
                            res_opt = Some(ref_res);
                            which_opt = Some(Reference{id: *obj, index: ResultInd{index: index}, ref_type: ref_type});
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
    Ok((which_opt, res_opt))
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
                    index += 1;
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(refer_ind)
}

pub fn snap_to(file: PathBuf, event: &UndoEventID, obj: RefID, index: ReferInd, other_obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<Option<RefGeometry>, DBError> {
    let (which_opt, res_opt) = get_closest_result(&file, &other_obj, only_match, guess)?;
    if let Some(which) = which_opt {
        if let Some(calc_res) = &res_opt {
            app_state::set_ref(&file, event, &obj, index, *calc_res, which)?;
            app_state::update_deps(file, obj);
            return Ok(res_opt);
        }
    }
    return Err(DBError::NotFound);
}

pub fn join_at(file: PathBuf, event: &UndoEventID, first: RefID, second: RefID, first_type: &RefType, second_type: &RefType, guess: &Point3f) -> Result<(), DBError> {
    let (which_opt_1, res_opt_1) = get_closest_result(&file, &first, first_type, guess)?;
    let which_opt_2 = get_closest_ref(&file, &second, second_type, guess)?;
    if let Some(ref_to_set_on_2) = which_opt_1 {
        if let Some(res_from_1) = res_opt_1 {
            if let Some(second_ref_index) = which_opt_2 {
                app_state::set_ref(&file, event, &second, second_ref_index, &res_from_1, ref_to_set_on_2)?;
                let res_opt_2 = get_result(&file, &second, other_index)?;
                if let Some(res_2) = res_opt_2 {
                    app_state::set_ref(&file, event, &first, other_index, &res_2, which_2)?;
                }
                app_state::update_all_deps(file, vec![first, second]);
                return Ok(());
            }
        }
    }
    return Err(DBError::NotFound);
}
