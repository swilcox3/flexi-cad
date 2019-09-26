#![allow(non_snake_case)]
extern crate wasm_bindgen;
extern crate data_model;

use wasm_bindgen::prelude::*;
use data_model::*;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    pub type CoordTriple;

    #[wasm_bindgen(structural, method, getter)]
    pub fn x(this: &CoordTriple) -> f64;

    #[wasm_bindgen(structural, method, getter)]
    pub fn y(this: &CoordTriple) -> f64;

    #[wasm_bindgen(structural, method, getter)]
    pub fn z(this: &CoordTriple) -> f64;
}

fn point_3f(coord: &CoordTriple) -> data_model::Point3f {
    data_model::Point3f::new(coord.x(), coord.y(), coord.z())
}

fn vector_3f(coord: &CoordTriple) -> data_model::Vector3f {
    data_model::Vector3f::new(coord.x(), coord.y(), coord.z())
}

#[wasm_bindgen]
pub struct JsWall {
    wall: data_model::Wall,
}

#[wasm_bindgen]
impl JsWall {
    #[wasm_bindgen(constructor)]
    pub fn new(first: CoordTriple, second: CoordTriple, width: WorldCoord, height: WorldCoord) -> JsWall {
        let wall = data_model::Wall::new(point_3f(&first), point_3f(&second), width, height );
        JsWall{ wall }
    }

    pub fn getTempRepr(&self) -> JsValue {
        let msg = self.wall.get_temp_repr().unwrap();
        JsValue::from_serde(&msg).unwrap()
    }

    pub fn moveObj(&mut self, delta: CoordTriple) {
        self.wall.move_obj(&vector_3f(&delta));
    }

    pub fn getObj(&self) -> JsValue {
        JsValue::from_serde(&data_model::to_json("Wall", &self.wall)).unwrap()
    }

    pub fn first_pt(&self) -> JsValue {
        JsValue::from_serde(&self.wall.first_pt.geom.pt).unwrap()
    }

    pub fn set_first_pt(&mut self, val: CoordTriple) {
        self.wall.first_pt.geom.pt = point_3f(&val);
    }

    pub fn second_pt(&self) -> JsValue {
        JsValue::from_serde(&self.wall.second_pt.geom.pt).unwrap()
    }

    pub fn set_second_pt(&mut self, val: CoordTriple) {
        self.wall.second_pt.geom.pt = point_3f(&val);
    }

    pub fn height(&self) -> WorldCoord {
        self.wall.height
    }

    pub fn set_height(&mut self, val: WorldCoord) {
        self.wall.height = val;
    }

    pub fn width(&self) -> WorldCoord {
        self.wall.width
    }

    pub fn set_width(&mut self, val: WorldCoord) {
        self.wall.width = val;
    }

    pub fn id(&self) -> String {
        format!("{:?}", self.wall.get_id())
    }
}

#[wasm_bindgen]
pub struct JsDoor {
    door: data_model::Door,
}

#[wasm_bindgen]
impl JsDoor {
    #[wasm_bindgen(constructor)]
    pub fn new(first: CoordTriple, second: CoordTriple, width: WorldCoord, height: WorldCoord) -> JsDoor {
        let door = data_model::Door::new(point_3f(&first), point_3f(&second), width, height );
        JsDoor{ door }
    }

    pub fn getTempRepr(&self) -> JsValue {
        let msg = self.door.get_temp_repr().unwrap();
        JsValue::from_serde(&msg).unwrap()
    }

    pub fn moveObj(&mut self, delta: CoordTriple) {
        self.door.move_obj(&vector_3f(&delta));
    }

    pub fn getObj(&self) -> JsValue {
        JsValue::from_serde(&data_model::to_json("Door", &self.door)).unwrap()
    }

    pub fn setDir(&mut self, delta: CoordTriple) {
        self.door.dir.geom.set_dir(&vector_3f(&delta));
    }

    pub fn first_pt(&self) -> JsValue {
        JsValue::from_serde(&self.door.dir.geom.pt_1).unwrap()
    }

    pub fn set_first_pt(&mut self, val: CoordTriple) {
        self.door.dir.geom.pt_2 = point_3f(&val);
    }

    pub fn second_pt(&self) -> JsValue {
        JsValue::from_serde(&self.door.dir.geom.pt_2).unwrap()
    }

    pub fn set_second_pt(&mut self, val: CoordTriple) {
        self.door.dir.geom.pt_2 = point_3f(&val);
    }

    pub fn height(&self) -> WorldCoord {
        self.door.height
    }

    pub fn set_height(&mut self, val: WorldCoord) {
        self.door.height = val;
    }

    pub fn width(&self) -> WorldCoord {
        self.door.width
    }

    pub fn set_width(&mut self, val: WorldCoord) {
        self.door.width = val;
    }

    pub fn id(&self) -> String {
        format!("{:?}", self.door.get_id())
    }
}

#[wasm_bindgen]
pub struct JsDimension {
    dim: data_model::Dimension
}

#[wasm_bindgen]
impl JsDimension {
    #[wasm_bindgen(constructor)]
    pub fn new(first: CoordTriple, second: CoordTriple, offset: WorldCoord) -> JsDimension {
        let dim = data_model::Dimension::new(point_3f(&first), point_3f(&second), offset);
        JsDimension{ dim }
    }

    pub fn getTempRepr(&self) -> JsValue {
        let msg = self.dim.get_temp_repr().unwrap();
        JsValue::from_serde(&msg).unwrap()
    }

    pub fn moveObj(&mut self, delta: CoordTriple) {
        self.dim.move_obj(&vector_3f(&delta));
    }

    pub fn getObj(&self) -> JsValue{
        JsValue::from_serde(&data_model::to_json("Dimension", &self.dim)).unwrap()
    }

    pub fn first_pt(&self) -> JsValue {
        JsValue::from_serde(&self.dim.first.geom.pt).unwrap()
    }

    pub fn set_first_pt(&mut self, val: CoordTriple) {
        self.dim.first.geom.pt = point_3f(&val);
    }

    pub fn second_pt(&self) -> JsValue {
        JsValue::from_serde(&self.dim.second.geom.pt).unwrap()
    }

    pub fn set_second_pt(&mut self, val: CoordTriple) {
        self.dim.second.geom.pt = point_3f(&val);
    }

    pub fn offset(&self) -> WorldCoord {
        self.dim.offset
    }

    pub fn set_offset(&mut self, val: WorldCoord) {
        self.dim.offset = val;
    }

    pub fn id(&self) -> String {
        format!("{:?}", self.dim.get_id())
    }
}

#[wasm_bindgen]
pub fn projectOnLine(first: CoordTriple, second: CoordTriple, project: CoordTriple) -> JsValue {
    JsValue::from_serde(&data_model::project_on_line(&point_3f(&first), &point_3f(&second), &point_3f(&project))).unwrap()
}

#[wasm_bindgen]
pub fn getUserId() -> String {
    format!("{:?}", UserID::new_v4())
}

#[wasm_bindgen]
pub fn getUndoEventId() -> String {
    format!("{:?}", UndoEventID::new_v4())
}

#[wasm_bindgen]
pub fn getQueryId() -> String {
    format!("{:?}", QueryID::new_v4())
}