use crate::*;
use serde::{Deserialize, Serialize};
use cgmath::InnerSpace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: Point3f,
    pub second_pt: Point3f,
    pub width: WorldCoord,
    pub height: WorldCoord,
    joined_first: Option<Reference>,
    joined_second: Option<Reference>,
    id: RefID
}

interfaces!(Wall: query_interface::ObjectClone, std::fmt::Debug, Data, ReferTo, HasRefs, Position, UpdateFromRefs);

impl Wall {
    pub fn new(id: RefID, first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        Wall {
            first_pt: first,
            second_pt: second,
            width: width,
            height: height,
            joined_first: None,
            joined_second: None,
            id: id
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

    fn update(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(json!({
                "type": "Wall",
                "width": self.width,
                "height": self.height,
            }))
        };
        let dir = self.second_pt - self.first_pt;
        let perp = dir.cross(Vector3f::unit_z()).normalize();
        let offset = perp * self.width;
        let first = self.first_pt + offset;
        let second = self.first_pt - offset;
        let third = self.second_pt + offset;
        let fourth = self.second_pt - offset;
        let vert_offset = Vector3f::new(0.0, 0.0, self.height);
        let fifth = first + vert_offset;
        let sixth = second + vert_offset;
        let seventh = third + vert_offset;
        let eighth = fourth + vert_offset;
        data.push_pt(first);
        data.push_pt(second);
        data.push_pt(third);
        data.push_pt(fourth);
        data.push_pt(fifth);
        data.push_pt(sixth);
        data.push_pt(seventh);
        data.push_pt(eighth);
        data.indices.extend(&[0, 1, 2]);
        data.indices.extend(&[1, 2, 3]);
        data.indices.extend(&[0, 1, 5]);
        data.indices.extend(&[0, 5, 4]);
        data.indices.extend(&[4, 5, 7]);
        data.indices.extend(&[4, 7, 6]);
        data.indices.extend(&[1, 3, 7]);
        data.indices.extend(&[1, 7, 5]);
        data.indices.extend(&[2, 3, 7]);
        data.indices.extend(&[2, 7, 6]);
        data.indices.extend(&[2, 0, 4]);
        data.indices.extend(&[2, 4, 6]);
        Ok(UpdateMsg::Mesh{data: data})
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
        }
        else {
            Err(DBError::NotFound)
        }
    }
}

impl ReferTo for Wall {
    fn get_result(&self, which: &RefType) -> Option<RefResult> {
        match which {
            RefType::Point{which_pt} => {
                match which_pt {
                    0 => Some(RefResult::Point{pt: self.first_pt}),
                    1 => Some(RefResult::Point{pt: self.second_pt}),
                    _ => None 
                }
            }
            _ => None
        }
    }
}

impl UpdateFromRefs for Wall {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new(); 
        results.push(self.joined_first.clone());
        results.push(self.joined_second.clone());
        results
    }

    fn set_ref(&mut self, which_self: &RefType, result: &RefResult, other_ref: Reference) {
        match which_self {
            RefType::Point{which_pt} => {
                match which_pt {
                    0 => {
                        if let RefResult::Point{pt} = result {
                            self.first_pt = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.joined_first = Some(other_ref);
                        }
                    }
                    1 => {
                        if let RefResult::Point{pt} = result {
                            self.second_pt = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.joined_second = Some(other_ref);
                        }
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
            if let Some(RefResult::Point{pt}) = refer {
                self.first_pt = *pt;
            }
            else {
                self.joined_first = None;
            }
        }
        if let Some(refer) = results.get(1) {
            if let Some(RefResult::Point{pt}) = refer {
                self.second_pt = *pt;
            }
            else {
                self.joined_second = None;
            }
        }
        self.update()
    }
}

impl HasRefs for Wall {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.joined_first {
            deps.register_sub(&refer.id, self.id.clone());
        }
        if let Some(refer) = &self.joined_second {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.joined_first = None;
        self.joined_second = None;
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt += *delta;
        self.second_pt += *delta;
    }
}



