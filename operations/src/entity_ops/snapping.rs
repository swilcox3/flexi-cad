use crate::prelude::*;

fn get_result(file: &PathBuf, obj: &RefID, index: PointIndex) -> Result<Option<Point3f>, DBError> {
    let mut res_opt = None;
    app_state::get_obj(file, obj, |read| {
        match read.query_ref::<dyn ReferTo>() {
            Some(refer) => {
                res_opt = refer.get_point(index);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(res_opt)
}

pub fn can_refer_to(file: &PathBuf, obj: &RefID) -> Result<bool, DBError> {
    let mut result = false;
    app_state::get_obj(file, obj, |refer_obj| {
        if let Some(_) = refer_obj.query_ref::<dyn ReferTo>() {
            result = true;
        }
        Ok(())
    })?;
    Ok(result)
}

pub fn get_closest_point(file: &PathBuf, obj: &RefID, guess: &Point3f) -> Result<Option<(GeometryId, Point3f)>, DBError> {
    let mut result = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<dyn ReferTo>() {
            Some(joinable) => {
                let results = joinable.get_all_points();
                let mut dist = std::f64::MAX;
                let mut index = 0;
                for ref_res in results {
                    let cur_dist = ref_res.distance2(*guess);
                    if cur_dist < dist {
                        let which = GeometryId{obj: *obj, index: index};
                        result = Some((which, ref_res));
                        dist = cur_dist;
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

fn get_closest_ref(file: &PathBuf, obj: &RefID, ref_obj: &RefID, guess: &Point3f) -> Result<(Option<PointIndex>), DBError> {
    let mut refer_ind = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<dyn UpdateFromRefs>() {
            Some(joinable) => {
                let refs = joinable.get_refs();
                let mut dist = std::f64::MAX;
                let mut index = 0;
                for ref_opt in refs {
                    let mut should_check = true;
                    if let Some(refer) = ref_opt {
                        if refer.obj != *ref_obj {
                            should_check = false;
                        }
                    }
                    if should_check {
                        if let Some(ref_geom) = joinable.get_associated_point(index) {
                            let cur_dist = ref_geom.distance2(*guess);
                            if cur_dist < dist {
                                refer_ind = Some(index);
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

pub fn snap_to_ref(file: &PathBuf, event: &UndoEventID, obj: &RefID, other_obj: &RefID, guess: &Point3f) -> Result<(), DBError> {
    let res_opt = get_closest_point(file, other_obj, guess)?;
    if let Some((which, calc_res)) = res_opt {
        let which_opt = get_closest_ref(file, obj, other_obj, guess)?;
        match which_opt {
            Some(index) => app_state::set_ref(file, event, obj, index, calc_res, which, &Some(*guess))?,
            None => app_state::add_ref(file, event, obj, calc_res, which, &Some(*guess))?
        }
        Ok(())
    }
    else {
        Err(DBError::NotFound(String::from("Nothing to snap to")))
    }
}

pub fn join_refs(file: &PathBuf, event: &UndoEventID, first: &RefID, second: &RefID, guess: &Point3f) -> Result<(), DBError> {
    snap_to_ref(file, event, second, first, guess)?;
    snap_to_ref(file, event, first, second, guess)?;
    Ok(())
}
