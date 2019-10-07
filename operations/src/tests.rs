use crate::prelude::*;
use serde::{Serialize, Deserialize};

pub trait Store: Send + Sync {
    fn set_store_data(&mut self, data: String);
    fn get_store_data(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestObj {
    id: RefID,
    data: String,
    point: ParametricPoint,
    point_2: ParametricPoint,
}

interfaces!(TestObj: dyn Store, dyn query_interface::ObjectClone, dyn std::fmt::Debug, dyn Data, dyn UpdateFromRefs, dyn Position, dyn ReferTo);

impl TestObj {
    pub fn new(dat: &str) -> TestObj {
        TestObj { 
            id: RefID::new_v4(), 
            data: String::from(dat), 
            point: ParametricPoint::new(Point3f::new(0.0, 0.0, 0.0)),
            point_2: ParametricPoint::new(Point3f::new(1.0, 0.0, 0.0)),
        }
    }
}

#[typetag::serde]
impl Data for TestObj {
    fn get_id(&self) -> &RefID {
        return &self.id;
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        Ok(UpdateMsg::Other{data: serde_json::to_value(&self).unwrap()})
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        Ok(UpdateMsg::Other{data: serde_json::to_value(&self).unwrap()})
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "data" => Ok(json!(self.data)),
            _ => Err(DBError::PropertyNotFound)
        }
    }

    fn set_data(&mut self, _data: &serde_json::Value) -> Result<(), DBError> {
        Ok(())
    }
}

impl Store for TestObj {
    fn set_store_data(&mut self, data: String) {
        self.data = data;
    }
    fn get_store_data(&self) -> String {
        self.data.clone()
    }
}

impl ReferTo for TestObj {
    fn get_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.point.pt),
            1 => Some(self.point_2.pt),
            _ => None
        }
    }

    fn get_all_points(&self) -> Vec<Point3f> {
        let mut results = Vec::new();
        results.push(self.point.pt);
        results.push(self.point_2.pt);
        results
    }

    fn get_num_points(&self) -> usize {
        2
    }
}

impl UpdateFromRefs for TestObj {
    fn get_refs(&self) -> Vec<Option<GeometryId>> {
        let mut results = Vec::new();
        results.push(self.point.refer.clone());
        results.push(self.point_2.refer.clone());
        results
    }

    fn get_num_refs(&self) -> usize {
        2
    }

    fn clear_refs(&mut self) {
        self.point.refer = None;
        self.point_2.refer = None;
    }

    fn add_ref(&mut self, _: Point3f, _: GeometryId) -> bool {
        false
    }

    fn set_ref(&mut self, index: PointIndex, result: Point3f, other_ref: GeometryId) {
        match index {
            0 => self.point.set_reference(result, other_ref),
            1 => self.point_2.set_reference(result, other_ref),
            _ => ()
        }
    }

    fn delete_ref(&mut self, index: PointIndex) {
        match index {
            0 => self.point.refer = None,
            1 => self.point_2.refer = None,
            _ => ()
        }
    }

    fn get_associated_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.point.pt),
            1 => Some(self.point_2.pt),
            _ => None
        }
    }

    fn set_associated_point(&mut self, index: PointIndex, geom: Option<Point3f>) {
        match index {
            0 => self.point.update(geom),
            1 => self.point.update(geom),
            _ => ()
        }
    }
}

impl Position for TestObj {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.point.pt += *delta;
        self.point_2.pt += *delta;
    }
}
