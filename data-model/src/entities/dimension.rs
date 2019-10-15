use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    id: RefID,
    pub first: UpdatableGeometry<RefPoint>,
    pub second: UpdatableGeometry<RefPoint>,
    pub offset: WorldCoord,
}

impl Dimension {
    pub fn new(first: Point3f, second: Point3f, offset: WorldCoord) -> Dimension {
        let id = RefID::new_v4();
        Dimension {
            id,
            first: UpdatableGeometry::new(RefPoint { pt: first }),
            second: UpdatableGeometry::new(RefPoint { pt: second }),
            offset: offset,
        }
    }
}

interfaces!(
    Dimension: dyn query_interface::ObjectClone,
    dyn std::fmt::Debug,
    dyn Data,
    dyn Position,
    dyn UpdateFromRefs
);

#[typetag::serde]
impl Data for Dimension {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        let perp = get_perp_2d(&self.first.geom.pt, &self.second.geom.pt);
        let line_1 = self.first.geom.pt + perp * self.offset;
        let line_2 = self.second.geom.pt + perp * self.offset;
        let text_pos = line_1 + (line_2 - line_1) * 0.5;
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
        Ok(UpdateMsg::Other { data: data })
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let perp = get_perp_2d(&self.first.geom.pt, &self.second.geom.pt);
        let line_1 = self.first.geom.pt + perp * self.offset;
        let line_2 = self.second.geom.pt + perp * self.offset;
        let text_pos = line_1 + (line_2 - line_1) * 0.5;
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
        Ok(UpdateMsg::Other { data: data })
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            "Offset" => Ok(json!(self.offset)),
            "First" => serde_json::to_value(&self.first.geom.pt).map_err(error_other),
            "Second" => serde_json::to_value(&self.second.geom.pt).map_err(error_other),
            _ => Err(DBError::PropertyNotFound),
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
        } else {
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
        let mut results = Vec::new();
        if let Some(id) = &self.first.refer {
            results.push(Some(Reference::new(self.id.clone(), 0, id.clone())));
        } else {
            results.push(None);
        }
        if let Some(id) = &self.second.refer {
            results.push(Some(Reference::new(self.id.clone(), 1, id.clone())));
        } else {
            results.push(None);
        }
        results
    }

    fn get_available_refs(&self) -> Vec<ReferInd> {
        let mut results = Vec::new();
        if let None = self.first.refer {
            results.push(0);
        }
        if let None = self.second.refer {
            results.push(1);
        }
        results
    }

    fn get_num_refs(&self) -> usize {
        2
    }

    fn set_ref(
        &mut self,
        index: ReferInd,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) {
        match index {
            0 => self.first.set_reference(result, other_ref, snap_pt),
            1 => self.second.set_reference(result, other_ref, snap_pt),
            _ => (),
        }
    }

    fn add_ref(&mut self, _: &RefGeometry, _: GeometryId, _: &Option<Point3f>) -> bool {
        return false;
    }

    fn delete_ref(&mut self, index: ReferInd) {
        match index {
            0 => self.first.refer = None,
            1 => self.second.refer = None,
            _ => (),
        }
    }

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        match index {
            0 => Some(self.first.geom.get_geom()),
            1 => Some(self.second.geom.get_geom()),
            _ => None,
        }
    }

    fn set_associated_geom(&mut self, index: ReferInd, geom: &Option<RefGeometry>) {
        match index {
            0 => self.first.update(geom),
            1 => self.second.update(geom),
            _ => (),
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
