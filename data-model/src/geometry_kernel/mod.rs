use serde::{Deserialize, Serialize};
use crate::RefID;
use cgmath::prelude::*;

pub mod primitives;

pub const ORIGIN: Point3f = Point3f::new(0.0, 0.0, 0.0);

pub type Point3f = cgmath::Point3<f64>;
pub type WorldCoord = f64;
pub type Vector3f = cgmath::Vector3<f64>;
pub type ResultInd = usize;
pub type ReferInd = usize;

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

pub trait Updatable {
    fn get_geom(&self) -> RefGeometry;
    fn update_geom(&mut self, geom: &RefGeometry, snap_pt: &Option<Point3f>);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatableGeometry<T: Updatable> {
    pub refer: Option<GeometryId>,
    pub geom: T
}

impl<T: Updatable> UpdatableGeometry<T> {
    pub fn new(geom: T) -> UpdatableGeometry<T> {
        UpdatableGeometry {
            refer: None,
            geom: geom
        }
    }

    pub fn update(&mut self, ref_geom: &Option<RefGeometry>) {
        if let Some(geom) = ref_geom {
            self.geom.update_geom(&geom, &None);
        }
        else {
            self.refer = None;
        }
    }

    pub fn set_reference(&mut self, result: &RefGeometry, refer: GeometryId, snap_pt: &Option<Point3f>) {
        self.refer = Some(refer);
        self.geom.update_geom(&result, snap_pt);
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefPoint {
    pub pt: Point3f
}

impl Updatable for RefPoint {
    fn get_geom(&self) -> RefGeometry {
        RefGeometry::Point{pt: self.pt}
    }
    
    fn update_geom(&mut self, geom: &RefGeometry, _: &Option<Point3f>) {
        if let RefGeometry::Point{pt} = geom {
            self.pt = *pt;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefLineSeg {
    pub pt_1: Point3f,
    pub pt_2: Point3f,
    pub length: WorldCoord,
    pub interp: Interp
}

impl RefLineSeg {
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
    fn get_geom(&self) -> RefGeometry {
        RefGeometry::Line{pt_1: self.pt_1, pt_2: self.pt_2}
    }
    
    fn update_geom(&mut self, geom: &RefGeometry, snap_pt: &Option<Point3f>) {
        if let RefGeometry::Line{pt_1, pt_2} = geom {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefRect {
    pub pt_1: Point3f,
    pub pt_2: Point3f,
    pub pt_3: Point3f,
}

impl RefRect {
    pub fn new(pt_1: Point3f, pt_2: Point3f, pt_3: Point3f) -> RefRect {
        RefRect {
            pt_1: pt_1, 
            pt_2: pt_2,
            pt_3: pt_3
        }
    }
}

impl Updatable for RefRect {
    fn get_geom(&self) -> RefGeometry {
        RefGeometry::Rect{pt_1: self.pt_1, pt_2: self.pt_2, pt_3: self.pt_3}
    }
    
    fn update_geom(&mut self, geom: &RefGeometry, _: &Option<Point3f>) {
        if let RefGeometry::Rect{pt_1, pt_2, pt_3} = geom {
            self.pt_1 = *pt_1;
            self.pt_2 = *pt_2;
            self.pt_3 = *pt_3;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct GeometryId {
    pub id: RefID,
    pub index: ResultInd
}

impl GeometryId {
    pub fn new(id: RefID, index: usize) -> GeometryId {
        GeometryId {
            id,
            index
        }
    }
}

///The operations kernel will look up another object by RefID, then index into its geometry vector to pass a Option<RefGeometry> into update_from_refs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct Reference {
    pub owner: GeometryId,
    pub other: GeometryId
}

impl Reference {
    pub fn new(owner_id: RefID, owner_index: usize, other: GeometryId) -> Reference {
        Reference {
            owner: GeometryId::new(owner_id, owner_index),
            other
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RefType {
    Point,
    Line,
    Rect,
    Any,
}

impl RefType {
    pub fn type_equals(&self, other: &RefGeometry) -> bool {
        match *self {
            RefType::Point => {
                if let RefGeometry::Point{..} = other {
                    true
                }
                else {
                    false
                }
            }
            RefType::Line => {
                if let RefGeometry::Line{..} = other {
                    true
                }
                else {
                    false
                }
            }
            RefType::Rect => {
                if let RefGeometry::Rect{..} = other {
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

///Specific geometry that can be referenced on the object that implements ReferTo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefGeometry {
    Point{pt: Point3f},
    Line{pt_1: Point3f, pt_2: Point3f},
    Rect{pt_1: Point3f, pt_2: Point3f, pt_3: Point3f}
}

impl RefGeometry {
    pub fn distance2(&self, in_pt: &Point3f) -> WorldCoord {
        match *self {
            RefGeometry::Point{pt} => pt.distance2(*in_pt),
            RefGeometry::Line{pt_1, pt_2} => {
                let projected = project_on_line(&pt_1, &pt_2, in_pt);
                projected.distance2(*in_pt)
            }
            RefGeometry::Rect{pt_1, ..} => pt_1.distance2(*in_pt)
        }
    }

    pub fn get_type(&self) -> RefType {
        match *self {
            RefGeometry::Point{..} => RefType::Point,
            RefGeometry::Line{..} => RefType::Line,
            RefGeometry::Rect{..} => RefType::Rect
        }
    }
}

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
