mod data_manager;
mod dep_manager;
#[cfg(test)]
mod tests;

use crate::*;
use data_manager::*;
use dep_manager::*;
use crossbeam_channel::Sender;

pub struct OperationManager {
    data: DataManager,
    deps: DependencyManager,
    pub updates: Vec<Sender<UpdateMsg>>,
}

impl OperationManager {
    pub fn new(sender: Sender<UpdateMsg>) -> OperationManager {
        OperationManager {
            data: DataManager::new(),
            deps: DependencyManager::new(),
            updates: vec![sender],
        }
    }

    pub fn open(path: &PathBuf, sender: Sender<UpdateMsg>) -> Result<OperationManager, DBError> {
        let data = DataManager::open(path)?;
        let ops = OperationManager {
            data: data,
            deps: DependencyManager::new(),
            updates: vec![sender],
        };
        ops.data.iterate_all(&mut |obj: &DataObject| {
            if let Some(dep_obj) = obj.query_ref::<UpdateFromRefs>() {
                let refs = dep_obj.get_refs();
                for ref_opt in refs {
                    if let Some(refer) = ref_opt {
                        ops.deps.register_sub(&refer.id, obj.get_id().clone());
                    }
                }
            }
            let msg = obj.update()?;
            self.send(msg);
            Ok(())
        })?;
        Ok(ops)
    }

    fn send(&self, msg: UpdateMsg) {
        for upd in &self.updates {
            upd.send(msg.clone()).unwrap();
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), DBError> {
        self.data.save(path)
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
        let deps: Vec<RefID> = set.into_iter().collect();
        self.update_all_deps(&deps)
    }

    pub fn take_undo_snapshot(&self, event_id: &UndoEventID, key: &RefID) -> Result<(), DBError> {
        self.data.take_undo_snapshot(event_id, key)
    }

    pub fn undo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.undo_latest(user)?;
        self.update_set(&set)?;
        let deps: Vec<RefID> = set.into_iter().collect();
        self.update_all_deps(&deps)
    }

    pub fn redo_latest(&self, user: &UserID) -> Result<(), DBError> {
        let set = self.data.redo_latest(user)?;
        self.update_set(&set)?;
        let deps: Vec<RefID> = set.into_iter().collect();
        self.update_all_deps(&deps)
    }

    pub fn update_all(&self) -> Result<(), DBError> {
        let mut set = HashSet::new();
        self.data.iterate_all(&mut |obj: &DataObject| {
            set.insert(obj.get_id().clone());
            Ok(())
        })?;
        self.update_set(&set)
    }

    fn update_set(&self, set: &HashSet<RefID>) -> Result<(), DBError> {
        for obj_id in set {
            if let Err(e) = self.data.get_mut_obj_no_undo(&obj_id, &mut |obj: &mut DataObject| {
                let msg = obj.update()?;
                self.send(msg);
                Ok(())
            }) {
                match e {
                    DBError::NotFound => self.send(UpdateMsg::Delete{key: *obj_id}).unwrap(),
                    _ => return Err(e)
                }
            }
        }
        Ok(())
    }

    fn get_ref_result(&self, refer_opt: &Option<Reference>) -> Option<RefGeometry> {
        match refer_opt {
            Some(refer) => {
                let mut result = None;
                match self.get_obj(&refer.id, |obj| {
                    match obj.query_ref::<ReferTo>() {
                        Some(update_from) => {
                            result = update_from.get_result(refer.index);
                            Ok(())
                        }
                        None => Err(DBError::ObjLacksTrait)
                    }
                }) {
                    Ok(()) => result,
                    Err(_) => None,
                }
            }
            None => None,
        }
    }

    fn update_from_refs(&self, obj_id: &RefID) -> Result<UpdateMsg, DBError> {
        let mut refs = Vec::new();
        self.get_obj(obj_id, &mut |obj: &DataObject| {
            if let Some(updatable) = obj.query_ref::<UpdateFromRefs>() {
                refs = updatable.get_refs();
            }
            Ok(())
        })?;
        let mut results = Vec::new();
        for refer in refs {
            results.push(self.get_ref_result(&refer));
        }
        let mut msg = UpdateMsg::Empty;
        self.data.get_mut_obj_no_undo(obj_id, |obj| {
            match obj.query_mut::<UpdateFromRefs>() {
                Some(updatable) => {
                    updatable.update_from_refs(&results);
                    msg = obj.update()?;
                    Ok(())
                }
                None => Err(DBError::ObjLacksTrait)
            }
        })?;
        Ok(msg)
    }

    fn update_set_from_refs(&self, deps: &HashSet<RefID>) -> Result<(), DBError> {
        for dep_id in deps {
            match self.update_from_refs(&dep_id) {
                Ok(msg) => {
                    self.send(msg).unwrap();
                }
                Err(DBError::NotFound) => {
                    self.send(UpdateMsg::Delete{key: dep_id.clone()}).unwrap();
                }
                Err(DBError::ObjLacksTrait) => {
                    //Check other update traits
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn update_deps(&self, id: &RefID) -> Result<(), DBError> {
        let deps = self.deps.get_deps(id);
        self.update_set_from_refs(&deps)
    }

    pub fn update_all_deps<'a>(&self, ids: &Vec<RefID>) -> Result<(), DBError>{
        let deps = self.deps.get_all_deps(ids);
        self.update_set_from_refs(&deps)
    }

    pub fn add_dep(&self, publisher: &RefID, sub: RefID) {
        self.deps.register_sub(publisher, sub);
    }

    pub fn remove_dep(&self, publisher: &RefID, sub: &RefID) {
        self.deps.delete_sub(publisher, sub);
    }

    pub fn copy_obj(&self, event: &UndoEventID, id: &RefID) -> Result<RefID, DBError> {
        let mut copy = self.data.duplicate_obj(id)?;
        if let Some(updatable) = copy.query_mut::<UpdateFromRefs>() {
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
        if let Some(dep_obj) = obj.query_ref::<UpdateFromRefs>() {
            let refs = dep_obj.get_refs();
            for ref_opt in refs {
                if let Some(refer) = ref_opt {
                    self.deps.register_sub(&refer.id, obj.get_id().clone());
                }
            }
        }
        let msg = obj.update()?;
        self.data.add_obj(event, obj)?;
        self.send(msg).unwrap();
        Ok(())
    }

    pub fn delete_obj(&self, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
        let obj = self.data.delete_obj(event, id)?;
        self.send(UpdateMsg::Delete{key: *id}).unwrap();
        self.update_deps(id)?;
        self.deps.delete_obj(id);
        Ok(obj)
    }

    pub fn modify_obj(&self, event: &UndoEventID, id: &RefID, mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_mut_obj(event, id, |mut obj| {
            callback(&mut obj)?;
            self.send(obj.update()?).unwrap();
            Ok(())
        })
    }

    pub fn get_obj(&self, id: &RefID, callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_obj(id, callback)
    }
}