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

    fn update(&self) -> Result<UpdateMsg, DBError> {
        Err(DBError::NotFound)
    }
}

impl Update for LinearDimension {
    fn init(&self, deps: &DepStore) {
        deps.register_sub(&self.first.id, self.id.clone());
        deps.register_sub(&self.second.id, self.id.clone());
    }

    fn update_from_refs(&mut self, ops: &ObjStore) -> Result<UpdateMsg, DBError> {
        Err(DBError::NotFound)
    }
}

impl Position for LinearDimension {
    fn move_obj(&mut self, delta: &Vector3f) {

    }
}

