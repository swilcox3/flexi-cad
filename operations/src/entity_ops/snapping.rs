use crate::prelude::*;

fn get_result(file: &PathBuf, obj: &RefID, index: ResultInd) -> Result<Option<RefGeometry>, DBError> {
    let mut res_opt = None;
    app_state::get_obj(file, obj, |read| match read.query_ref::<dyn ReferTo>() {
        Some(refer) => {
            res_opt = refer.get_result(index);
            Ok(())
        }
        None => Err(DBError::ObjLacksTrait),
    })?;
    Ok(res_opt)
}

pub fn get_closest_result(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<Option<(GeometryId, RefGeometry)>, DBError> {
    let mut result = None;
    app_state::get_obj(file, obj, |refer_obj| match refer_obj.query_ref::<dyn ReferTo>() {
        Some(joinable) => {
            let results = joinable.get_all_results();
            let mut dist = std::f64::MAX;
            let mut index = 0;
            for ref_res in results {
                if only_match.type_equals(&ref_res) {
                    let cur_dist = ref_res.distance2(&guess);
                    if cur_dist < dist {
                        let which = GeometryId { id: *obj, index };
                        result = Some((which, ref_res));
                        dist = cur_dist;
                    }
                }
                index += 1;
            }
            Ok(())
        }
        None => Err(DBError::ObjLacksTrait),
    })?;
    Ok(result)
}

fn get_closest_ref(file: &PathBuf, obj: &RefID, only_match: &RefType, guess: &Point3f) -> Result<(Option<ReferInd>), DBError> {
    let mut refer_ind = None;
    app_state::get_obj(file, obj, |refer_obj| match refer_obj.query_ref::<dyn UpdateFromRefs>() {
        Some(joinable) => {
            let indices = joinable.get_available_refs();
            let mut dist = std::f64::MAX;
            for index in indices {
                if let Some(ref_geom) = joinable.get_associated_geom(index) {
                    if only_match.type_equals(&ref_geom) {
                        let cur_dist = ref_geom.distance2(guess);
                        if cur_dist < dist {
                            refer_ind = Some(index);
                            dist = cur_dist;
                        }
                    }
                }
            }
            Ok(())
        }
        None => Err(DBError::ObjLacksTrait),
    })?;
    Ok(refer_ind)
}

pub fn snap_to_ref(
    file: &PathBuf,
    event: &UndoEventID,
    obj: &RefID,
    other_obj: &RefID,
    only_match: &RefType,
    guess: &Point3f,
) -> Result<(), DBError> {
    let res_opt = get_closest_result(file, other_obj, only_match, guess)?;
    if let Some((which, calc_res)) = res_opt {
        let which_opt = get_closest_ref(file, obj, only_match, guess)?;
        match which_opt {
            Some(index) => app_state::set_ref(file, event, obj, index, &calc_res, which, &Some(*guess))?,
            None => app_state::add_ref(file, event, obj, &calc_res, which, &Some(*guess))?,
        }
        Ok(())
    } else {
        Err(DBError::NotFound(String::from("Nothing to snap to")))
    }
}

pub fn join_refs(
    file: &PathBuf,
    event: &UndoEventID,
    first: &RefID,
    second: &RefID,
    first_wants: &RefType,
    second_wants: &RefType,
    guess: &Point3f,
) -> Result<(), DBError> {
    snap_to_ref(file, event, second, first, second_wants, guess)?;
    snap_to_ref(file, event, first, second, first_wants, guess)?;
    Ok(())
}
