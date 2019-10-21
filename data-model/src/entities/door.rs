use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    id: RefID,
    pub dir: UpdatableGeometry<RefLineSeg>,
    pub width: WorldCoord,
    pub height: WorldCoord,
}

impl Door {
    pub fn new(first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Door {
        let id = RefID::new_v4();
        Door {
            id: id,
            dir: UpdatableGeometry::new(RefLineSeg::new(first, second)),
            width: width,
            height: height,
        }
    }
}

interfaces!(
    Door: dyn query_interface::ObjectClone,
    dyn std::fmt::Debug,
    dyn Data,
    dyn ReferTo,
    dyn Position,
    dyn UpdateFromRefs
);

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
            metadata: Some(json!({
                "type": "Door",
                "traits": ["ReferTo", "Position", "UpdateFromRefs"],
                "obj": {
                    "Width": self.width,
                    "Height": self.height,
                    "First": self.dir.geom.pt_1,
                    "Second": self.dir.geom.pt_2
                }
            })),
        };
        let rotated = rotate_point_through_angle_2d(
            &self.dir.geom.pt_1,
            &self.dir.geom.pt_2,
            cgmath::Rad(std::f64::consts::FRAC_PI_4),
        );
        primitives::rectangular_prism(
            &self.dir.geom.pt_1,
            &rotated,
            self.width,
            self.height,
            &mut data,
        );
        Ok(UpdateMsg::Mesh { data: data })
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: None,
        };
        let rotated = rotate_point_through_angle_2d(
            &self.dir.geom.pt_1,
            &self.dir.geom.pt_2,
            cgmath::Rad(std::f64::consts::FRAC_PI_4),
        );
        primitives::rectangular_prism(
            &self.dir.geom.pt_1,
            &rotated,
            self.width,
            self.height,
            &mut data,
        );
        Ok(UpdateMsg::Mesh { data: data })
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "Length" => Ok(json!(self.dir.geom.length)),
            "First" => serde_json::to_value(&self.dir.geom.pt_1).map_err(error_other),
            "Second" => serde_json::to_value(&self.dir.geom.pt_2).map_err(error_other),
            _ => Err(DBError::PropertyNotFound),
        }
    }

    fn set_data(&mut self, data: serde_json::Value) -> Result<(), DBError> {
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
        } else {
            Err(DBError::PropertyNotFound)
        }
    }
}

impl ReferTo for Door {
    fn get_result(&self, res: ResultInd) -> Option<RefGeometry> {
        match res {
            0 => Some(RefGeometry::Point {
                pt: self.dir.geom.pt_1,
            }),
            1 => Some(RefGeometry::Point {
                pt: self.dir.geom.pt_2,
            }),
            2 => {
                let third = Point3f::new(
                    self.dir.geom.pt_2.x,
                    self.dir.geom.pt_2.y,
                    self.dir.geom.pt_2.z + self.height,
                );
                Some(RefGeometry::Rect {
                    pt_1: self.dir.geom.pt_1,
                    pt_2: self.dir.geom.pt_2,
                    pt_3: third,
                })
            }
            _ => None,
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point {
            pt: self.dir.geom.pt_1,
        });
        results.push(RefGeometry::Point {
            pt: self.dir.geom.pt_2,
        });
        let third = Point3f::new(
            self.dir.geom.pt_2.x,
            self.dir.geom.pt_2.y,
            self.dir.geom.pt_2.z + self.height,
        );
        results.push(RefGeometry::Rect {
            pt_1: self.dir.geom.pt_1,
            pt_2: self.dir.geom.pt_2,
            pt_3: third,
        });
        results
    }

    fn get_num_results(&self) -> usize {
        3
    }
}

impl UpdateFromRefs for Door {
    fn clear_refs(&mut self) {
        self.dir.refer = None;
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        if let Some(id) = &self.dir.refer {
            results.push(Some(Reference::new(self.id.clone(), 0, id.clone())));
        } else {
            results.push(None);
        }
        let self_id_0 = GeometryId::new(self.id.clone(), 0);
        let self_id_1 = GeometryId::new(self.id.clone(), 1);
        let self_id_2 = GeometryId::new(self.id.clone(), 2);
        results.push(Some(Reference {
            owner: self_id_2.clone(),
            other: self_id_0,
        }));
        results.push(Some(Reference {
            owner: self_id_2,
            other: self_id_1,
        }));
        results
    }

    fn get_available_refs(&self) -> Vec<ReferInd> {
        let mut results = Vec::new();
        if let None = self.dir.refer {
            results.push(0);
        }
        results
    }

    fn get_num_refs(&self) -> usize {
        1
    }

    fn set_ref(
        &mut self,
        index: ReferInd,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) {
        match index {
            0 => self.dir.set_reference(result, other_ref, snap_pt),
            _ => (),
        }
    }

    fn add_ref(&mut self, _: &RefGeometry, _: GeometryId, _: &Option<Point3f>) -> bool {
        return false;
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index {
            0 => self.dir.refer = None,
            _ => (),
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index {
            0 => Some(self.dir.geom.get_geom()),
            _ => None,
        }
    }

    fn set_associated_geom(&mut self, index: ReferInd, geom: &Option<RefGeometry>) {
        match index {
            0 => self.dir.update(geom),
            _ => (),
        }
    }
}

impl Position for Door {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.dir.geom.pt_1 += *delta;
        self.dir.geom.pt_2 += *delta;
    }
}
