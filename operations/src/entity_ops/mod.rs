mod wall_ops;

pub use wall_ops::*;
use crate::*;

pub fn move_obj(file: PathBuf, event: UndoEventID, id: RefID, delta: Vector3f) -> Result<(), DBError> {
    app_state::modify_obj(&file, &event, &id, |obj| {
        match obj.query_mut::<Position>() {
            Some(movable) => {
                movable.move_obj(&delta);
                Ok(())
            }
            None => Err(DBError::ObjLacksTrait)
        }
    })?;
    app_state::update_deps(file, id);
    Ok(())
}

pub fn set_obj_data(file: PathBuf, event: UndoEventID, id: RefID, data: serde_json::Value) -> Result<(), DBError> {
    app_state::modify_obj(&file, &event, &id, |obj| {
        obj.set_data(&data)
    })?;
    app_state::update_deps(file, id);
    Ok(())
}