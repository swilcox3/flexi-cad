use crate::*;
use serde::{Serialize, Deserialize};
use cgmath::InnerSpace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefLineSeg {
    pt_1: Point3f,
    pt_2: Point3f,
    length: WorldCoord,
    interp: Interp
}

impl RefLineSeg {
    fn new(pt_1: Point3f, pt_2: Point3f) -> RefLineSeg {
        let length = (pt_2 - pt_1).magnitude();
        RefLineSeg {
            pt_1: pt_1, 
            pt_2: pt_2,
            length: length,
            interp: Interp::new(0.0)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    id: RefID,
    pub dir: UpdatableGeometry<RefLineSeg>,
    pub width: WorldCoord,
    pub height: WorldCoord,
}

impl Door {
    pub fn new(id: RefID, first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Door {
        Door {
            id: id,
            dir: UpdatableGeometry::new(RefLineSeg::new(first, second)),
            width: width,
            height: height,
        }
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
                "Length": self.dir.geom.length
            }))
        };
        primitives::rectangular_prism(&self.dir.geom.pt_1, &self.dir.geom.pt_2, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &String) -> Result<serde_json::Value, DBError> {
        match prop_name.as_ref() {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "Length" => Ok(json!(self.dir.geom.length)),
            "First" => serde_json::to_value(&self.dir.geom.pt_1).map_err(error_other),
            "Second" => serde_json::to_value(&self.dir.geom.pt_2).map_err(error_other),
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
            self.dir.geom.length = num.as_f64().unwrap();
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
    fn get_result(&self, res: ResultInd) -> Option<RefGeometry> {
        match res.index {
            0 => Some(RefGeometry::Point{pt: self.dir.geom.pt_1}),
            1 => Some(RefGeometry::Point{pt: self.dir.geom.pt_2}),
            2 => {
                let third = Point3f::new(self.dir.geom.pt_2.x, self.dir.geom.pt_2.y, self.dir.geom.pt_2.z + self.height);
                Some(RefGeometry::Rect{pt_1: self.dir.geom.pt_1, pt_2: self.dir.geom.pt_2, pt_3: third})
            }
            _ => None 
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point{pt: self.dir.geom.pt_1});
        results.push(RefGeometry::Point{pt: self.dir.geom.pt_2});
        let third = Point3f::new(self.dir.geom.pt_2.x, self.dir.geom.pt_2.y, self.dir.geom.pt_2.z + self.height);
        results.push(RefGeometry::Rect{pt_1: self.dir.geom.pt_1, pt_2: self.dir.geom.pt_2, pt_3: third});
        results
    }
}

impl UpdateFromRefs for Door {
    fn get_refs(&self) -> Vec<UpdatableGeometry<RefLineSeg>> {
        let mut results = Vec::new(); 
        results.push(self.dir.clone());
        results
    }

    fn set_ref(&mut self, refer: ReferInd, result: &RefGeometry, other_ref: Reference) {
        match refer.index {
            0 => {
                if let RefGeometry::Line{pt_1, pt_2} = result {
                    if let RefType::Line{interp} = other_ref.ref_type {
                        let dir = pt_2 - pt_1;
                        self.dir.geom.pt_1 = pt_1 + dir * interp.val();
                        self.dir.geom.pt_2 = self.dir.geom.pt_1 + dir.normalize() * self.dir.geom.length;
                        self.dir.refer = Some(other_ref);
                    }
                }
            }
            _ => ()
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefGeometry>>) -> Result<UpdateMsg, DBError> {
        //std::thread::sleep(std::time::Duration::from_secs(1));
        if let Some(refer) = results.get(0) {
            if let Some(RefGeometry::Line{pt_1, pt_2}) = refer {
                if let Some(own_refer) = &self.dir.refer {
                    if let RefType::Line{interp} = own_refer.ref_type {
                        let dir = pt_2 - pt_1;
                        self.dir.geom.pt_1 = pt_1 + dir * interp.val();
                        self.dir.geom.pt_2 = self.dir.geom.pt_1 + dir.normalize() * self.dir.geom.length;
                    }
                }
            }
            else {
                self.dir.refer = None;
            }
        }
        self.update()
    }
}

impl HasRefs for Door {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.dir.refer {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.dir.refer = None;
    }
}

impl Position for Door {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.dir.geom.pt_1 += *delta;
        self.dir.geom.pt_2 += *delta;
    }
}



