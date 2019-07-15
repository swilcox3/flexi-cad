use crate::*;
use serde::{Deserialize, Serialize};
use cgmath::InnerSpace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: Point3f,
    pub second_pt: Point3f,
    pub width: WorldCoord,
    pub height: WorldCoord,
    joined_first: Reference,
    joined_second: Reference,
    id: RefID
}

interfaces!(Wall: query_interface::ObjectClone, std::fmt::Debug, Data, Update, Position, RefPoint);

impl Wall {
    pub fn new(id: RefID, first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        Wall {
            first_pt: first,
            second_pt: second,
            width: width,
            height: height,
            joined_first: Reference::nil(),
            joined_second: Reference::nil(),
            id: id
        }
    }
}

impl Data for Wall {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn update(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36)
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
}

impl RefPoint for Wall {
    fn get_point(&self, which: u64) -> Option<&Point3f> {
        match which {
            0 => Some(&self.first_pt),
            1 => Some(&self.second_pt),
            _ => None 
        }
    }

    fn set_point(&mut self, which_self: u64, pt: Point3f, other_ref: Reference) {
        match which_self {
            0 => {
                self.first_pt = pt;
                self.joined_first = other_ref;
            }
            1 => {
                self.second_pt = pt;
                self.joined_second = other_ref;
            }
            _ => ()
        }
    }
}

impl Update for Wall {
    fn init(&self, deps: &DepStore) {
        deps.register_sub(&self.joined_first.id, self.id.clone());
        deps.register_sub(&self.joined_second.id, self.id.clone());
    }

    fn update_from_refs(&mut self, ops: &ObjStore) -> Result<UpdateMsg, DBError> {
        if let Ok(pt) = ops.get_ref_point(&mut self.joined_first) {
            self.first_pt = pt;
        }
        if let Ok(pt) = ops.get_ref_point(&mut self.joined_second) {
            self.second_pt = pt;
        }
        self.update()
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt += *delta;
        self.second_pt += *delta;
    }
}



