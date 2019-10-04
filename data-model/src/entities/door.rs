use crate::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    id: RefID,
    pub dir: Geometry<RefLineSeg>,
    pub width: WorldCoord,
    pub height: WorldCoord,
}

impl Door {
    pub fn new(first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Door {
        let id = RefID::new_v4();
        Door {
            id: id,
            dir: Geometry::new(RefLineSeg::new(first, second)),
            width: width,
            height: height,
        }
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
        let rotated = rotate_point_through_angle_2d(&self.dir.geom.pt_1, &self.dir.geom.pt_2, cgmath::Rad(std::f64::consts::FRAC_PI_4));
        primitives::rectangular_prism(&self.dir.geom.pt_1, &rotated, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }


    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: None
        };
        let rotated = rotate_point_through_angle_2d(&self.dir.geom.pt_1, &self.dir.geom.pt_2, cgmath::Rad(std::f64::consts::FRAC_PI_4));
        primitives::rectangular_prism(&self.dir.geom.pt_1, &rotated, self.width, self.height, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "Length" => Ok(json!(self.dir.geom.length)),
            "First" => serde_json::to_value(&self.dir.geom.pt_1).map_err(error_other),
            "Second" => serde_json::to_value(&self.dir.geom.pt_2).map_err(error_other),
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
            self.dir.geom.length = num.as_f64().unwrap();
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
    fn get_point(&self, res: PointIndex) -> Option<Point3f> {
        match res.index {
            0 => Some(Point3f::Point{pt: self.dir.geom.pt_1}),
            1 => Some(Point3f::Point{pt: self.dir.geom.pt_2}),
            2 => {
                let third = Point3f::new(self.dir.geom.pt_2.x, self.dir.geom.pt_2.y, self.dir.geom.pt_2.z + self.height);
                Some(Point3f::Rect{pt_1: self.dir.geom.pt_1, pt_2: self.dir.geom.pt_2, pt_3: third})
            }
            _ => None 
        }
    }

    fn get_all_points(&self) -> Vec<Point3f> {
        let mut results = Vec::new();
        results.push(Point3f::Point{pt: self.dir.geom.pt_1});
        results.push(Point3f::Point{pt: self.dir.geom.pt_2});
        let third = Point3f::new(self.dir.geom.pt_2.x, self.dir.geom.pt_2.y, self.dir.geom.pt_2.z + self.height);
        results.push(Point3f::Rect{pt_1: self.dir.geom.pt_1, pt_2: self.dir.geom.pt_2, pt_3: third});
        results
    }
}

impl UpdateFromRefs for Door {
    fn clear_refs(&mut self) {
        self.dir.refer = None;
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        vec![self.dir.refer.clone()]
    }

    fn set_ref(&mut self, index: PointIndex, result: &Point3f, other_ref: Reference, snap_pt: &Option<Point3f>) {
        match index {
            0 => self.dir.set_reference(result, other_ref, snap_pt),
            _ => ()
        }
    }

    fn add_ref(&mut self, _: &Point3f, _: Reference, _: &Option<Point3f>) -> bool {
        return false;
    }

    fn delete_ref(&mut self, index: PointIndex) {
        match index {
            0 => self.dir.refer = None,
            _ => ()
        }
    }

    fn get_associated_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.dir.pt),
            _ => {
                None
            }
        }
    }

    fn set_associated_point(&mut self, index: PointIndex, geom: &Option<Point3f>) {
        match index {
            0 => self.dir.update(geom),
            _ => ()
        }
    }
}

impl Position for Door {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.dir.geom.pt_1 += *delta;
        self.dir.geom.pt_2 += *delta;
    }
}



