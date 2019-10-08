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

pub fn get_closest_line(file: &PathBuf, obj: &RefID, guess: &Point3f) -> Result<Option<[(GeometryId, Point3f); 2]>, DBError> {
    let mut result = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<dyn ReferTo>() {
            Some(joinable) => {
                let results = joinable.get_all_points();
                let mut dist = std::f64::MAX;
                for index in 0..results.len() {
                    if let Some(pt_1) = results.get(index) {
                        let (pt_2, next_index) = match results.get(index + 1) {
                            Some(pt) => (pt, index + 1),
                            None => (&results[0], 0)
                        };
                        let projected = project_on_line(pt_1, pt_2, guess);
                        let cur_dist = projected.distance2(*guess);
                        if cur_dist < dist {
                            let which_1 = GeometryId{obj: *obj, index: index};
                            let which_2 = GeometryId{obj: *obj, index: next_index};
                            let arr = [(which_1, pt_1.clone()), (which_2, pt_2.clone())];
                            result = Some(arr);
                            dist = cur_dist;
                        }
                    }
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(result)
}

pub fn get_closest_rect(file: &PathBuf, obj: &RefID, guess: &Point3f) -> Result<Option<[(GeometryId, Point3f); 3]>, DBError> {
    let mut result = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<dyn ReferTo>() {
            Some(joinable) => {
                let results = joinable.get_all_points();
                let mut dist = std::f64::MAX;
                for index in 0..results.len() {
                    if let Some(pt_1) = results.get(index) {
                        let (pt_2, next_index) = match results.get(index + 1) {
                            Some(pt) => (pt, index + 1),
                            None => {
                                let ind = (index + 1) % results.len();
                                (&results[ind], ind)
                            }
                        };
                        let (pt_3, next_next_index) = match results.get(index + 2) {
                            Some(pt) => (pt, index + 2),
                            None => {
                                let ind = (index + 2) % results.len();
                                (&results[ind], ind)
                            }
                        };
                        let cur_dist = pt_1.distance2(*guess);
                        if cur_dist < dist {
                            let which_1 = GeometryId{obj: *obj, index: index};
                            let which_2 = GeometryId{obj: *obj, index: next_index};
                            let which_3 = GeometryId{obj: *obj, index: next_next_index};
                            let arr = [(which_1, pt_1.clone()), (which_2, pt_2.clone()), (which_3, pt_3.clone())];
                            result = Some(arr);
                            dist = cur_dist;
                        }
                    }
                }
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    Ok(result)
}

fn get_closest_ref(file: &PathBuf, obj: &RefID, point: &Point3f) -> Result<(Option<PointIndex>), DBError> {
    let mut refer_ind = None;
    app_state::get_obj(file, obj, |refer_obj| {
        match refer_obj.query_ref::<dyn UpdateFromRefs>() {
            Some(joinable) => {
                let indices = joinable.get_available_refs();
                let mut dist = std::f64::MAX;
                for index in indices {
                    if let Some(ref_geom) = joinable.get_associated_point(index) {
                        let cur_dist = ref_geom.distance2(*point);
                        if cur_dist < dist {
                            refer_ind = Some(index);
                            dist = cur_dist;
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

fn snap_ref(file: &PathBuf, event: &UndoEventID, obj: &RefID, refer: GeometryId, calc_point: Point3f, guess: &Point3f, add: bool) -> Result<(), DBError> {
    if !add {
        let ref_opt = get_closest_ref(file, obj, &calc_point)?;
        match ref_opt {
            Some(index) => app_state::set_ref(file, event, obj, index, calc_point, refer, &Some(*guess)),
            None => Err(DBError::NotFound(String::from("No available reference to set")))
        }
    }
    else {
        app_state::add_ref(file, event, obj, calc_point, refer, &Some(*guess))
    }
}

pub fn snap_to_point(file: &PathBuf, event: &UndoEventID, obj: &RefID, other_obj: &RefID, guess: &Point3f, add: bool) -> Result<(), DBError> {
    let res_opt = get_closest_point(file, other_obj, guess)?;
    if let Some((which, calc_res)) = res_opt {
        snap_ref(file, event, obj, which, calc_res, guess, add)
    }
    else {
        Err(DBError::NotFound(String::from("Nothing to snap to")))
    }
}

pub fn join_points(file: &PathBuf, event: &UndoEventID, first: &RefID, second: &RefID, guess: &Point3f, add: bool) -> Result<(), DBError> {
    snap_to_point(file, event, second, first, guess, add)?;
    snap_to_point(file, event, first, second, guess, add)?;
    Ok(())
}

pub fn snap_to_line(file: &PathBuf, event: &UndoEventID, obj: &RefID, other_obj: &RefID, guess: &Point3f, add: bool) -> Result<(), DBError> {
    let res_opt = get_closest_line(file, other_obj, guess)?;
    if let Some(arr) = res_opt {
        snap_ref(file, event, obj, arr[0].0.clone(), arr[0].1, guess, add)?;
        snap_ref(file, event, obj, arr[1].0.clone(), arr[1].1, guess, add)
    }
    else {
        Err(DBError::NotFound(String::from("Nothing to snap to")))
    }
}

pub fn snap_to_rect(file: &PathBuf, event: &UndoEventID, obj: &RefID, other_obj: &RefID, guess: &Point3f, add: bool) -> Result<(), DBError> {
    let res_opt = get_closest_rect(file, other_obj, guess)?;
    if let Some(arr) = res_opt {
        snap_ref(file, event, obj, arr[0].0.clone(), arr[0].1, guess, add)?;
        snap_ref(file, event, obj, arr[1].0.clone(), arr[1].1, guess, add)?;
        snap_ref(file, event, obj, arr[2].0.clone(), arr[2].1, guess, add)
    }
    else {
        Err(DBError::NotFound(String::from("Nothing to snap to")))
    }
}
