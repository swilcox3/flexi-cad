use crate::*;
use serde::{Deserialize, Serialize};
use cgmath::MetricSpace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearDimension {
    first: Reference,
    second: Reference,
    offset: WorldCoord,
    id: RefID
}

interfaces!(LinearDimension: query_interface::ObjectClone, std::fmt::Debug, Data, Update, Position);

impl LinearDimension {
    pub fn new(first: Reference, second: Reference, offset: WorldCoord) -> LinearDimension {
        LinearDimension {
            first: first,
            second: second,
            offset: offset,
            id: RefID::new_v4()
        }
    }

    fn get_distance(&mut self, ops: impl ObjStore) -> Result<WorldCoord, DBError> {
        let first_pt = ops.get_ref_point(&mut self.first)?;
        let second_pt = ops.get_ref_point(&mut self.second)?;
        Ok(second_pt.distance(first_pt).abs())
    }
}

impl Data for LinearDimension {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&self) -> Result<UpdateMsg, DBError> {
        Err(DBError::NotFound)
    }

    fn set_data(&mut self, data: &serde_json::Value) -> Result<(), DBError> {
        Ok(())
    }
}

impl Update for LinearDimension {
    fn init(&self, deps: &DepStore) {
        deps.register_sub(&self.first.id, self.id.clone());
        deps.register_sub(&self.second.id, self.id.clone());
    }

    fn clear_refs(&mut self) {
        self.first = Reference::nil();
        self.second = Reference::nil();
    }

    fn get_refs(&self) -> Vec<RefID> {
        let mut results = Vec::new();
        if self.first.id != RefID::nil() {
            results.push(self.first.id.clone());
        }
        if self.second.id != RefID::nil() {
            results.push(self.second.id.clone());
        }
        results
    }

    fn update_from_refs(&mut self, ops: &ObjStore) -> Result<UpdateMsg, DBError> {
        Err(DBError::NotFound)
    }
}

impl Position for LinearDimension {
    fn move_obj(&mut self, delta: &Vector3f) {

    }
}

