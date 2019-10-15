use crate::*;
use primitives::PrismOpening;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: UpdatableGeometry<RefPoint>,
    pub second_pt: UpdatableGeometry<RefPoint>,
    pub width: WorldCoord,
    pub height: WorldCoord,
    openings: Vec<UpdatableGeometry<RefRect>>,
    #[serde(skip_serializing, default = "String::new")]
    data: String,
    id: RefID,
}

interfaces!(
    Wall: dyn query_interface::ObjectClone,
    dyn std::fmt::Debug,
    dyn Data,
    dyn ReferTo,
    dyn Position,
    dyn UpdateFromRefs
);

impl Wall {
    pub fn new(first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        let id = RefID::new_v4();
        Wall {
            first_pt: UpdatableGeometry::new(RefPoint { pt: first }),
            second_pt: UpdatableGeometry::new(RefPoint { pt: second }),
            width: width,
            height: height,
            openings: Vec::new(),
            data: String::new(),
            id: id,
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

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        if self.data.len() == 0 {
            for i in 0..1000000 {
                self.data += &i.to_string();
            }
        }
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(to_json(
                "Wall",
                &["ReferTo", "Position", "UpdateFromRefs"],
                &self,
            )),
        };
        let self_length = (self.second_pt.geom.pt - self.first_pt.geom.pt).magnitude();
        self.openings.retain(|open| open.refer != None);
        let mut sorted: Vec<PrismOpening> = self
            .openings
            .iter()
            .map(|val| {
                let position = (val.geom.pt_1 - self.first_pt.geom.pt).magnitude();
                let interp = Interp::new(position / self_length);
                let length = (val.geom.pt_2 - val.geom.pt_1).magnitude();
                let height = val.geom.pt_3.z - val.geom.pt_2.z;
                PrismOpening {
                    interp: interp,
                    height: height,
                    length: length,
                }
            })
            .collect();
        sorted.sort_by(|first, second| first.interp.partial_cmp(&second.interp).unwrap());

        primitives::prism_with_openings(
            &self.first_pt.geom.pt,
            &self.second_pt.geom.pt,
            self.width,
            self.height,
            sorted,
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
        let self_length = (self.second_pt.geom.pt - self.first_pt.geom.pt).magnitude();
        let mut sorted: Vec<PrismOpening> = self
            .openings
            .iter()
            .map(|val| {
                let position = (val.geom.pt_1 - self.first_pt.geom.pt).magnitude();
                let interp = Interp::new(position / self_length);
                let length = (val.geom.pt_2 - val.geom.pt_1).magnitude();
                let height = val.geom.pt_3.z - val.geom.pt_2.z;
                PrismOpening {
                    interp: interp,
                    height: height,
                    length: length,
                }
            })
            .collect();
        sorted.sort_by(|first, second| first.interp.partial_cmp(&second.interp).unwrap());

        primitives::prism_with_openings(
            &self.first_pt.geom.pt,
            &self.second_pt.geom.pt,
            self.width,
            self.height,
            sorted,
            &mut data,
        );
        Ok(UpdateMsg::Mesh { data: data })
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "First" => serde_json::to_value(&self.first_pt.geom.pt).map_err(error_other),
            "Second" => serde_json::to_value(&self.second_pt.geom.pt).map_err(error_other),
            _ => Err(DBError::PropertyNotFound),
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
        if changed {
            Ok(())
        } else {
            Err(DBError::PropertyNotFound)
        }
    }
}

impl ReferTo for Wall {
    fn get_result(&self, result: ResultInd) -> Option<RefGeometry> {
        match result {
            0 => Some(RefGeometry::Point {
                pt: self.first_pt.geom.pt,
            }),
            1 => Some(RefGeometry::Point {
                pt: self.second_pt.geom.pt,
            }),
            2 => Some(RefGeometry::Line {
                pt_1: self.first_pt.geom.pt,
                pt_2: self.second_pt.geom.pt,
            }),
            _ => {
                if let Some(open) = self.openings.get(result - 2) {
                    Some(RefGeometry::Rect {
                        pt_1: open.geom.pt_1,
                        pt_2: open.geom.pt_2,
                        pt_3: open.geom.pt_3,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results.push(RefGeometry::Point {
            pt: self.first_pt.geom.pt,
        });
        results.push(RefGeometry::Point {
            pt: self.second_pt.geom.pt,
        });
        results.push(RefGeometry::Line {
            pt_1: self.first_pt.geom.pt,
            pt_2: self.second_pt.geom.pt,
        });
        for open in &self.openings {
            results.push(RefGeometry::Rect {
                pt_1: open.geom.pt_1,
                pt_2: open.geom.pt_2,
                pt_3: open.geom.pt_3,
            });
        }
        results
    }

    fn get_num_results(&self) -> usize {
        3 + self.openings.len()
    }
}

impl UpdateFromRefs for Wall {
    fn clear_refs(&mut self) {
        self.first_pt.refer = None;
        self.second_pt.refer = None;
        for open in &mut self.openings {
            open.refer = None;
        }
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        if let Some(id) = &self.first_pt.refer {
            results.push(Some(Reference::new(self.id.clone(), 0, id.clone())));
        } else {
            results.push(None);
        }
        if let Some(id) = &self.second_pt.refer {
            results.push(Some(Reference::new(self.id.clone(), 1, id.clone())));
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
        let mut index = 3;
        for open in &self.openings {
            if let Some(id) = &open.refer {
                results.push(Some(Reference::new(self.id.clone(), index, id.clone())));
                index += 1;
            }
        }
        results
    }

    fn get_num_refs(&self) -> usize {
        2 + self.openings.len()
    }

    fn get_available_refs(&self) -> Vec<ReferInd> {
        let mut results = Vec::new();
        if let None = self.first_pt.refer {
            results.push(0);
        }
        if let None = self.second_pt.refer {
            results.push(1);
        }
        results
    }

    fn set_ref(
        &mut self,
        index: ReferInd,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) {
        match index {
            0 => self.first_pt.set_reference(result, other_ref, snap_pt),
            1 => self.second_pt.set_reference(result, other_ref, snap_pt),
            _ => {
                if let Some(open) = self.openings.get_mut(index - 3) {
                    open.set_reference(result, other_ref, snap_pt);
                }
            }
        }
    }

    fn add_ref(
        &mut self,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) -> bool {
        if let RefGeometry::Rect { pt_1, pt_2, pt_3 } = result {
            let mut new_open = UpdatableGeometry::new(RefRect::new(*pt_1, *pt_2, *pt_3));
            new_open.set_reference(result, other_ref, snap_pt);
            self.openings.push(new_open);
            true
        } else {
            false
        }
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index {
            0 => self.first_pt.refer = None,
            1 => self.second_pt.refer = None,
            _ => {
                if self.openings.len() > (index - 3) {
                    self.openings.remove(index - 3);
                }
            }
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index {
            0 => Some(self.first_pt.geom.get_geom()),
            1 => Some(self.second_pt.geom.get_geom()),
            _ => {
                if let Some(open) = self.openings.get(index - 3) {
                    Some(open.geom.get_geom())
                } else {
                    None
                }
            }
        }
    }

    fn set_associated_geom(&mut self, index: ReferInd, geom: &Option<RefGeometry>) {
        match index {
            0 => self.first_pt.update(geom),
            1 => self.second_pt.update(geom),
            _ => {
                if let Some(open) = self.openings.get_mut(index - 3) {
                    open.update(geom);
                }
            }
        }
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt.geom.pt += *delta;
        self.second_pt.geom.pt += *delta;
        for open in &mut self.openings {
            open.geom.pt_1 += *delta;
            open.geom.pt_2 += *delta;
            open.geom.pt_3 += *delta;
        }
    }
}
