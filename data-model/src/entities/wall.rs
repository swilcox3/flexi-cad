use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: Point3f,
    pub second_pt: Point3f,
    pub width: WorldCoord,
    pub height: WorldCoord,
    joined_first: Option<Reference>,
    joined_second: Option<Reference>,
    openings: Vec<Reference>,
    id: RefID
}

interfaces!(Wall: query_interface::ObjectClone, std::fmt::Debug, Data, ReferTo, HasRefs, Position, UpdateFromRefs);

impl Wall {
    pub fn new(id: RefID, first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        Wall {
            first_pt: first,
            second_pt: second,
            width: width,
            height: height,
            joined_first: None,
            joined_second: None,
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
                "width": self.width,
                "height": self.height,
            }))
        };
        primitives::rectangular_prism(&self.first_pt, &self.second_pt, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
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
    fn get_result(&self, which: &RefType) -> Option<RefResult> {
        match which {
            RefType::Point{which_pt} => {
                match which_pt {
                    0 => Some(RefResult::Point{pt: self.first_pt}),
                    1 => Some(RefResult::Point{pt: self.second_pt}),
                    _ => None 
                }
            }
            RefType::Line{pts} => {
                let pt_1_opt = match pts.0 {
                    0 => Some(self.first_pt),
                    1 => Some(self.second_pt),
                    _ => None
                };
                let pt_2_opt = match pts.1 {
                    0 => Some(self.first_pt),
                    1 => Some(self.second_pt),
                    _ => None
                };
                if let Some(pt_1) = pt_1_opt {
                    if let Some(pt_2) = pt_2_opt {
                        Some(RefResult::Line{pts: (pt_1, pt_2)})
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                }
            }
            _ => None
        }
    }

    fn get_results_for_type(&self, which: &RefType) -> Vec<RefResult> {
        let mut results = Vec::new();
        match which {
            RefType::Point{..} => {
                results.push(RefResult::Point{pt: self.first_pt});
                results.push(RefResult::Point{pt: self.second_pt});
            }
            RefType::Line{pts} => {
                let pt_1_opt = match pts.0 {
                    0 => Some(self.first_pt),
                    1 => Some(self.second_pt),
                    _ => None
                };
                let pt_2_opt = match pts.1 {
                    0 => Some(self.first_pt),
                    1 => Some(self.second_pt),
                    _ => None
                };
                if let Some(pt_1) = pt_1_opt {
                    if let Some(pt_2) = pt_2_opt {
                        results.push(RefResult::Line{pts: (pt_1, pt_2)});
                    }
                }
            }
            _ => ()
        }
        results
    }
}

impl UpdateFromRefs for Wall {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new(); 
        results.push(self.joined_first.clone());
        results.push(self.joined_second.clone());
        results
    }

    fn set_ref(&mut self, which_self: &RefType, result: &RefResult, other_ref: Reference) {
        match which_self {
            RefType::Point{which_pt} => {
                match which_pt {
                    0 => {
                        if let RefResult::Point{pt} = result {
                            self.first_pt = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.joined_first = Some(other_ref);
                        }
                    }
                    1 => {
                        if let RefResult::Point{pt} = result {
                            self.second_pt = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.joined_second = Some(other_ref);
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefResult>>) -> Result<UpdateMsg, DBError> {
        //std::thread::sleep(std::time::Duration::from_secs(1));
        if let Some(refer) = results.get(0) {
            if let Some(RefResult::Point{pt}) = refer {
                self.first_pt = *pt;
            }
            else {
                self.joined_first = None;
            }
        }
        if let Some(refer) = results.get(1) {
            if let Some(RefResult::Point{pt}) = refer {
                self.second_pt = *pt;
            }
            else {
                self.joined_second = None;
            }
        }
        self.update()
    }
}

impl HasRefs for Wall {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.joined_first {
            deps.register_sub(&refer.id, self.id.clone());
        }
        if let Some(refer) = &self.joined_second {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.joined_first = None;
        self.joined_second = None;
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt += *delta;
        self.second_pt += *delta;
    }
}



