use crate::prelude::*;
use serde::{Deserialize, Serialize};

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

interfaces!(
    TestObj: dyn Store,
    dyn query_interface::ObjectClone,
    dyn std::fmt::Debug,
    dyn Data,
    dyn UpdateFromRefs,
    dyn Position,
    dyn ReferTo
);

impl TestObj {
    pub fn new(dat: &str) -> TestObj {
        TestObj {
            id: RefID::new_v4(),
            data: String::from(dat),
            point: UpdatableGeometry::new(RefPoint {
                pt: Point3f::new(0.0, 0.0, 0.0),
            }),
            point_2: UpdatableGeometry::new(RefPoint {
                pt: Point3f::new(1.0, 0.0, 0.0),
            }),
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
        Ok(UpdateMsg::Other {
            data: serde_json::to_value(&self).unwrap(),
        })
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        Ok(UpdateMsg::Other {
            data: serde_json::to_value(&self).unwrap(),
        })
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "data" => Ok(json!(self.data)),
            _ => Err(DBError::PropertyNotFound),
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
        match index {
            0 => Some(self.point.geom.get_geom()),
            1 => Some(self.point_2.geom.get_geom()),
            2 => Some(RefGeometry::Line {
                pt_1: self.point.geom.pt,
                pt_2: self.point_2.geom.pt,
            }),
            _ => None,
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point { pt: self.point.geom.pt });
        results.push(RefGeometry::Point { pt: self.point_2.geom.pt });
        results.push(RefGeometry::Line {
            pt_1: self.point.geom.pt,
            pt_2: self.point_2.geom.pt,
        });
        results
    }

    fn get_num_results(&self) -> usize {
        3
    }
}

impl UpdateFromRefs for TestObj {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        if let Some(id) = &self.point.refer {
            results.push(Some(Reference::new(self.id.clone(), 0, id.clone())));
        } else {
            results.push(None);
        }
        if let Some(id) = &self.point_2.refer {
            results.push(Some(Reference::new(self.id.clone(), 1, id.clone())));
        } else {
            results.push(None);
        }
        results
    }

    fn get_available_refs(&self) -> Vec<ReferInd> {
        let mut results = Vec::new();
        if let None = self.point.refer {
            results.push(0);
        }
        if let None = self.point_2.refer {
            results.push(1);
        }
        results
    }

    fn get_num_refs(&self) -> usize {
        2
    }

    fn clear_refs(&mut self) {
        self.point.refer = None;
        self.point_2.refer = None;
    }

    fn add_ref(&mut self, _: &RefGeometry, _: GeometryId, _: &Option<Point3f>) -> bool {
        false
    }

    fn set_ref(&mut self, index: ReferInd, result: &RefGeometry, other_ref: GeometryId, snap_pt: &Option<Point3f>) {
        match index {
            0 => self.point.set_reference(result, other_ref, snap_pt),
            1 => self.point_2.set_reference(result, other_ref, snap_pt),
            _ => (),
        }
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index {
            0 => self.point.refer = None,
            1 => self.point_2.refer = None,
            _ => (),
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index {
            0 => Some(self.point.geom.get_geom()),
            1 => Some(self.point_2.geom.get_geom()),
            _ => None,
        }
    }

    fn set_associated_geom(&mut self, index: ReferInd, geom: &Option<RefGeometry>) {
        match index {
            0 => self.point.update(geom),
            1 => self.point_2.update(geom),
            _ => (),
        }
    }
}

impl Position for TestObj {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.point.geom.pt += *delta;
        self.point_2.geom.pt += *delta;
    }
}
