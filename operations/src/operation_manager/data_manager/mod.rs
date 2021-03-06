use crate::prelude::*;
use std::sync::Mutex;

#[cfg(test)]
mod tests;

mod database;
mod undo;

pub struct DataManager {
    db: database::FileDatabase,
    pending: undo::PendingEvents,
    undo: Mutex<undo::UndoStack>,
}

impl DataManager {
    pub fn new() -> DataManager {
        DataManager {
            db: database::FileDatabase::new(),
            pending: undo::PendingEvents::new(),
            undo: Mutex::new(undo::UndoStack::new()),
        }
    }

    pub fn open(path: &PathBuf) -> Result<(DataManager, Vec<RefID>), DBError> {
        let db = database::FileDatabase::new();
        let keys = db.open(path)?;
        Ok((
            DataManager {
                db: db,
                pending: undo::PendingEvents::new(),
                undo: Mutex::new(undo::UndoStack::new()),
            },
            keys,
        ))
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), DBError> {
        self.db.save(path)
    }

    pub fn begin_undo_event(&self, user: &UserID, event_id: UndoEventID, desc: String) -> Result<(), DBError> {
        self.pending.begin_event(user, event_id, desc)
    }

    pub fn end_undo_event(&self, event_id: UndoEventID) -> Result<(), DBError> {
        let mut stack = self.undo.lock().expect("Poisoned mutex");
        self.pending.end_event(&mut stack, &event_id)
    }

    pub fn suspend_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        self.pending.suspend_event(event_id)
    }

    pub fn resume_event(&self, event_id: &UndoEventID) -> Result<(), DBError> {
        self.pending.resume_event(event_id)
    }

    pub fn cancel_event(&self, event_id: &UndoEventID) -> Result<HashSet<RefID>, DBError> {
        self.pending.cancel_event(&self.db, event_id)
    }

    pub fn add_obj(&self, event_id: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
        self.pending.add_obj(&self.db, event_id, obj)
    }

    pub fn delete_obj(&self, event_id: &UndoEventID, key: &RefID) -> Result<DataObject, DBError> {
        self.pending.delete_obj(&self.db, event_id, key)
    }

    pub fn get_obj(&self, key: &RefID, callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.db.get(key, callback)
    }

    pub fn iterate_all(&self, callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.db.iterate_all(callback)
    }

    pub fn iterate_all_mut(&self, callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.db.iterate_all_mut(callback)
    }

    pub fn get_mut_obj(
        &self,
        event_id: &UndoEventID,
        key: &RefID,
        callback: impl FnMut(&mut DataObject) -> Result<(), DBError>,
    ) -> Result<(), DBError> {
        self.pending.get_mut_obj(&self.db, event_id, key, callback)
    }

    pub fn get_mut_obj_no_undo(&self, key: &RefID, callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        self.db.get_mut(key, callback)
    }

    pub fn duplicate_obj(&self, key: &RefID) -> Result<DataObject, DBError> {
        self.db.duplicate(key)
    }

    pub fn undo_latest(&self, user: &UserID) -> Result<HashSet<RefID>, DBError> {
        let mut stack = self.undo.lock().expect("Poisoned mutex");
        stack.undo_latest(user, &self.db)
    }

    pub fn take_undo_snapshot(&self, event_id: &UndoEventID, key: &RefID) -> Result<(), DBError> {
        self.pending.take_snapshot(&self.db, event_id, key)
    }

    pub fn redo_latest(&self, user: &UserID) -> Result<HashSet<RefID>, DBError> {
        let mut stack = self.undo.lock().expect("Poisoned mutex");
        stack.redo_latest(user, &self.db)
    }

    pub fn debug_state(&self, output: &mut String) {
        self.db.debug_state(output);
        output.push_str(&"\n");
        self.pending.debug_state(output);
        output.push_str(&"\n");
        self.undo.lock().expect("Poisoned mtuex").debug_state(output);
        output.push_str(&"\n");
    }
}
