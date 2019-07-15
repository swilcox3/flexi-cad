use serde::{Deserialize, Serialize};
use crate::RefID;

pub type Point3f = cgmath::Point3<f64>;
pub type WorldCoord = f64;
pub type Vector3f = cgmath::Vector3<f64>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshData {
    pub id: RefID,
    pub positions: Vec<WorldCoord>,
    pub indices: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Reference {
    pub id: RefID,
    pub which_pt: u64
}

impl Reference {
    pub fn nil() -> Reference {
        Reference {
            id: RefID::nil(),
            which_pt: 0
        }
    }
}

impl MeshData {
    pub fn push_pt(&mut self, pt: Point3f) {
        self.positions.push(pt.x);
        self.positions.push(pt.z);
        self.positions.push(-pt.y);
    }
}

pub trait Position {
    fn move_obj(&mut self, delta: &Vector3f);
}

pub trait RefPoint {
    fn get_point(&self, which: u64) -> Option<&Point3f>;

    fn set_point(&mut self, which_self: u64, pt: Point3f, ref_other: Reference);
}

#[cfg(test)]
mod tests {
}
