use crate::*;
use super::database::FileDatabase;
use std::time::{SystemTime, UNIX_EPOCH};
use ccl::dhashmap::DHashMap;
use std::collections::HashSet;

fn get_time() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Backwards time?").as_millis()
}

#[derive(Debug, Clone)]
pub enum Change {
    Add{key: RefID},
    Modify{obj: DataObject},
    Delete{obj: DataObject}
}

#[derive(Debug, Clone)]
pub struct UndoEvent {
    event_id: UndoEventID,
    user_id: UserID,
    pub changes: Vec<Change>,
    timestamp: u128,
    desc: String,
    nested: usize,
    suspended: usize,
}

impl UndoEvent {
    fn new(user: UserID, desc: String) -> UndoEvent {
        UndoEvent {
            event_id: RefID::new_v4(),
            user_id: user,
            changes: Vec::new(),
            timestamp: get_time(),
            desc: desc,
            nested: 0,
            suspended: 0,
        }
    }

    fn get_changed_objects(&self) -> HashSet<RefID> {
        let mut results = HashSet::new();
        for change in &self.changes {
            match change {
                Change::Add{key} => results.insert(key.clone()),
                Change::Modify{obj} => results.insert(obj.get_id().clone()),
                Change::Delete{obj} => results.insert(obj.get_id().clone()),
            };
        }
        results
    }
}

pub struct UndoStack {
    stack: std::collections::VecDeque<UndoEvent>,
    redo_stack: std::collections::VecDeque<UndoEvent>
}

impl UndoStack {
    pub fn new() -> UndoStack {
        UndoStack{ 
            stack: std::collections::VecDeque::new(),
            redo_stack: std::collections::VecDeque::new()
        }
    }

    pub fn undo_latest(&mut self, user_id: &UserID, db: &FileDatabase) -> Result<HashSet<RefID>, DBError> {
        for i in (0..self.stack.len()).rev() {
            if let Some(event) = self.stack.get(i) {
                if event.user_id == *user_id {
                    if let Some(event) = self.stack.remove(i) {
                        let redo = db.undo(event)?;
                        let objs = redo.get_changed_objects();
                        self.redo_stack.push_back(redo);
                        return Ok(objs);
                    }
                }
            }
        }
        Err(DBError::NoUndoEvent)
    }

    pub fn redo_latest(&mut self, user_id: &UserID, db: &FileDatabase) -> Result<HashSet<RefID>, DBError> {
        for i in (0..self.redo_stack.len()).rev() {
            if let Some(event) = self.redo_stack.get(i) {
                if event.user_id == *user_id {
                    if let Some(event) = self.redo_stack.remove(i) {
                        let undo = db.undo(event)?;
                        let objs = undo.get_changed_objects();
                        self.stack.push_back(undo);
                        return Ok(objs);
                    }
                }
            }
        }
        Err(DBError::NoUndoEvent)
    }
}

pub struct PendingEvents {
    events: DHashMap<UndoEventID, UndoEvent>,
}

impl PendingEvents {
    pub fn new() -> PendingEvents {
        PendingEvents{ events: DHashMap::default() }
    }

    pub fn begin_event(&self, user: &UserID, desc: String) -> Result<UndoEventID, DBError> {
        match self.events.get_mut(user) {
            Some(mut event) => {
                event.nested = event.nested + 1;
                Ok(event.event_id)
            }
            None => {
                let event = UndoEvent::new(user.clone(), desc);
                let id = event.event_id.clone();
                self.events.insert(event.event_id, event);
                Ok(id)
            }
        }
    }

    #[allow(unused_assignments)]
    pub fn end_event(&self, undo_stack: &mut UndoStack, event_id: &UndoEventID) -> Result<(), DBError> {
        let mut remove = false;
        match self.events.get_mut(event_id) {
            Some(mut event) => {
                if event.nested > 0 {
                    event.nested = event.nested - 1;
                    return Ok(());
                }
                else {
                    remove = true;
                }
            }
            None => {
                return Err(DBError::NoUndoEvent);
            }
        }
        if remove {
            match self.events.remove(event_id) {
                Some(event) => {
                    undo_stack.stack.push_back(event.1);
                    return Ok(())
                }
                None => return Err(DBError::NoUndoEvent)
            }
        }
        Ok(())
    }

    pub fn take_snapshot(&self, db: &FileDatabase, event_id: &UndoEventID, obj_id: &RefID) -> Result<(), DBError> {
        match self.events.get_mut(event_id) {
            Some(mut event) => {
                db.get(obj_id, &mut |obj: &DataObject| {
                    if event.suspended == 0 {
                        event.changes.push(Change::Modify{obj: obj.clone()});
                    }
                    Ok(())
                })
            }
            None => Err(DBError::NotFound)
        }
    }

    pub fn suspend_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        match self.events.get_mut(event_id) {
            Some(mut event) => {
                event.suspended = event.suspended + 1;
                Ok(())
            }
            None => Err(DBError::NotFound)
        }
    }

    pub fn resume_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        match self.events.get_mut(event_id) {
            Some(mut event) => {
                if event.suspended > 0 {
                    event.suspended = event.suspended - 1;
                }
                Ok(())
            }
            None => Err(DBError::NotFound)
        }
    }

    pub fn cancel_event(&self, db: &FileDatabase, event_id: &UndoEventID) -> Result<HashSet<RefID>, DBError> {
        match self.events.remove(event_id) {
            Some((_, event)) => {
                let redo = db.undo(event)?;
                Ok(redo.get_changed_objects())
            }
            None => Err(DBError::NotFound)
        }
    }

    pub fn add_obj(&self, db: &FileDatabase, event_id: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
        match self.events.get_mut(event_id) {
            Some(mut event) => {
                let key = obj.get_id().clone();
                match db.add(obj) {
                    Ok(()) => {
                        if event.suspended == 0 {
                            event.changes.push(Change::Add{key: key});
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            None => {
                Err(DBError::NoUndoEvent)
            }
        }
    }

    pub fn delete_obj(&self, db: &FileDatabase, event_id: &UndoEventID, key: &RefID) -> Result<DataObject, DBError> {
        if !self.events.contains_key(event_id) {
            Err(DBError::NoUndoEvent)
        }
        else {
            let obj = db.remove(key)?;
            match self.events.get_mut(event_id) {
                Some(mut event) => {
                    if event.suspended == 0 {
                        event.changes.push(Change::Delete{obj: obj.clone()});
                    }
                    Ok(obj)
                }
                None => Err(DBError::NoUndoEvent),
            }
        }
    }

    pub fn get_mut_obj(&self, db: &FileDatabase, event_id: &UndoEventID, key: &RefID, callback: &mut FnMut(&mut DataObject)->Result<(), DBError>) -> Result<(), DBError> {
        match self.events.get_mut(&event_id) {
            Some(mut event) => {
                db.get(key, &mut |obj: &DataObject| {
                    if event.suspended == 0 {
                        event.changes.push(Change::Modify{obj: obj.clone()});
                    }
                    Ok(())
                })?;
                db.get_mut(key, callback)
            }
            None => Err(DBError::NoUndoEvent),
        }
    }
}








