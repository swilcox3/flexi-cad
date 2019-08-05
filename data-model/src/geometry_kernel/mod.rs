use serde::{Deserialize, Serialize};
use crate::{RefID, ResIndex};
use cgmath::prelude::*;

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
    pub index: ResIndex,
    pub ref_type: RefType
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RefType {
    Point,
    Line{interp: Interp},
    Rect,
    Any,
}

impl RefType {
    pub fn type_equals(&self, other: &RefResult) -> bool {
        match *self {
            RefType::Point => {
                if let RefResult::Point{..} = other {
                    true
                }
                else {
                    false
                }
            }
            RefType::Line{..} => {
                if let RefResult::Line{..} = other {
                    true
                }
                else {
                    false
                }
            }
            RefType::Rect => {
                if let RefResult::Rect{..} = other {
                    true
                }
                else {
                    false
                }
            }
            RefType::Any => {
                true
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefResult {
    Point{pt: Point3f},
    Line{pt_1: Point3f, pt_2: Point3f},
    Rect{pt_1: Point3f, pt_2: Point3f, pt_3: Point3f}
}

impl RefResult {
    pub fn distance2(&self, in_pt: &Point3f) -> (WorldCoord, RefType) {
        match *self {
            RefResult::Point{pt} => (pt.distance2(*in_pt), RefType::Point),
            RefResult::Line{pt_1, pt_2} => {
                let dir = pt_2 - pt_1;
                let proj_vec = in_pt.to_vec().project_on(dir);
                let projected: Point3f = EuclideanSpace::from_vec(proj_vec);
                let interp = (proj_vec.magnitude2() / dir.magnitude2()).sqrt();
                (projected.distance2(*in_pt), RefType::Line{interp: Interp::new(interp)})
            }
            RefResult::Rect{pt_1, ..} => (pt_1.distance2(*in_pt), RefType::Rect)
        }
    }
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