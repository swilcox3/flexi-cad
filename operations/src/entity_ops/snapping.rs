use crate::*;

fn get_closest_result(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<(Option<Reference>, Option<RefResult>), DBError> {
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
                            which_opt = Some(Reference{id: *obj, index: index, ref_type: ref_type});
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

pub fn snap_to(file: PathBuf, event: &UndoEventID, obj: RefID, index: RefIndex, other_obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<Option<RefResult>, DBError> {
    let (which_opt, res_opt) = get_closest_result(&file, &other_obj, only_match, guess)?;
    if let Some(which) = which_opt {
        if let Some(calc_res) = &res_opt {
            app_state::set_ref(&file, event, &obj, index, calc_res, which)?;
            app_state::update_deps(file, obj);
            return Ok(res_opt);
        }
    }
    return Err(DBError::NotFound);
}

pub fn join_at(file: PathBuf, event: &UndoEventID, first: RefID, second: RefID, first_type: &RefType, second_type: &RefType, guess: &Point3f) -> Result<(), DBError> {
    let (which_opt_1, res_opt_1) = get_closest_result(&file, &first, first_type, guess)?;
    let (which_opt_2, res_opt_2) = get_closest_result(&file, &second, second_type, guess)?;
    if let Some(which_1) = which_opt_1 {
        if let Some(res_1) = res_opt_1 {
            if let Some(which_2) = which_opt_2 {
                if let Some(_) = res_opt_2 {
                    let other_index = which_1.index;
                    app_state::set_ref(&file, event, &second, which_2.index, &res_1, which_1)?;
                    //This is weird, but we need the index to set above, but we want to only move the second object passed in,
                    //updating it to the position of the first one.
                    let (which_opt_2, res_opt_2) = get_closest_result(&file, &second, second_type, guess)?;
                    if let Some(which_2) = which_opt_2 {
                        if let Some(res_2) = res_opt_2 {
                            app_state::set_ref(&file, event, &first, other_index, &res_2, which_2)?;
                        }
                    }
                    app_state::update_all_deps(file, vec![first, second]);
                    return Ok(());
                }
            }
        }
    }
    return Err(DBError::NotFound);
}
