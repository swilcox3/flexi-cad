use serde::{Deserialize, Serialize};
use crate::RefID;
use cgmath::prelude::*;

pub mod primitives;

pub const ORIGIN: Point3f = Point3f::new(0.0, 0.0, 0.0);

pub type Point3f = cgmath::Point3<f64>;
pub type WorldCoord = f64;
pub type Vector3f = cgmath::Vector3<f64>;

pub type PointIndex = usize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub pt_1: Point3f,
    pub pt_2: Point3f
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub pt_1: Point3f,
    pub pt_2: Point3f,
    pub pt_3: Point3f,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Hash, Eq)]
pub struct GeometryId {
    pub obj: RefID,
    pub index: PointIndex,
}

///The other object that is looking at the piece of geometry this is attached to
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Hash, Eq)]
pub struct Reference {
    pub owner: GeometryId,
    pub other: GeometryId
}

impl Reference {
    pub fn new(owner: GeometryId, other: GeometryId) -> Reference {
        Reference{
            owner,
            other
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricPoint {
    pub pt: Point3f,
    pub refer: Option<GeometryId>
}

impl ParametricPoint {
    pub fn new(pt: Point3f) -> ParametricPoint {
        ParametricPoint {
            pt,
            refer: None
        }
    }

    pub fn set_reference(&mut self, pt: Point3f, other_ref: GeometryId) {
        self.pt = pt;
        self.refer = Some(other_ref); 
    }

    pub fn update(&mut self, geom: Option<Point3f>) {
        if let Some(pt) = geom {
            self.pt = pt;
        }
        else {
            self.refer = None;
        }
    }
}

/*impl Point3f {
    pub fn distance2(&self, in_pt: &Point3f) -> WorldCoord {
        match *self {
            Point3f::Point{pt} => pt.distance2(*in_pt),
            Point3f::Line{pt_1, pt_2} => {
                let projected = project_on_line(&pt_1, &pt_2, in_pt);
                projected.distance2(*in_pt)
            }
            Point3f::Rect{pt_1, ..} => pt_1.distance2(*in_pt)
        }
    }
}*/

/*impl RefLineSeg {
    pub fn new(pt_1: Point3f, pt_2: Point3f) -> RefLineSeg {
        let length = (pt_2 - pt_1).magnitude();
        RefLineSeg {
            pt_1: pt_1, 
            pt_2: pt_2,
            length: length,
            interp: Interp::new(0.0)
        }
    }

    pub fn set_dir(&mut self, dir: &Vector3f) {
        self.pt_2 = self.pt_1 + dir.normalize()*self.length;
    }
}

impl Updatable for RefLineSeg {
    fn update_geom(&mut self, geom: Vec<Point3f>, snap_pt: &Option<Point3f>) {
        if let Some(pt_1) = geom.get(0) {
            if let Some(pt_2) = geom.get(1) {
                if let Some(snap) = snap_pt {
                    self.interp = get_interp_along_line(pt_1, pt_2, snap);
                }
                let dir = pt_2 - pt_1;
                let norm = dir.normalize();
                self.pt_1 = pt_1 + dir * self.interp.val;
                self.pt_2 = self.pt_1 + norm * self.length;
            }
        }
    }
}*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshData {
    pub id: RefID,
    pub positions: Vec<WorldCoord>,
    pub indices: Vec<u64>,
    pub metadata: Option<serde_json::Value>
}

impl MeshData {
    pub fn push_pt(&mut self, pt: Point3f) {
        //Bake in coordinate transformations to graphical space
        self.positions.push(pt.x);
        self.positions.push(pt.z);
        self.positions.push(-pt.y);
    }
}

pub trait Position {
    fn move_obj(&mut self, delta: &Vector3f);
}

pub fn project_on_line(first: &Point3f, second: &Point3f, project: &Point3f) -> Point3f {
    let dir = second - first;
    let proj_vec = (project - first).project_on(dir);
    first + proj_vec
}

pub fn get_interp_along_line(first: &Point3f, second: &Point3f, project: &Point3f) -> Interp {
    let dir = second - first;
    let proj_vec = (project - first).project_on(dir);
    Interp::new((proj_vec.magnitude2() / dir.magnitude2()).sqrt())
}

pub fn rotate_point_through_angle_2d(origin: &Point3f, point: &Point3f, angle: cgmath::Rad<f64>) -> Point3f {
    let dir = point - origin;
    let rot = cgmath::Matrix3::from_angle_z(angle);
    let rotated = rot * dir;
    origin + rotated
}

pub fn get_perp_2d(first: &Point3f, second: &Point3f) -> Vector3f {
    (second - first).cross(Vector3f::unit_z()).normalize()
}

pub fn graphic_space(pt: &Point3f) -> Point3f {
    Point3f::new(pt.x, pt.z, -pt.y)
}

///A value between 0 and 1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_on_line() {
        let first = Point3f::new(0.0, 0.0, 0.0);
        let second = Point3f::new(1.0, 0.0, 0.0);
        let project = Point3f::new(0.5, 1.0, 0.0);
        assert_eq!(project_on_line(&first, &second, &project), Point3f::new(0.5, 0.0, 0.0));

        let first = Point3f::new(0.0, 0.0, 0.0);
        let second = Point3f::new(1.0, 0.0, 0.0);
        let project = Point3f::new(0.5, -1.0, 0.0);
        assert_eq!(project_on_line(&first, &second, &project), Point3f::new(0.5, 0.0, 0.0));

        let first = Point3f::new(0.0, 0.0, 0.0);
        let second = Point3f::new(1.0, 0.0, 0.0);
        let project = Point3f::new(-1.0, -1.0, 1.0);
        assert_eq!(project_on_line(&first, &second, &project), Point3f::new(-1.0, 0.0, 0.0));

        let first = Point3f::new(-50.0, 20.0, 0.0);
        let second = Point3f::new(-40.0, 20.0, 0.0);
        let project = Point3f::new(-45.0, 19.0, 0.0);
        assert_eq!(project_on_line(&first, &second, &project), Point3f::new(-45.0, 20.0, 0.0));
    }
}
