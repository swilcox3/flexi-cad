mod data_manager;
mod dep_manager;
#[cfg(test)]
mod tests;

use crate::prelude::*;
use data_manager::*;
use dep_manager::*;
use crossbeam_channel::Sender;
use ccl::dhashmap::DHashMap;

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
        let data = DataManager::open(path)?;
        let ops = OperationManager {
            data: data,
            deps: DependencyManager::new(),
            updates: DHashMap::default(),
        };
        ops.updates.insert(user.clone(), sender);
        ops.data.iterate_all_mut(&mut |obj: &mut DataObject| {
            if let Some(dep_obj) = obj.query_ref::<dyn UpdateFromRefs>() {
                let refs = dep_obj.get_refs();
                for ref_opt in refs {
                    if let Some(refer) = ref_opt {
                        ops.deps.register_sub(&refer.id, obj.get_id().clone());
                    }
                }
            }
            let msg = obj.update()?;
            ops.send(msg, Some(&user))
        })?;
        Ok(ops)
    }

    pub fn send(&self, msg: UpdateMsg, only_to: Option<&UserID>) -> Result<(), DBError> {
        if let Some(user) = only_to {
            if let Some(upd) = self.updates.get(user) {
                upd.send(msg).map_err(error_other)
            }
            else {
                Err(DBError::UserNotFound)
            }
        }
        else {
            for chunk in self.updates.chunks() {
                for (_, upd) in chunk.iter() {
                    upd.send(msg.clone()).map_err(error_other)?;
                }
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
        let mut set = HashSet::new();
        self.data.iterate_all(&mut |obj: &DataObject| {
            set.insert(obj.get_id().clone());
            Ok(())
        })?;
        self.update_set(set, only_to)
    }

    fn update_set<T>(&self, set: T, only_to: Option<&UserID>) -> Result<(), DBError> 
        where T: IntoIterator<Item = RefID>
    {
        for obj_id in set.into_iter() {
            if let Err(e) = self.data.get_mut_obj_no_undo(&obj_id, &mut |obj: &mut DataObject| {
                let msg = obj.update()?;
                self.send(msg, only_to)
            }) {
                match e {
                    DBError::ObjNotFound => self.send(UpdateMsg::Delete{key: obj_id}, None)?,
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
                    match obj.query_ref::<dyn ReferTo>() {
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
            if let Some(updatable) = obj.query_ref::<dyn UpdateFromRefs>() {
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
            match obj.query_mut::<dyn UpdateFromRefs>() {
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

    fn update_set_from_refs<T>(&self, deps: T) -> Result<(), DBError> 
        where T: IntoIterator<Item = RefID> + std::fmt::Debug
    {
        println!("deps: {:?}", deps);
        for dep_id in deps.into_iter() {
            match self.update_from_refs(&dep_id) {
                Ok(msg) => {
                    self.send(msg, None)?;
                }
                Err(DBError::ObjNotFound) => {
                    self.send(UpdateMsg::Delete{key: dep_id.clone()}, None)?;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn update_deps(&self, id: &RefID) -> Result<(), DBError> {
        let deps = self.deps.get_all_deps(vec![id.clone()]);
        self.update_set_from_refs(deps)
    }

    pub fn update_all_deps<T>(&self, ids: T) -> Result<(), DBError>
        where T: IntoIterator<Item = RefID>
    {
        let deps = self.deps.get_all_deps(ids);
        self.update_set_from_refs(deps)
    }

    pub fn add_dep(&self, publisher: &RefID, sub: RefID) {
        self.deps.register_sub(publisher, sub);
    }

    pub fn remove_dep(&self, publisher: &RefID, sub: &RefID) {
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

    pub fn add_object(&self, event: &UndoEventID, mut obj: DataObject) -> Result<(), DBError> {
        if let Some(dep_obj) = obj.query_ref::<dyn UpdateFromRefs>() {
            let refs = dep_obj.get_refs();
            for ref_opt in refs {
                if let Some(refer) = ref_opt {
                    self.deps.register_sub(&refer.id, obj.get_id().clone());
                }
            }
        }
        let msg = obj.update()?;
        self.data.add_obj(event, obj)?;
        self.send(msg, None)
    }

    pub fn delete_obj(&self, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
        let obj = self.data.delete_obj(event, id)?;
        self.send(UpdateMsg::Delete{key: *id}, None)?;
        self.update_deps(id)?;
        self.deps.delete_obj(id);
        Ok(obj)
    }

    pub fn modify_obj(&self, event: &UndoEventID, id: &RefID, mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_mut_obj(event, id, |mut obj| {
            callback(&mut obj)?;
            self.send(obj.update()?, None)?;
            Ok(())
        })
    }

    pub fn get_obj(&self, id: &RefID, callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.data.get_obj(id, callback)
    }
}