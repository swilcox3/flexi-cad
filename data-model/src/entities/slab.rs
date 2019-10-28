use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slab {
    id: RefID,
}

interfaces!(
    Slab: dyn query_interface::ObjectClone,
    dyn std::fmt::Debug,
    dyn Data,
    dyn ReferTo,
    dyn Position,
    dyn UpdateFromRefs
);

impl Slab {
    pub fn new() -> Slab {
        let id = RefID::new_v4();
        Slab { id: id }
    }
}

#[typetag::serde]
impl Data for Slab {
    fn get_id(&self) -> &RefID {
        &self.id
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&mut self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: Some(json! ({
                "type": "Wall",
                "traits": ["ReferTo", "Position", "UpdateFromRefs"],
                "obj": {
                }
            })),
        };
        Ok(UpdateMsg::Mesh { data: data })
    }

    fn get_temp_repr(&self) -> Result<UpdateMsg, DBError> {
        let mut data = MeshData {
            id: self.get_id().clone(),
            positions: Vec::with_capacity(24),
            indices: Vec::with_capacity(36),
            metadata: None,
        };
        Ok(UpdateMsg::Mesh { data: data })
    }

    fn get_data(&self, prop_name: &str) -> Result<serde_json::Value, DBError> {
        match prop_name {
            _ => Err(DBError::PropertyNotFound),
        }
    }

    fn set_data(&mut self, mut data: serde_json::Value) -> Result<(), DBError> {
        let mut changed = false;
        if changed {
            Ok(())
        } else {
            Err(DBError::PropertyNotFound)
        }
    }
}

impl ReferTo for Slab {
    fn get_result(&self, result: ResultInd) -> Option<RefGeometry> {
        None
    }

    fn get_all_results(&self) -> Vec<RefGeometry> {
        let mut results = Vec::new();
        results
    }

    fn get_num_results(&self) -> usize {
        0
    }
}

impl UpdateFromRefs for Slab {
    fn clear_refs(&mut self) {}

    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        results
    }

    fn get_num_refs(&self) -> usize {
        0
    }

    fn get_available_refs(&self) -> Vec<ReferInd> {
        let mut results = Vec::new();
        results
    }

    fn set_ref(
        &mut self,
        index: ReferInd,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) {
    }

    fn add_ref(
        &mut self,
        result: &RefGeometry,
        other_ref: GeometryId,
        snap_pt: &Option<Point3f>,
    ) -> bool {
        false
    }

    fn delete_ref(&mut self, index: ReferInd) {}

    fn get_associated_geom(&self, index: ReferInd) -> Option<RefGeometry> {
        None
    }

    fn set_associated_geom(&mut self, index: ReferInd, geom: &Option<RefGeometry>) {}
}

impl Position for Slab {
    fn move_obj(&mut self, delta: &Vector3f) {}
}
