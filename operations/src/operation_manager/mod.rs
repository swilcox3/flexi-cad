mod data_manager;
mod dep_manager;
#[cfg(test)]
mod tests;

use crate::*;
use data_manager::*;
use dep_manager::*;
use crossbeam_channel::Sender;
use std::collections::HashSet;

pub struct OperationManager {
    data: DataManager,
    deps: DependencyManager,
    updates: Sender<UpdateMsg>,
}

impl OperationManager {
    pub fn new(sender: Sender<UpdateMsg>) -> OperationManager {
        OperationManager {
            data: DataManager::new(),
            deps: DependencyManager::new(),
            updates: sender,
        }
    }

    pub fn begin_undo_event(&self, user_id: &UserID, desc: String) -> Result<UndoEventID, DBError> {
        self.data.begin_undo_event(user_id, desc)
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
        self.update_set(&set)?;
        self.update_all_deps(set.iter())
    }

    pub fn take_undo_snapshot(&self, event_id: &UndoEventID, key: &RefID) -> Result<(), DBError> {
        self.data.take_undo_snapshot(event_id, key)
    }

    pub fn undo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.undo_latest(user)?;
        self.update_set(&set)?;
        self.update_all_deps(set.iter())
    }

    pub fn redo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.redo_latest(user)?;
        self.update_set(&set)?;
        self.update_all_deps(set.iter())
    }


    fn update_set(&self, set: &HashSet<RefID>) -> Result<(), DBError> {
        //println!("{:?}", set);
        for obj_id in set {
            if let Err(e) = self.data.get_mut_obj_no_undo(&obj_id, &mut |obj: &mut DataObject| {
                self.updates.send(obj.update()?).unwrap();
                Ok(())
            }) {
                match e {
                    DBError::NotFound => self.updates.send(UpdateMsg::Delete{key: *obj_id}).unwrap(),
                    _ => return Err(e)
                }
            }
        }
        Ok(())
    }

    fn update_set_from_refs(&self, deps: &HashSet<RefID>) -> Result<(), DBError> {
        for dep_id in deps {
            if let Err(e) = self.data.get_mut_obj_no_undo(&dep_id, &mut |dep_obj: &mut DataObject| {
                if let Some(to_update) = dep_obj.query_mut::<Update>() {
                    self.updates.send(to_update.update_from_refs(self)?).unwrap();
                }
                Ok(())
            }) {
                match e {
                    DBError::NotFound => self.updates.send(UpdateMsg::Delete{key: *dep_id}).unwrap(),
                    _ => return Err(e)
                }
            }
        }
        Ok(())
    }

    pub fn update_deps(&self, id: &RefID) -> Result<(), DBError> {
        let deps = self.deps.get_deps(id);
        self.update_set_from_refs(&deps)
    }

    pub fn update_all_deps<'a>(&self, ids: impl Iterator<Item=&'a RefID>) -> Result<(), DBError>{
        let deps = self.deps.get_all_deps(ids);
        self.update_set_from_refs(&deps)
    }

    pub fn add_dep(&self, publisher: &RefID, sub: RefID) {
        self.deps.register_sub(publisher, sub);
    }

    pub fn remove_dep(&self, publisher: &RefID, sub: &RefID) {
        self.deps.delete_sub(publisher, sub);
    }

}

impl ObjStore for OperationManager {
    fn add_object(&self, event: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
        if let Some(dep_obj) = obj.query_ref::<Update>() {
            dep_obj.init(&self.deps);
        }
        let msg = obj.update()?;
        self.data.add_obj(event, obj)?;
        self.updates.send(msg).unwrap();
        Ok(())
    }

    fn delete_obj(&self, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
        let obj = self.data.delete_obj(event, id)?;
        self.updates.send(UpdateMsg::Delete{key: *id}).unwrap();
        self.update_deps(id)?;
        self.deps.delete_obj(id);
        Ok(obj)
    }

    fn modify_obj(&self, event: &UndoEventID, id: &RefID, callback: &mut FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_mut_obj(event, id, &mut |mut obj| {
            callback(&mut obj)?;
            self.updates.send(obj.update()?).unwrap();
            Ok(())
        })
    }

    fn get_obj(&self, id: &RefID, callback: &mut FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_obj(id, callback)
    }
}