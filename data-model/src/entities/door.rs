use crate::*;
use serde::{Serialize, Deserialize};
use cgmath::InnerSpace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    id: RefID,
    pub first_pt: Point3f,
    pub second_pt: Point3f,
    pub width: WorldCoord,
    pub height: WorldCoord,
    pub length: WorldCoord,
    line_ref: Option<Reference>
}

impl Door {
    pub fn new(id: RefID, first_pt: Point3f, second_pt: Point3f, width: WorldCoord, height: WorldCoord, length: WorldCoord) -> Door {
        Door {
            id: id,
            first_pt: first_pt,
            second_pt: second_pt,
            width: width,
            height: height,
            length: length,
            line_ref: None
        }
    }

    pub fn set_dir(&mut self, dir: &Vector3f) {
        let norm = dir.normalize();
        let offset = norm * self.length;
        self.second_pt.x = self.first_pt.x + offset.x;
        self.second_pt.y = self.first_pt.y + offset.y;
        self.second_pt.z = self.first_pt.z + offset.z;
    }
}

interfaces!(Door: query_interface::ObjectClone, std::fmt::Debug, Data, ReferTo, HasRefs, Position, UpdateFromRefs);

#[typetag::serde]
impl Data for Door {
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
                "type": "Door",
                "Width": self.width,
                "Height": self.height,
                "Length": self.length
            }))
        };
        primitives::rectangular_prism(&self.first_pt, &self.second_pt, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &String) -> Result<serde_json::Value, DBError> {
        match prop_name.as_ref() {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "Length" => Ok(json!(self.length)),
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
        if let serde_json::Value::Number(num) = &data["Length"] {
            changed = true;
            self.length = num.as_f64().unwrap();
        }
        if changed {
            Ok(())
        }
        else {
            Err(DBError::NotFound)
        }
    }
}

impl ReferTo for Door {
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

impl UpdateFromRefs for Door {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new(); 
        results.push(self.line_ref.clone());
        results
    }

    fn set_ref(&mut self, which_self: &RefType, result: &RefResult, other_ref: Reference) {
        match which_self {
            RefType::Line{..} => {
                match result {
                    RefResult::Line{pts} => {
                        let other_dir = pts.1 - pts.0;
                        let proj_1 = (self.first_pt - ORIGIN).project_on(other_dir);
                        let proj_2 = (self.second_pt - ORIGIN).project_on(other_dir);
                        self.first_pt = Point3f::new(proj_1.x, proj_1.y, proj_1.z);
                        self.second_pt = Point3f::new(proj_2.x, proj_2.y, proj_2.z);
                        self.line_ref = Some(other_ref);
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
            if let Some(RefResult::Line{pts}) = refer {
                let other_dir = pts.1 - pts.0;
                let proj_1 = (self.first_pt - ORIGIN).project_on(other_dir);
                let proj_2 = (self.second_pt - ORIGIN).project_on(other_dir);
                self.first_pt = Point3f::new(proj_1.x, proj_1.y, proj_1.z);
                self.second_pt = Point3f::new(proj_2.x, proj_2.y, proj_2.z);
            }
            else {
                self.line_ref = None;
            }
        }
        self.update()
    }
}

impl HasRefs for Door {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.line_ref {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.line_ref = None;
    }
}

impl Position for Door {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt += *delta;
        self.second_pt += *delta;
    }
}



