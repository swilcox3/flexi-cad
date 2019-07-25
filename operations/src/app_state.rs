use ccl::dhashmap::DHashMap;
use std::path::PathBuf;
use crate::*;
use crossbeam_channel::Sender;
use crate::scheduler::Scheduler;
use operation_manager::OperationManager;

lazy_static!{
    static ref APP_STATE: AppState = AppState::new();
}

pub struct AppState {
    files: DHashMap<PathBuf, OperationManager>,
    user_id: UserID
}

impl AppState {
    fn new() -> AppState {
        AppState {
            files: DHashMap::default(),
            user_id: UserID::new_v4()
        }
    }
}

pub fn init_file(file: PathBuf, updates: Sender<UpdateMsg>) {
    APP_STATE.files.insert(file, OperationManager::new(updates));
}

pub fn open_file(file: PathBuf, updates: Sender<UpdateMsg>) -> Result<(), DBError> {
    let ops = OperationManager::open(&file, updates)?;
    APP_STATE.files.insert(file.clone(), ops);
    Scheduler::spawn(move || {
        match APP_STATE.files.get(&file) {
            Some(ops) => ops.update_all(),
            None => Err(DBError::NotFound)
        }
    });
    Ok(())
}

pub fn save_file(file: &PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.save(file),
        None => Err(DBError::NotFound)
    }
}

pub fn save_as_file(orig_file: &PathBuf, file_new: PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.remove(orig_file) {
        Some((_, ops)) => {
            match ops.save(&file_new) {
                Ok(()) => {
                    APP_STATE.files.insert(file_new, ops);
                    Ok(())
                }
                Err(e) => {
                    APP_STATE.files.insert(orig_file.clone(), ops);
                    Err(e)
                }
            }
        }
        None => Err(DBError::NotFound)
    }
}

pub fn begin_undo_event(file: &PathBuf, desc: String) -> Result<UndoEventID, DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.begin_undo_event(&APP_STATE.user_id, desc),
        None => Err(DBError::NotFound)
    }
}

pub fn end_undo_event(file: &PathBuf, event: UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.end_undo_event(event),
        None => Err(DBError::NotFound)
    }
}

pub fn undo_latest(file: &PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.undo_latest(&APP_STATE.user_id),
        None => Err(DBError::NotFound)
    }
}

pub fn redo_latest(file: &PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.redo_latest(&APP_STATE.user_id),
        None => Err(DBError::NotFound)
    }
}

pub fn suspend_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.suspend_event(event),
        None => Err(DBError::NotFound)
    }
}

pub fn resume_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.resume_event(event),
        None => Err(DBError::NotFound)
    }
}

pub fn cancel_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.cancel_event(event),
        None => Err(DBError::NotFound)
    }
}

pub fn take_undo_snapshot(file: &PathBuf, event: &UndoEventID, key: &RefID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.take_undo_snapshot(event, key),
        None => Err(DBError::NotFound)
    }
}

pub fn add_obj(file: &PathBuf, event: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.add_object(&event, obj),
        None => Err(DBError::NotFound)
    }
}

pub fn get_obj(file: &PathBuf, id: &RefID, mut callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.get_obj(id, &mut callback),
        None => Err(DBError::NotFound)
    }
}

pub fn modify_obj(file: &PathBuf, event: &UndoEventID, id: &RefID, mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.modify_obj(event, id, &mut callback),
        None => Err(DBError::NotFound)
    }
}

pub fn delete_obj(file: &PathBuf, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.delete_obj(event, id),
        None => Err(DBError::NotFound)
    }
}

pub fn update_deps(file: PathBuf, id: RefID) {
    Scheduler::spawn(move || {
        match APP_STATE.files.get(&file) {
            Some(ops) => {
                ops.update_deps(&id)
            }
            None => Err(DBError::NotFound)
        }
    });
}

pub fn update_all_deps(file: PathBuf, ids: Vec<RefID>) {
    Scheduler::spawn(move || {
        match APP_STATE.files.get(&file) {
            Some(ops) => {
                ops.update_all_deps(ids.iter())
            }
            None => Err(DBError::NotFound)
        }
    });
}

pub fn add_dep(file: &PathBuf, publisher: &RefID, subscriber: RefID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => Ok(ops.add_dep(publisher, subscriber)),
        None => Err(DBError::NotFound)
    }
}

pub fn remove_dep(file: &PathBuf, publisher: &RefID, subscriber: &RefID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => Ok(ops.remove_dep(publisher, subscriber)),
        None => Err(DBError::NotFound)
    }
}

pub fn copy_obj(file: &PathBuf, event: &UndoEventID, id: &RefID) -> Result<RefID, DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.copy_obj(event, id),
        None => Err(DBError::NotFound)
    }
}

pub fn debug_state() -> String {
    let mut total = String::from("");
    for chunk in APP_STATE.files.chunks() {
        for (path, ops) in chunk.iter() {
            total.push_str(&format!("File: {:?}\n", path));
            ops.debug_state(&mut total);
        }
    }
    return total;
}
