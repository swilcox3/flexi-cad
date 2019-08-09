use crate::*;
use primitives::PrismOpening;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: UpdatablePoint,
    pub second_pt: UpdatablePoint,
    pub width: WorldCoord,
    pub height: WorldCoord,
    openings: Vec<UpdatableRect>,
    id: RefID
}

interfaces!(Wall: query_interface::ObjectClone, std::fmt::Debug, Data, ReferTo, HasRefs, Position, UpdateFromRefs);

impl Wall {
    pub fn new(id: RefID, first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        Wall {
            first_pt: UpdatablePoint::new(first),
            second_pt: UpdatablePoint::new(second),
            width: width,
            height: height,
            openings: Vec::new(),
            id: id
        }
    }
}

#[typetag::serde]
impl Data for Wall {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(json!({
                "type": "Wall",
                "Width": self.width,
                "Height": self.height,
            }))
        };
        let self_length = (self.second_pt.pt - self.first_pt.pt).magnitude();
        let mut sorted: Vec<PrismOpening> = self.openings.iter().map(|val| {
            let length = (val.pt_1 - self.first_pt.pt).magnitude();
            let interp = Interp::new(length / self_length);
            let height = val.pt_3.z - val.pt_2.z;
            PrismOpening{interp: interp, height: height, length: length}
        }).collect();
        sorted.sort_by(|first, second| first.interp.partial_cmp(&second.interp).unwrap());

        primitives::prism_with_openings(&self.first_pt.pt, &self.second_pt.pt, self.width, self.height, sorted, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &String) -> Result<serde_json::Value, DBError> {
        match prop_name.as_ref() {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "First" => serde_json::to_value(&self.first_pt).map_err(error_other),
            "Second" => serde_json::to_value(&self.second_pt).map_err(error_other),
            _ => Err(DBError::NotFound)
        }
    }

    fn set_data(&mut self, data: &serde_json::Value) -> Result<(), DBError> {
        let mut changed = false;
        if let serde_json::Value::Number(num) = &data["Width"] {
            changed = true;
            self.width = num.as_f64().unwrap();
        }
        if let serde_json::Value::Number(num) = &data["Height"] {
            changed = true;
            self.height = num.as_f64().unwrap();
        }
        if changed {
            Ok(())
        }
        else {
            Err(DBError::NotFound)
        }
    }
}

impl ReferTo for Wall {
    fn get_result(&self, result: ResultInd) -> Option<RefGeometry> {
        match result.index {
            0 => Some(RefGeometry::Point{pt: self.first_pt.pt}),
            1 => Some(RefGeometry::Point{pt: self.second_pt.pt}),
            2 => Some(RefGeometry::Line{pt_1: self.first_pt.pt, pt_2: self.second_pt.pt}),
            _ => None 
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point{pt: self.first_pt.pt});
        results.push(RefGeometry::Point{pt: self.second_pt.pt});
        results.push(RefGeometry::Line{pt_1: self.first_pt.pt, pt_2: self.second_pt.pt});
        results
    }
}

impl UpdateFromRefs for Wall {
    fn get_refs(&self) -> Vec<&UpdatableGeometry> {
        let mut results: Vec<&UpdatableGeometry> = Vec::new(); 
        results.push(&self.first_pt);
        results.push(&self.second_pt);
        for open in &self.openings {
            results.push(open);
        }
        results
    }

    fn set_ref(&mut self, refer: ReferInd, result: &RefGeometry, other_ref: Reference) {
        match refer.index {
            0 => {
                if let RefGeometry::Point{pt} = result {
                    self.first_pt.pt = *pt;
                }
                self.first_pt.refer = Some(other_ref);
            }
            1 => {
                if let RefGeometry::Point{pt} = result {
                    self.second_pt.pt = *pt;
                }
                self.second_pt.refer = Some(other_ref);
            }
            _ => {
                if let RefGeometry::Rect{pt_1, pt_2, pt_3} = result {
                    if let Some(open) = self.openings.get_mut(refer.index - 2) {
                        open.refer = Some(other_ref);
                    }
                    else {
                        let mut new_open = UpdatableRect::new(*pt_1, *pt_2, *pt_3);
                        new_open.refer = Some(other_ref);
                        self.openings.push(new_open);
                    }
                }
            }
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefGeometry>>) -> Result<UpdateMsg, DBError> {
        //std::thread::sleep(std::time::Duration::from_secs(1));
        if let Some(refer) = results.get(0) {
            self.first_pt.update(refer);
        }
        if let Some(refer) = results.get(1) {
            self.second_pt.update(refer);
        }
        let mut to_remove = Vec::new();
        for i in 2..results.len() {
            if let Some(refer) = results.get(i) {
                if let Some(open) = self.openings.get_mut(i - 2) {
                    open.update(refer);
                    if let None = open.refer {
                        to_remove.push(i);
                    }
                }
                else {
                    return Err(DBError::NotFound);
                }
            }
        }
        for index in to_remove.iter().rev() {
            self.openings.remove(*index);
        }
        self.update()
    }
}

impl HasRefs for Wall {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.first_pt.refer {
            deps.register_sub(&refer.id, self.id.clone());
        }
        if let Some(refer) = &self.second_pt.refer {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.first_pt.refer = None;
        self.second_pt.refer = None;
        self.openings.clear();
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt.pt += *delta;
        self.second_pt.pt += *delta;
    }
}



