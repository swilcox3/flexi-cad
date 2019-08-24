use crate::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    id: RefID,
    pub first: UpdatableGeometry<RefPoint>,
    pub second: UpdatableGeometry<RefPoint>,
    pub offset: WorldCoord,
}

impl Dimension {
    pub fn new(id: RefID, first: Point3f, second: Point3f, offset: WorldCoord) -> Dimension {
        Dimension {
            id: id,
            first: UpdatableGeometry::new(RefPoint{pt: first}),
            second: UpdatableGeometry::new(RefPoint{pt: second}),
            offset: offset,
        }
    }
}

interfaces!(Dimension: query_interface::ObjectClone, std::fmt::Debug, Data, Position, UpdateFromRefs);

#[typetag::serde]
impl Data for Dimension{
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&self) -> Result<UpdateMsg, DBError> {
        let perp = get_perp_2d(&self.first.geom.pt, &self.second.geom.pt);
        let line_1 = self.first.geom.pt + perp * self.offset; 
        let line_2 = self.second.geom.pt + perp * self.offset;
        let text_pos = line_1 + (line_2 - line_1)*0.5;
        let distance = (line_2 - line_1).magnitude();
        let text = format!("{:.3}", distance);

        let data = json!({
            "id": self.get_id().clone(),
            "first": graphic_space(&self.first.geom.pt),
            "first_off": graphic_space(&line_1),
            "second": graphic_space(&self.second.geom.pt),
            "second_off": graphic_space(&line_2),
            "text_pos": graphic_space(&text_pos),
            "text": text,
            "offset": self.offset,
            "metadata": {
                "type": "Dimension",
                "Offset": self.offset
            }
        });
        Ok(UpdateMsg::Other{data: data})
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Offset" => Ok(json!(self.offset)),
            "First" => serde_json::to_value(&self.first.geom.pt).map_err(error_other),
            "Second" => serde_json::to_value(&self.second.geom.pt).map_err(error_other),
            _ => Err(DBError::PropertyNotFound)
        }
    }

    fn set_data(&mut self, data: &serde_json::Value) -> Result<(), DBError> {
        let mut changed = false;
        if let serde_json::Value::Number(num) = &data["Offset"] {
            changed = true;
            self.offset = num.as_f64().unwrap();
        }
        if changed {
            Ok(())
        }
        else {
            Err(DBError::PropertyNotFound)
        }
    }
}

impl UpdateFromRefs for Dimension {
    fn clear_refs(&mut self) {
        self.first.refer = None;
        self.second.refer = None;
    }

    fn get_refs(&self) -> Vec<Option<Reference>> {
        vec![self.first.refer.clone(), self.second.refer.clone()]
    }

    fn set_ref(&mut self, index: ReferInd, result: &RefGeometry, other_ref: Reference, snap_pt: &Option<Point3f>) {
        match index.index {
            0 => self.first.set_reference(result, other_ref, snap_pt),
            1 => self.second.set_reference(result, other_ref, snap_pt),
            _ => ()
        }
    }

    fn add_ref(&mut self, _: &RefGeometry, _: Reference, _: &Option<Point3f>) -> bool {
        return false;
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index.index {
            0 => self.first.refer = None,
            1 => self.second.refer = None,
            _ => ()
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index.index {
            0 => Some(self.first.geom.get_geom()),
            1 => Some(self.second.geom.get_geom()),
            _ => {
                None
            }
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefGeometry>>) {
        if let Some(geom) = results.get(0) {
            self.first.update(geom);
        }
        if let Some(geom) = results.get(1) {
            self.second.update(geom);
        }
    }
}

impl Position for Dimension {
    fn move_obj(&mut self, delta: &Vector3f) {
        let perp = get_perp_2d(&self.first.geom.pt, &self.second.geom.pt);
        let projected = delta.project_on(perp);
        self.offset = projected.magnitude();
    }
}



