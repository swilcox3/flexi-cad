use crate::prelude::*;
use crate::tests::*;
use super::*;
use crossbeam_channel::Receiver;

lazy_static!{
    static ref USER: RefID = RefID::new_v4();
}

fn test_setup(cb: impl Fn(&OperationManager, Receiver<UpdateMsg>)) {
    let (s, r) = crossbeam_channel::unbounded();
    let ops = OperationManager::new(USER.clone(), s);
    cb(&ops, r);
}

#[test]
fn test_dep_update() {
    test_setup( |ops, _| {
        let event = ops.begin_undo_event(&USER, String::from("dep_update")).unwrap();
        let mut obj_1 = TestObj::new("some stuff");
        let mut obj_2 = TestObj::new("other stuff");
        let id_1 = obj_1.get_id().clone();
        let id_2 = obj_2.get_id().clone();
        let ref_1 = Reference{id: id_1.clone(), index: ResultInd{index:0}};
        let ref_2 = Reference{id: id_2.clone(), index: ResultInd{index:0}};
        obj_1.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}, ref_2, &None);
        obj_2.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(2.0, 1.0, 0.0)}, ref_1, &None);
        ops.add_object(&event, Box::new(obj_1)).unwrap();
        ops.add_object(&event, Box::new(obj_2)).unwrap();
        ops.modify_obj(&event, &id_1, &mut |write_1: &mut DataObject| {
            let point_ref = write_1.query_mut::<Position>().unwrap();
            point_ref.move_obj(&Vector3f::new(3.0, 2.0, 1.0));
            Ok(())
        }).unwrap();
        ops.end_undo_event(event).unwrap();
        ops.get_obj(&id_1, |read_1| {
            let point_ref = read_1.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(3.0, 3.0, 3.0)}));
            Ok(())
        }).unwrap();
        ops.get_obj(&id_2, |read_2| {
            let point_ref = read_2.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(2.0, 1.0, 0.0)}));
            Ok(())
        }).unwrap();
        ops.update_deps(&id_1).unwrap();
        ops.get_obj(&id_2, |read_2| {
            let point_ref = read_2.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(3.0, 3.0, 3.0)}));
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_dep_undo() {
    test_setup(|ops, _| {
        let event = ops.begin_undo_event(&USER, String::from("dep_undo")).unwrap();
        let mut obj_1 = TestObj::new("some stuff");
        let mut obj_2 = TestObj::new("other stuff");
        let id_1 = obj_1.get_id().clone();
        let id_2 = obj_2.get_id().clone();
        let ref_1 = Reference{id: id_1.clone(), index: ResultInd{index:0}};
        let ref_2 = Reference{id: id_2.clone(), index: ResultInd{index:0}};
        obj_1.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}, ref_2, &None);
        obj_2.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(2.0, 1.0, 0.0)}, ref_1, &None);
        ops.add_object(&event, Box::new(obj_1)).unwrap();
        ops.add_object(&event, Box::new(obj_2)).unwrap();
        ops.end_undo_event(event).unwrap();
        let event = ops.begin_undo_event(&USER, String::from("dep_undo")).unwrap();
        ops.modify_obj(&event, &id_1, &mut |write_1: &mut DataObject| {
            let point_ref = write_1.query_mut::<Position>().unwrap();
            point_ref.move_obj(&Vector3f::new(3.0, 2.0, 1.0));
            Ok(())
        }).unwrap();
        ops.end_undo_event(event).unwrap();
        ops.update_deps(&id_1).unwrap();
        ops.undo_latest(&USER).unwrap();
        ops.get_obj(&id_1, |read_1| {
            let point_ref = read_1.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}));
            Ok(())
        }).unwrap();
        ops.get_obj(&id_2, |read_2| {
            let point_ref = read_2.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}));
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_dep_redo() {
    test_setup(|ops, _| {
        let event = ops.begin_undo_event(&USER, String::from("dep_undo")).unwrap();
        let mut obj_1 = TestObj::new("some stuff");
        let mut obj_2 = TestObj::new("other stuff");
        let id_1 = obj_1.get_id().clone();
        let id_2 = obj_2.get_id().clone();
        let ref_1 = Reference{id: id_1.clone(), index: ResultInd{index:0}};
        let ref_2 = Reference{id: id_2.clone(), index: ResultInd{index:0}};
        obj_1.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}, ref_2, &None);
        obj_2.set_ref(ReferInd{index:0}, &RefGeometry::Point{pt: Point3f::new(2.0, 1.0, 0.0)}, ref_1, &None);
        ops.add_object(&event, Box::new(obj_1)).unwrap();
        ops.add_object(&event, Box::new(obj_2)).unwrap();
        ops.end_undo_event(event).unwrap();
        ops.update_deps(&id_1).unwrap();
        ops.undo_latest(&USER).unwrap();
        assert_eq!(ops.get_obj(&id_1, &mut |_: &DataObject| {Ok(())}), Err(DBError::ObjNotFound));
        assert_eq!(ops.get_obj(&id_2, &mut |_: &DataObject| {Ok(())}), Err(DBError::ObjNotFound));
        ops.redo_latest(&USER).unwrap();
        ops.get_obj(&id_1, |read_1| {
            let point_ref = read_1.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}));
            Ok(())
        }).unwrap();
        ops.get_obj(&id_2, |read_1| {
            let point_ref = read_1.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(0.0, 1.0, 2.0)}));
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_copy() {
    test_setup(|ops, _| {
        let event = ops.begin_undo_event(&USER, String::from("copy")).unwrap();
        let mut obj_1 = Box::new(TestObj::new("some stuff"));
        let id_1 = obj_1.get_id().clone();
        obj_1.move_obj(&Vector3f::new(1.0, 2.0, 3.0));
        ops.add_object(&event, obj_1).unwrap();
        let copy_id = ops.copy_obj(&event, &id_1).unwrap();
        assert!(copy_id != id_1);
        ops.end_undo_event(event).unwrap();
        ops.get_obj(&copy_id, |copy| {
            let point_ref = copy.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(1.0, 2.0, 3.0)}));
            Ok(())
        }).unwrap();
    });
}