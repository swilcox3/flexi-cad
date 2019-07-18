use crate::*;

pub fn get_temp_wall(id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<UpdateMsg, DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    wall.update()
}

pub fn create_wall(file: PathBuf, event: UndoEventID, id: RefID, point_1: Point3f, point_2: Point3f, width: WorldCoord, height: WorldCoord) -> Result<(), DBError> {
    let wall = Box::new(Wall::new(id, point_1, point_2, width, height));
    app_state::add_obj(&file, &event, wall)
}
