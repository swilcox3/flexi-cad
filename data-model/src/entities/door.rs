use crate::*;
use serde::{Serialize, Deserialize};
use cgmath::InnerSpace;

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
                self.dir.set_reference(Some(other_ref));
                self.dir.update(&Some(*result));
            }
            _ => ()
        }
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



