use crate::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    id: RefID,
    wall_pt_1: ParametricPoint,
    wall_pt_2: ParametricPoint,
    interp: Interp,
    length: WorldCoord,
    pt_1: Point3f,
    pt_2: Point3f,
    width: WorldCoord,
    height: WorldCoord,
}

impl Door {
    pub fn new(first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Door {
        let id = RefID::new_v4();
        let length = (first - second).magnitude();
        Door {
            id: id,
            wall_pt_1: ParametricPoint::new(Point3f::new(0.0, 0.0, 0.0)),
            wall_pt_2: ParametricPoint::new(Point3f::new(0.0, 0.0, 0.0)),
            interp: Interp::new(0.0),
            length,
            pt_1: first,
            pt_2: second,
            width,
            height,
        }
    }

    pub fn set_dir(&mut self, dir: &Vector3f) {
        self.pt_2 = self.pt_1 + dir.normalize()*self.length;
    }
}

interfaces!(Door: dyn query_interface::ObjectClone, dyn std::fmt::Debug, dyn Data, dyn ReferTo, dyn Position, dyn UpdateFromRefs);

#[typetag::serde]
impl Data for Door {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(to_json("Door", &self))
        };
        let rotated = rotate_point_through_angle_2d(&self.pt_1, &self.pt_2, cgmath::Rad(std::f64::consts::FRAC_PI_4));
        primitives::rectangular_prism(&self.pt_1, &rotated, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }


    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: None
        };
        let rotated = rotate_point_through_angle_2d(&self.pt_1, &self.pt_2, cgmath::Rad(std::f64::consts::FRAC_PI_4));
        primitives::rectangular_prism(&self.pt_1, &rotated, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "Length" => Ok(json!(self.length)),
            "First" => serde_json::to_value(&self.pt_1).map_err(error_other),
            "Second" => serde_json::to_value(&self.pt_2).map_err(error_other),
            _ => Err(DBError::PropertyNotFound)
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
            Err(DBError::PropertyNotFound)
        }
    }
}

impl ReferTo for Door {
    fn get_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.pt_1),
            1 => Some(self.pt_2),
            2 => {
                let third = Point3f::new(self.pt_2.x, self.pt_2.y, self.pt_2.z + self.height);
                Some(third)
            }
            _ => None 
        }
    }

    fn get_all_points(&self) -> Vec<Point3f> {
        let mut results = Vec::new();
        results.push(self.pt_1);
        results.push(self.pt_2);
        let third = Point3f::new(self.pt_2.x, self.pt_2.y, self.pt_2.z + self.height);
        results.push(third);
        results
    }
}

impl UpdateFromRefs for Door {
    fn clear_refs(&mut self) {
        self.wall_pt_1.refer = None;
        self.wall_pt_2.refer = None;
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        vec![self.wall_pt_1.refer.clone(), self.wall_pt_2.refer.clone()]
    }

    fn set_ref(&mut self, index: PointIndex, result: Point3f, other_ref: Reference) {
        match index {
            0 => self.wall_pt_1.set_reference(result, other_ref),
            1 => self.wall_pt_2.set_reference(result, other_ref),
            _ => ()
        }
    }

    fn add_ref(&mut self, _: Point3f, _: Reference) -> bool {
        return false;
    }

    fn delete_ref(&mut self, index: PointIndex) {
        match index {
            0 => self.wall_pt_1.refer = None,
            1 => self.wall_pt_2.refer = None,
            _ => ()
        }
    }

    fn get_associated_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.wall_pt_1.pt),
            1 => Some(self.wall_pt_2.pt),
            _ => {
                None
            }
        }
    }

    fn set_associated_point(&mut self, index: PointIndex, geom: &Option<Point3f>) {
        match index {
            0 => self.wall_pt_1.update(geom),
            1 => self.wall_pt_2.update(geom),
            _ => ()
        }
    }
}

impl Position for Door {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.pt_1 += *delta;
        self.pt_2 += *delta;
    }
}



