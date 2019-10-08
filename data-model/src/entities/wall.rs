use crate::*;
use primitives::PrismOpening;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub first_pt: ParametricPoint,
    pub second_pt: ParametricPoint,
    pub width: WorldCoord,
    pub height: WorldCoord,
    openings: Vec<ParametricPoint>,
    #[serde(skip_serializing, default = "String::new")]
    data: String,
    id: RefID
}

interfaces!(Wall: dyn query_interface::ObjectClone, dyn std::fmt::Debug, dyn Data, dyn ReferTo, dyn Position, dyn UpdateFromRefs);

impl Wall {
    pub fn new(first: Point3f, second: Point3f, width: WorldCoord, height: WorldCoord) -> Wall {
        let id = RefID::new_v4();
        Wall {
            first_pt: ParametricPoint::new(first),
            second_pt: ParametricPoint::new(second),
            width: width,
            height: height,
            openings: Vec::new(),
            data: String::new(),
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

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        if self.data.len() == 0 {
            for i in 0..500000 {
                self.data += &i.to_string();
            }
        }
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(to_json("Wall", &self))
        };
        let self_length = (self.second_pt.pt - self.first_pt.pt).magnitude();
        self.openings.retain(|open| open.refer != None);
        let mut sorted = Vec::new();
        for i in (0..self.openings.len()).step_by(3) {
            if let Some(pt_1) = self.openings.get(i) {
                if let Some(pt_2) = self.openings.get(i+1) {
                    if let Some(pt_3) = self.openings.get(i+2) {
                        let position = (pt_1.pt - self.first_pt.pt).magnitude();
                        let interp = Interp::new(position / self_length);
                        let length = (pt_2.pt - pt_1.pt).magnitude();
                        let height = pt_3.pt.z - pt_2.pt.z;
                        sorted.push(PrismOpening{interp: interp, height: height, length: length})
                    }
                }
            }
        }
        sorted.sort_by(|first, second| first.interp.partial_cmp(&second.interp).unwrap());

        primitives::prism_with_openings(&self.first_pt.pt, &self.second_pt.pt, self.width, self.height, sorted, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: None
        };
        let self_length = (self.second_pt.pt - self.first_pt.pt).magnitude();
        let mut sorted = Vec::new();
        for i in (0..self.openings.len()).step_by(3) {
            if let Some(pt_1) = self.openings.get(i) {
                if let Some(pt_2) = self.openings.get(i+1) {
                    if let Some(pt_3) = self.openings.get(i+2) {
                        let position = (pt_1.pt - self.first_pt.pt).magnitude();
                        let interp = Interp::new(position / self_length);
                        let length = (pt_2.pt - pt_1.pt).magnitude();
                        let height = pt_3.pt.z - pt_2.pt.z;
                        sorted.push(PrismOpening{interp: interp, height: height, length: length})
                    }
                }
            }
        }
        sorted.sort_by(|first, second| first.interp.partial_cmp(&second.interp).unwrap());

        primitives::prism_with_openings(&self.first_pt.pt, &self.second_pt.pt, self.width, self.height, sorted, &mut data);
        Ok(UpdateMsg::Mesh{data: data})
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Width" => Ok(json!(self.width)),
            "Height" => Ok(json!(self.height)),
            "First" => serde_json::to_value(&self.first_pt.pt).map_err(error_other),
            "Second" => serde_json::to_value(&self.second_pt.pt).map_err(error_other),
            _ => Err(DBError::PropertyNotFound)
        }
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
            Err(DBError::PropertyNotFound)
        }
    }
}

impl ReferTo for Wall {
    fn get_point(&self, result: PointIndex) -> Option<Point3f> {
        match result {
            0 => Some(self.first_pt.pt),
            1 => Some(self.second_pt.pt),
            _ => {
                if let Some(pt) = self.openings.get(result - 2) {
                    Some(pt.pt)
                }
                else {
                    None
                }
            }
        }
    }

    fn get_all_points(&self) -> Vec<Point3f> {
        let mut results = Vec::new();
        results.push(self.first_pt.pt);
        results.push(self.second_pt.pt);
        for pt in &self.openings {
            results.push(pt.pt);
        }
        results
    }

    fn get_num_points(&self) -> usize {
        2 + self.openings.len()
    }

}

impl UpdateFromRefs for Wall {
    fn clear_refs(&mut self) {
        self.first_pt.refer = None;
        self.second_pt.refer = None;
        for open in &mut self.openings {
            open.refer = None;
        }
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        if let Some(id) = &self.first_pt.refer {
            results.push(Some(Reference::new(GeometryId{obj: self.id.clone(), index: 0}, id.clone())));
        }
        else {
            results.push(None);
        }
        if let Some(id) = &self.second_pt.refer {
            results.push(Some(Reference::new(GeometryId{obj: self.id.clone(), index: 1}, id.clone())));
        }
        else {
            results.push(None);
        }
        for i in (0..self.openings.len()).step_by(3) {
            if let Some(pt_1) = self.openings.get(i) {
                if let Some(pt_2) = self.openings.get(i+1) {
                    if let Some(pt_3) = self.openings.get(i+2) {
                        if let Some(id_1) = &pt_1.refer {
                            if let Some(id_2) = &pt_2.refer {
                                if let Some(id_3) = &pt_3.refer {
                                    let refer_1 = GeometryId{obj: self.id.clone(), index:i + 2};
                                    let refer_2 = GeometryId{obj: self.id.clone(), index:i + 3};
                                    let refer_3 = GeometryId{obj: self.id.clone(), index:i + 4};
                                    results.push(Some(Reference::new(refer_1.clone(), id_1.clone())));
                                    results.push(Some(Reference::new(refer_2.clone(), id_2.clone())));
                                    results.push(Some(Reference::new(refer_3.clone(), id_3.clone())));
                                    results.push(Some(Reference::new(refer_1.clone(), refer_2.clone())));
                                    results.push(Some(Reference::new(refer_1.clone(), refer_3.clone())));
                                    results.push(Some(Reference::new(refer_2.clone(), refer_1.clone())));
                                    results.push(Some(Reference::new(refer_2.clone(), refer_3.clone())));
                                    results.push(Some(Reference::new(refer_3.clone(), refer_1.clone())));
                                    results.push(Some(Reference::new(refer_3.clone(), refer_2.clone())));
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }

    fn get_available_refs(&self) -> Vec<PointIndex> {
        let mut results = Vec::new();
        if let None = self.first_pt.refer {
            results.push(0);
        }
        if let None = self.second_pt.refer {
            results.push(1);
        }
        results
    }

    fn get_num_refs(&self) -> usize {
        2 + self.openings.len()
    }

    fn set_ref(&mut self, index: PointIndex, result: Point3f, other_ref: GeometryId, _: &Option<Point3f>) {
        match index {
            0 => self.first_pt.set_reference(result, other_ref),
            1 => self.second_pt.set_reference(result, other_ref),
            _ => {
                if let Some(open) = self.openings.get_mut(index - 2) {
                    open.set_reference(result, other_ref);
                }
            }
        }
    }

    fn add_ref(&mut self, result: Point3f, other_ref: GeometryId, _: &Option<Point3f>) -> bool {
        let mut new_open = ParametricPoint::new(result);
        new_open.set_reference(new_open.pt, other_ref);
        self.openings.push(new_open);
        true
    }

    fn delete_ref(&mut self, index: PointIndex) {
        match index {
            0 => self.first_pt.refer = None,
            1 => self.second_pt.refer = None,
            _ => {
                if self.openings.len() > (index - 2) {
                    self.openings.remove(index - 2);
                }
            }
        }
    }

    fn get_associated_point(&self, index: PointIndex) -> Option<Point3f> {
        match index {
            0 => Some(self.first_pt.pt),
            1 => Some(self.second_pt.pt),
            _ => {
                if let Some(open) = self.openings.get(index - 2) {
                    Some(open.pt)
                }
                else {
                    None
                }
            }
        }
    }

    fn set_associated_point(&mut self, index: PointIndex, geom: Option<Point3f>) {
        match index {
            0 => self.first_pt.update(geom),
            1 => self.second_pt.update(geom),
            _ => {
                if let Some(open) = self.openings.get_mut(index - 2) {
                    open.update(geom);
                }
            }
        }
    }
}

impl Position for Wall {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.first_pt.pt += *delta;
        self.second_pt.pt += *delta;
    }
}



