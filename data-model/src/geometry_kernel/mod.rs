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
    pub metadata: Option<serde_json::Value>,
}

impl MeshData {
    pub fn push_pt(&mut self, pt: Point3f) {
        //Bake in coordinate transformations to graphical space
        self.positions.push(pt.x);
        self.positions.push(pt.z);
        self.positions.push(-pt.y);
    }
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

pub trait Position {
    fn move_obj(&mut self, delta: &Vector3f);
}

#[cfg(test)]
mod tests {
}
