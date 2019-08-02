use serde::{Deserialize, Serialize};
use crate::RefID;

pub mod primitives;

pub const ORIGIN: Point3f = Point3f::new(0.0, 0.0, 0.0);

pub type Point3f = cgmath::Point3<f64>;
pub type WorldCoord = f64;
pub type Vector3f = cgmath::Vector3<f64>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect(Point3f, Point3f, Point3f, Point3f);

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
    pub ref_type: RefType
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RefType {
    Point{which_pt: u64},
    Line{interp: Interp, pts: (u64, u64)},
    Opening{which: u64},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefResult {
    Point{pt: Point3f},
    Line{pt: Point3f, dir: Vector3f},
    Opening{interp: Interp, height: WorldCoord, length: WorldCoord}
}

//A value between 0 and 1
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Interp {
    val: f64,
}

impl Interp {
    pub fn new(mut in_val: f64) -> Interp {
        if in_val > 1.0 {
            in_val = 1.0;
        }
        if in_val < 0.0 {
            in_val = 0.0;
        }
        Interp {
            val: in_val,
        }
    }

    pub fn val(&self) -> f64 {
        self.val
    }
}

pub trait Position {
    fn move_obj(&mut self, delta: &Vector3f);
}