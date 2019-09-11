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
pub struct Point3d {
    pt: data_model::Point3f
}

#[wasm_bindgen]
impl Point3d {
    #[wasm_bindgen(constructor)]
    pub fn new(x: WorldCoord, y: WorldCoord, z: WorldCoord) -> Point3d {
        Point3d {
            pt: Point3f::new(x, y, z)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> WorldCoord {
        self.pt.x
    }

    #[wasm_bindgen(setter)]
    pub fn set_x(&mut self, val: WorldCoord) {
        self.pt.x = val;
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> WorldCoord {
        self.pt.y
    }

    #[wasm_bindgen(setter)]
    pub fn set_y(&mut self, val: WorldCoord) {
        self.pt.y = val;
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> WorldCoord {
        self.pt.z
    }

    #[wasm_bindgen(setter)]
    pub fn set_z(&mut self, val: WorldCoord) {
        self.pt.z = val;
    }
}

#[wasm_bindgen]
pub struct Vector3d {
    vect: data_model::Vector3f
}

#[wasm_bindgen]
impl Vector3d {
    #[wasm_bindgen(constructor)]
    pub fn new(x: WorldCoord, y: WorldCoord, z: WorldCoord) -> Vector3d {
        Vector3d {
            vect: Vector3f::new(x, y, z)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> WorldCoord {
        self.vect.x
    }

    #[wasm_bindgen(setter)]
    pub fn set_x(&mut self, val: WorldCoord) {
        self.vect.x = val;
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> WorldCoord {
        self.vect.y
    }

    #[wasm_bindgen(setter)]
    pub fn set_y(&mut self, val: WorldCoord) {
        self.vect.y = val;
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> WorldCoord {
        self.vect.z
    }

    #[wasm_bindgen(setter)]
    pub fn set_z(&mut self, val: WorldCoord) {
        self.vect.z = val;
    }
}
#[wasm_bindgen]
pub struct JsWall {
    wall: data_model::Wall,
}

#[wasm_bindgen]
impl JsWall {
    #[wasm_bindgen(constructor)]
    pub fn new(first: Point3d, second: Point3d, width: WorldCoord, height: WorldCoord) -> JsWall {
        let wall = data_model::Wall::new(first.pt, second.pt, width, height );
        JsWall{ wall }
    }

    pub fn getTempRepr(&self) -> JsValue {
        let msg = self.wall.get_temp_repr().unwrap();
        JsValue::from_serde(&msg).unwrap()
    }

    pub fn moveObj(&mut self, delta: Vector3d) {
        self.wall.move_obj(&delta.vect);
    }

    #[wasm_bindgen(getter)]
    pub fn first_pt(&self) -> Point3d {
        Point3d{pt: self.wall.first_pt.geom.pt.clone()}
    }

    #[wasm_bindgen(setter)]
    pub fn set_first_pt(&mut self, val: Point3d) {
        self.wall.first_pt.geom.pt = val.pt;
    }

    #[wasm_bindgen(getter)]
    pub fn second_pt(&self) -> Point3d {
        Point3d{pt: self.wall.second_pt.geom.pt.clone()}
    }

    #[wasm_bindgen(setter)]
    pub fn set_second_pt(&mut self, val: Point3d) {
        self.wall.second_pt.geom.pt = val.pt;
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> WorldCoord {
        self.wall.height
    }

    #[wasm_bindgen(setter)]
    pub fn set_height(&mut self, val: WorldCoord) {
        self.wall.height = val;
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> WorldCoord {
        self.wall.width
    }

    #[wasm_bindgen(setter)]
    pub fn set_width(&mut self, val: WorldCoord) {
        self.wall.width = val;
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        format!("{:?}", self.wall.get_id().clone())
    }
}

#[wasm_bindgen]
pub struct JsDoor {
    door: data_model::Door,
}

#[wasm_bindgen]
impl JsDoor {
    #[wasm_bindgen(constructor)]
    pub fn new(first: Point3d, second: Point3d, width: WorldCoord, height: WorldCoord) -> JsDoor {
        let door = data_model::Door::new(first.pt, second.pt, width, height );
        JsDoor{ door }
    }

    pub fn getTempRepr(&self) -> JsValue {
        let msg = self.door.get_temp_repr().unwrap();
        JsValue::from_serde(&msg).unwrap()
    }

    pub fn moveObj(&mut self, delta: Vector3d) {
        self.door.move_obj(&delta.vect);
    }

    #[wasm_bindgen(getter)]
    pub fn first_pt(&self) -> Point3d {
        Point3d{pt: self.door.dir.geom.pt_1.clone()}
    }

    #[wasm_bindgen(setter)]
    pub fn set_first_pt(&mut self, val: Point3d) {
        self.door.dir.geom.pt_2 = val.pt;
    }

    #[wasm_bindgen(getter)]
    pub fn second_pt(&self) -> Point3d {
        Point3d{pt: self.door.dir.geom.pt_2.clone()}
    }

    #[wasm_bindgen(setter)]
    pub fn set_second_pt(&mut self, val: Point3d) {
        self.door.dir.geom.pt_2 = val.pt;
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> WorldCoord {
        self.door.height
    }

    #[wasm_bindgen(setter)]
    pub fn set_height(&mut self, val: WorldCoord) {
        self.door.height = val;
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> WorldCoord {
        self.door.width
    }

    #[wasm_bindgen(setter)]
    pub fn set_width(&mut self, val: WorldCoord) {
        self.door.width = val;
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        format!("{:?}", self.door.get_id().clone())
    }
}

