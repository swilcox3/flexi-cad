use crate::operation_manager::OperationManager;
use crate::prelude::*;
use ccl::dhashmap::DHashMap;
use crossbeam_channel::Sender;
use std::path::PathBuf;

lazy_static! {
    static ref APP_STATE: AppState = AppState::new();
}

pub struct AppState {
    files: DHashMap<PathBuf, OperationManager>,
}

impl AppState {
    fn new() -> AppState {
        AppState { files: DHashMap::default() }
    }
}

pub fn init_file(file: PathBuf, user: UserID, updates: Sender<UpdateMsg>) {
    rayon::ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();
    if let Some(ops) = APP_STATE.files.get(&file) {
        ops.updates.insert(user, updates);
    } else {
        APP_STATE.files.insert(file, OperationManager::new(user, updates));
    }
}

pub fn open_file(file: PathBuf, user: UserID, updates: Sender<UpdateMsg>) -> Result<(), DBError> {
    if let Some(ops) = APP_STATE.files.get(&file) {
        ops.updates.insert(user, updates);
    } else {
        let ops = OperationManager::open(&file, user, updates)?;
        APP_STATE.files.insert(file.clone(), ops);
        rayon::spawn(move || {
            if let Some(ops) = APP_STATE.files.get(&file) {
                ops.update_all(Some(&user)).unwrap();
            }
        });
    }
    Ok(())
}

pub fn save_file(file: &PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.save(file),
        None => Err(DBError::FileNotFound),
    }
}

pub fn save_as_file(orig_file: &PathBuf, file_new: PathBuf) -> Result<(), DBError> {
    match APP_STATE.files.remove(orig_file) {
        Some((_, ops)) => match ops.save(&file_new) {
            Ok(()) => {
                APP_STATE.files.insert(file_new, ops);
                Ok(())
            }
            Err(e) => {
                APP_STATE.files.insert(orig_file.clone(), ops);
                Err(e)
            }
        },
        None => Err(DBError::FileNotFound),
    }
}

pub fn send_read_result(file: &PathBuf, query_id: QueryID, user: &UserID, data: serde_json::Value) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => {
            info!("Sending data {:?} to user {:?}", data, user);
            ops.send(
                UpdateMsg::Read {
                    query_id,
                    user: user.clone(),
                    data,
                },
                Some(user),
            )
        }
        None => Err(DBError::FileNotFound),
    }
}

pub fn begin_undo_event(file: &PathBuf, user_id: &UserID, event_id: UndoEventID, desc: String) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.begin_undo_event(user_id, event_id, desc),
        None => Err(DBError::FileNotFound),
    }
}

pub fn end_undo_event(file: &PathBuf, event: UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.end_undo_event(event),
        None => Err(DBError::FileNotFound),
    }
}

pub fn undo_latest(file: &PathBuf, user: &UserID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.undo_latest(user),
        None => Err(DBError::FileNotFound),
    }
}

pub fn redo_latest(file: &PathBuf, user: &UserID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.redo_latest(user),
        None => Err(DBError::FileNotFound),
    }
}

pub fn suspend_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.suspend_event(event),
        None => Err(DBError::FileNotFound),
    }
}

pub fn resume_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.resume_event(event),
        None => Err(DBError::FileNotFound),
    }
}

pub fn cancel_event(file: &PathBuf, event: &UndoEventID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.cancel_event(event),
        None => Err(DBError::FileNotFound),
    }
}

pub fn take_undo_snapshot(file: &PathBuf, event: &UndoEventID, key: &RefID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.take_undo_snapshot(event, key),
        None => Err(DBError::FileNotFound),
    }
}

pub fn add_obj(file: &PathBuf, event: &UndoEventID, obj: DataObject) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.add_object(&event, obj),
        None => Err(DBError::FileNotFound),
    }
}

pub fn get_obj(file: &PathBuf, id: &RefID, mut callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.get_obj(id, &mut callback),
        None => Err(DBError::FileNotFound),
    }
}

pub fn modify_obj(
    file: &PathBuf,
    event: &UndoEventID,
    id: &RefID,
    mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>,
) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.modify_obj(event, id, &mut callback),
        None => Err(DBError::FileNotFound),
    }
}

pub fn delete_obj(file: &PathBuf, event: &UndoEventID, id: &RefID) -> Result<DataObject, DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.delete_obj(event, id),
        None => Err(DBError::FileNotFound),
    }
}

pub fn add_ref(
    file: &PathBuf,
    event: &UndoEventID,
    obj: &RefID,
    result: &RefGeometry,
    refer: GeometryId,
    snap_pt: &Option<Point3f>,
) -> Result<(), DBError> {
    let mut index = 0;
    modify_obj(&file, &event, &obj, |owner| match owner.query_mut::<dyn UpdateFromRefs>() {
        Some(joinable) => {
            index = joinable.get_num_refs();
            if joinable.add_ref(result, refer.clone(), snap_pt) {
                Ok(())
            } else {
                Err(error_other("Reference not added"))
            }
        }
        None => Err(DBError::ObjLacksTrait),
    })?;
    add_deps(&file, obj)
}

pub fn set_ref(
    file: &PathBuf,
    event: &UndoEventID,
    obj: &RefID,
    index: ReferInd,
    result: &RefGeometry,
    refer: GeometryId,
    snap_pt: &Option<Point3f>,
) -> Result<(), DBError> {
    modify_obj(&file, &event, &obj, |owner| match owner.query_mut::<dyn UpdateFromRefs>() {
        Some(joinable) => {
            joinable.set_ref(index, result, refer.clone(), snap_pt);
            Ok(())
        }
        None => Err(DBError::ObjLacksTrait),
    })?;
    add_deps(&file, obj)
}

pub fn update_deps(file: PathBuf, id: RefID) {
    rayon::spawn(move || {
        if let Some(ops) = APP_STATE.files.get(&file) {
            if let Err(e) = ops.update_deps(&id) {
                println!("Error in background: {:?}", e);
            }
        }
    });
}

pub fn update_all_deps(file: PathBuf, ids: Vec<RefID>) {
    rayon::spawn(move || {
        if let Some(ops) = APP_STATE.files.get(&file) {
            if let Err(e) = ops.update_all_deps(ids) {
                println!("Error in background: {:?}", e);
            }
        }
    });
}

pub fn add_deps(file: &PathBuf, id: &RefID) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.add_deps(id),
        None => Err(DBError::FileNotFound),
    }
}

pub fn remove_dep(file: &PathBuf, publisher: &GeometryId, subscriber: &GeometryId) -> Result<(), DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => Ok(ops.remove_dep(publisher, subscriber)),
        None => Err(DBError::FileNotFound),
    }
}

pub fn copy_obj(file: &PathBuf, event: &UndoEventID, id: &RefID) -> Result<RefID, DBError> {
    match APP_STATE.files.get(file) {
        Some(ops) => ops.copy_obj(event, id),
        None => Err(DBError::FileNotFound),
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
