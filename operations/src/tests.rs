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
    point: UpdatableGeometry<RefPoint>,
    point_2: UpdatableGeometry<RefPoint>,
}

interfaces!(TestObj: Store, query_interface::ObjectClone, std::fmt::Debug, Data, UpdateFromRefs, Position, ReferTo);

impl TestObj {
    pub fn new(dat: &str) -> TestObj {
        TestObj { 
            id: RefID::new_v4(), 
            data: String::from(dat), 
            point: UpdatableGeometry::new(RefPoint{pt: Point3f::new(0.0, 0.0, 0.0)}),
            point_2: UpdatableGeometry::new(RefPoint{pt: Point3f::new(1.0, 0.0, 0.0)}),
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
    fn get_result(&self, index: ResultInd) -> Option<RefGeometry> {
        match index.index {
            0 => Some(self.point.geom.get_geom()),
            1 => Some(self.point_2.geom.get_geom()),
            2 => Some(RefGeometry::Line{pt_1: self.point.geom.pt, pt_2: self.point_2.geom.pt}),
            _ => None
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point{pt: self.point.geom.pt});
        results.push(RefGeometry::Point{pt: self.point_2.geom.pt});
        results.push(RefGeometry::Line{pt_1: self.point.geom.pt, pt_2: self.point_2.geom.pt});
        results
    }
}

impl UpdateFromRefs for TestObj {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        results.push(self.point.refer.clone());
        results.push(self.point_2.refer.clone());
        results
    }

    fn clear_refs(&mut self) {
        self.point.refer = None;
        self.point_2.refer = None;
    }

    fn add_ref(&mut self, _: &RefGeometry, _: Reference, _: &Option<Point3f>) -> bool {
        false
    }

    fn set_ref(&mut self, index: ReferInd, result: &RefGeometry, other_ref: Reference, snap_pt: &Option<Point3f>) {
        match index.index {
            0 => self.point.set_reference(result, other_ref, snap_pt),
            1 => self.point_2.set_reference(result, other_ref, snap_pt),
            _ => ()
        }
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index.index {
            0 => self.point.refer = None,
            1 => self.point_2.refer = None,
            _ => ()
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index.index {
            0 => Some(self.point.geom.get_geom()),
            1 => Some(self.point_2.geom.get_geom()),
            _ => None
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefGeometry>>) {
        if let Some(geom) = results.get(0) {
            self.point.update(geom);
        }
        if let Some(geom) = results.get(1) {
            self.point_2.update(geom);
        }
    }
}

impl Position for TestObj {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.point.geom.pt += *delta;
        self.point_2.geom.pt += *delta;
    }
}
