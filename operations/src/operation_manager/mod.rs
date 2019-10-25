mod data_manager;
mod dep_manager;
#[cfg(test)]
mod tests;

use crate::prelude::*;
use ccl::dhashmap::DHashMap;
use crossbeam_channel::Sender;
use data_manager::*;
use dep_manager::*;

pub struct OperationManager {
    data: DataManager,
    deps: DependencyManager,
    pub updates: DHashMap<UserID, Sender<UpdateMsg>>,
}

impl OperationManager {
    pub fn new(user: UserID, sender: Sender<UpdateMsg>) -> OperationManager {
        let ops = OperationManager {
            data: DataManager::new(),
            deps: DependencyManager::new(),
            updates: DHashMap::default(),
        };
        ops.updates.insert(user, sender);
        ops
    }

    pub fn open(path: &PathBuf, user: UserID, sender: Sender<UpdateMsg>) -> Result<OperationManager, DBError> {
        let (data, keys) = DataManager::open(path)?;
        info!("File is opened");
        let ops = OperationManager {
            data: data,
            deps: DependencyManager::new(),
            updates: DHashMap::default(),
        };
        ops.updates.insert(user.clone(), sender);
        keys.par_iter().for_each(|key| {
            if let Err(e) = ops.data.get_mut_obj_no_undo(key, |obj| {
                ops.register_deps(&obj);
                match obj.update() {
                    Ok(msg) => {
                        if let Err(e) = ops.send(msg, Some(&user)) {
                            error!("Error sending update: {:?}", e);
                        }
                    }
                    Err(e) => error!("Error updating object {:?}", e),
                }
                Ok(())
            }) {
                error!("Error getting object {:?}", e);
            }
        });
        Ok(ops)
    }

    pub fn send(&self, msg: UpdateMsg, only_to: Option<&UserID>) -> Result<(), DBError> {
        if let Some(user) = only_to {
            if let Some(upd) = self.updates.get(user) {
                upd.send(msg).map_err(error_other)
            } else {
                Err(DBError::UserNotFound)
            }
        } else {
            let mut to_remove = Vec::new();
            for chunk in self.updates.chunks() {
                for (key, upd) in chunk.iter() {
                    if let Err(_) = upd.send(msg.clone()) {
                        to_remove.push(key.clone());
                    }
                }
            }
            for delete in to_remove {
                self.updates.remove(&delete);
            }
            Ok(())
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), DBError> {
        self.data.save(path)
    }

    pub fn begin_undo_event(&self, user_id: &UserID, event_id: UndoEventID, desc: String) -> Result<(), DBError> {
        self.data.begin_undo_event(user_id, event_id, desc)
    }

    pub fn end_undo_event(&self, event: UndoEventID) -> Result<(), DBError> {
        self.data.end_undo_event(event)
    }

    pub fn suspend_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        self.data.suspend_event(event_id)
    }

    pub fn resume_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        self.data.resume_event(event_id)
    }

    pub fn cancel_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        let set = self.data.cancel_event(event_id)?;
        self.update_all_deps(set)
    }

    pub fn take_undo_snapshot(&self, event_id: &UndoEventID, key: &RefID) -> Result<(), DBError> {
        self.data.take_undo_snapshot(event_id, key)
    }

    pub fn undo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.undo_latest(user)?;
        self.update_all_deps(set)
    }

    pub fn redo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.redo_latest(user)?;
        self.update_all_deps(set)
    }

    pub fn update_all(&self, only_to: Option<&UserID>) -> Result<(), DBError> {
        self.data.iterate_all_mut(&mut |obj: &mut DataObject| {
            let msg = obj.update()?;
            self.send(msg, only_to)
        })
    }

    fn get_ref_result(&self, refer: &GeometryId) -> Option<RefGeometry> {
        let mut result = None;
        match self.get_obj(&refer.id, |obj| match obj.query_ref::<dyn ReferTo>() {
            Some(update_from) => {
                result = update_from.get_result(refer.index);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait),
        }) {
            Ok(()) => result,
            Err(_) => None,
        }
    }

    fn update_reference(&self, refer: &Reference) -> Result<(), DBError> {
        if refer.owner.id != refer.other.id {
            let result = self.get_ref_result(&refer.other);
            self.data
                .get_mut_obj_no_undo(&refer.owner.id, |obj| match obj.query_mut::<dyn UpdateFromRefs>() {
                    Some(updatable) => {
                        updatable.set_associated_geom(refer.owner.index, &result);
                        let update_msg = obj.update();
                        match update_msg {
                            Ok(msg) => self.send(msg, None),
                            Err(DBError::ObjNotFound) => self.send(UpdateMsg::Delete { key: obj.get_id().clone() }, None),
                            Err(e) => Err(e),
                        }
                    }
                    None => Err(DBError::ObjLacksTrait),
                })
        } else {
            Ok(())
        }
    }

    fn update_reference_set<T>(&self, refers: T) -> Result<(), DBError>
    where
        T: IntoIterator<Item = Reference>,
    {
        for refer in refers {
            self.update_reference(&refer)?;
        }
        Ok(())
    }

    fn update_set_from_refs<T>(&self, deps: T) -> Result<(), DBError>
    where
        T: IntoIterator<Item = RefID>,
    {
        let mut geom_ids = Vec::new();
        let mut to_remove = HashSet::new();
        for dep_id in deps.into_iter() {
            let for_each = |obj: &mut DataObject| {
                self.send(obj.update()?, None)?;
                match obj.query_ref::<dyn ReferTo>() {
                    Some(referrable) => {
                        for i in 0..referrable.get_num_results() {
                            geom_ids.push(GeometryId {
                                id: dep_id.clone(),
                                index: i,
                            });
                        }
                        Ok(())
                    }
                    None => Err(DBError::ObjLacksTrait),
                }
            };
            if let Err(e) = self.data.get_mut_obj_no_undo(&dep_id, for_each) {
                match e {
                    DBError::ObjNotFound => {
                        to_remove.insert(dep_id);
                    }
                    DBError::ObjLacksTrait => (),
                    _ => {
                        return Err(e);
                    }
                }
            }
        }
        if to_remove.len() > 0 {
            for delete in &to_remove {
                self.send(UpdateMsg::Delete { key: delete.clone() }, None)?;
            }
            self.deps.delete_ids(to_remove);
        }
        if geom_ids.len() > 0 {
            let refers = self.deps.get_all_deps(geom_ids);
            if refers.len() > 0 {
                self.update_reference_set(refers)?;
            }
        }
        Ok(())
    }

    pub fn update_deps(&self, id: &RefID) -> Result<(), DBError> {
        self.update_set_from_refs(vec![id.clone()])
    }

    pub fn update_all_deps<T>(&self, ids: T) -> Result<(), DBError>
    where
        T: IntoIterator<Item = RefID>,
    {
        self.update_set_from_refs(ids)
    }

    pub fn register_deps(&self, obj: &DataObject) {
        if let Some(dep_obj) = obj.query_ref::<dyn UpdateFromRefs>() {
            let refs = dep_obj.get_refs();
            for ref_opt in refs {
                if let Some(refer) = ref_opt {
                    self.deps.register_sub(&refer.other, refer.owner);
                }
            }
        }
    }

    pub fn add_deps(&self, id: &RefID) -> Result<(), DBError> {
        self.get_obj(&id, |obj| {
            self.register_deps(obj);
            Ok(())
        })
    }

    pub fn remove_dep(&self, publisher: &GeometryId, sub: &GeometryId) {
        self.deps.delete_sub(publisher, sub);
    }

    pub fn copy_obj(&self, event: &UndoEventID, id: &RefID) -> Result<RefID, DBError> {
        let mut copy = self.data.duplicate_obj(id)?;
        if let Some(updatable) = copy.query_mut::<dyn UpdateFromRefs>() {
            updatable.clear_refs();
        }
        let copy_id = copy.get_id().clone();
        self.add_object(event, copy)?;
        Ok(copy_id)
    }

    pub fn debug_state(&self, output: &mut String) {
        self.data.debug_state(output);
        output.push_str(&"\n");
        self.deps.debug_state(output);
        output.push_str(&"\n");
    }

    pub fn add_object(&self, event: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
        self.register_deps(&obj);
        self.data.add_obj(event, obj)
    }

    pub fn delete_obj(&self, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
        let obj = self.data.delete_obj(event, id)?;
        self.send(UpdateMsg::Delete { key: *id }, None)?;
        self.update_deps(id)?;
        if let Some(refer_obj) = obj.query_ref::<dyn ReferTo>() {
            let num_pts = refer_obj.get_num_results();
            for i in 0..num_pts {
                self.deps.delete_id(&GeometryId { id: id.clone(), index: i });
            }
        }
        Ok(obj)
    }

    pub fn modify_obj(
        &self,
        event: &UndoEventID,
        id: &RefID,
        mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>,
    ) -> Result<(), DBError> {
        self.data.get_mut_obj(event, id, |mut obj| {
            callback(&mut obj)?;
            Ok(())
        })
    }

    pub fn get_obj(&self, id: &RefID, callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_obj(id, callback)
    }
}
