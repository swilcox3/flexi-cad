use super::*;
use crate::prelude::*;
use crate::tests::*;
use crate::{join_objs};
use crossbeam_channel::{Receiver};

fn test_setup(desc: &str, callback: impl Fn(PathBuf, UserID, Receiver<UpdateMsg>)) {
    let file = PathBuf::from(desc);
    let (s, r) = crossbeam_channel::unbounded();
    let user = UserID::new_v4();
    app_state::init_file(file.clone(), user, s);
    callback(file, user, r);
}

//This makes sure that all the background updates have completed
fn empty_receiver(rcv: &Receiver<UpdateMsg>) {
    while let Ok(_) = rcv.recv_timeout(std::time::Duration::from_millis(500)) {
        //Do nothing
    }
}

#[test]
fn test_copy_objs() {
    test_setup("copy_objs", |file, user, rcv| {
        let mut first = Box::new(TestObj::new("first"));
        let id_1 = first.get_id().clone();
        first.move_obj(&Vector3f::new(1.0, 2.0, 3.0));
        let mut second = Box::new(TestObj::new("second"));
        let id_2 = second.get_id().clone();
        second.set_ref(ReferInd{index:0}, 
           &RefGeometry::Point{pt:Point3f::new(1.0, 2.0, 3.0)}, 
            Reference{id: id_1.clone(), 
                    index: ResultInd{index:0}},
            &None
        );

        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("add objs")).unwrap();
        app_state::add_obj(&file, &event, first).unwrap();
        app_state::add_obj(&file, &event, second).unwrap();
        app_state::end_undo_event(&file, event).unwrap();
        let mut copy_set = HashSet::new();
        copy_set.insert(id_1);
        copy_set.insert(id_2);
        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("copy objs")).unwrap();
        let (_, orig_to_dups) = copy_objs(&file, &event, copy_set).unwrap();
        assert_eq!(orig_to_dups.len(), 2);
        let copy_id_1 = orig_to_dups.get(&id_1).unwrap();
        let copy_id_2 = orig_to_dups.get(&id_2).unwrap();
        crate::move_obj(file.clone(), &event, copy_id_1.clone(), &Vector3f::new(0.0, 0.0, 1.0)).unwrap();
        empty_receiver(&rcv);
        app_state::get_obj(&file, &copy_id_1, |obj| {
            let point_ref = obj.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(1.0, 2.0, 4.0)}));
            Ok(())
        }).unwrap();
        app_state::get_obj(&file, &copy_id_2, |obj| {
            let point_ref = obj.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(ResultInd{index:0}), Some(RefGeometry::Point{pt: Point3f::new(1.0, 2.0, 4.0)}));
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_join_walls() {
    test_setup("join walls", |file, user, rcv| {
        let first = Box::new(Wall::new(Point3f::new(1.0, 2.0, 3.0), Point3f::new(2.0, 2.0, 3.0), 1.0, 1.0));
        let id_1 = first.get_id().clone();
        let second = Box::new(Wall::new(Point3f::new(2.0, 3.0, 4.0,), Point3f::new(4.0, 5.0, 6.0), 1.0, 1.0));
        let id_2 = second.get_id().clone();
        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("add objs")).unwrap();
        app_state::add_obj(&file, &event, first).unwrap();
        app_state::add_obj(&file, &event, second).unwrap();
        app_state::end_undo_event(&file, event).unwrap();

        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("snap objs")).unwrap();
        join_objs(file.clone(), &event, id_1.clone(), id_2.clone(), &RefType::Point, &RefType::Point, &Point3f::new(2.0, 4.0, 3.0)).unwrap();
        empty_receiver(&rcv);
        crate::move_obj(file.clone(), &event, id_1.clone(), &Vector3f::new(0.0, 1.0, 0.0)).unwrap();
        app_state::end_undo_event(&file, event).unwrap();
        empty_receiver(&rcv);
        app_state::get_obj(&file, &id_1, |first| {
            let read = first.query_ref::<ReferTo>().unwrap();
            let pts = read.get_all_results();
            assert_eq!(pts[0], RefGeometry::Point{pt: Point3f::new(1.0, 3.0, 3.0)});
            assert_eq!(pts[1], RefGeometry::Point{pt: Point3f::new(2.0, 3.0, 3.0)});
            Ok(())
        }).unwrap();
        app_state::get_obj(&file, &id_2, |second| {
            let read = second.query_ref::<ReferTo>().unwrap();
            let pts = read.get_all_results();
            assert_eq!(pts[0], RefGeometry::Point{pt: Point3f::new(2.0, 3.0, 3.0)});
            assert_eq!(pts[1], RefGeometry::Point{pt: Point3f::new(4.0, 5.0, 6.0)});
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_join_door_and_wall() {
    test_setup("snap door to wall", |file, user, rcv| {
        let first = Box::new(Wall::new(Point3f::new(0.0, 0.0, 0.0), Point3f::new(1.0, 0.0, 0.0), 1.0, 1.0));
        let id_1 = first.get_id().clone();
        let second = Box::new(Door::new(Point3f::new(1.0, 2.0, 3.0), Point3f::new(1.0, 2.5, 3.0), 1.0, 1.0));
        let id_2 = second.get_id().clone();

        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("add objs")).unwrap();
        app_state::add_obj(&file, &event, first).unwrap();
        app_state::add_obj(&file, &event, second).unwrap();
        app_state::end_undo_event(&file, event).unwrap();

        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("snap objs")).unwrap();
        join_objs(file.clone(), &event, id_1.clone(), id_2.clone(), &RefType::Rect, &RefType::Line, &Point3f::new(0.25, 1.0, 0.0)).unwrap();
        app_state::end_undo_event(&file, event).unwrap();
        empty_receiver(&rcv);
        app_state::get_obj(&file, &id_2, |second| {
            let read = second.query_ref::<ReferTo>().unwrap();
            let pts = read.get_all_results();
            assert_eq!(pts[0], RefGeometry::Point{pt: Point3f::new(0.25, 0.0, 0.0)});
            assert_eq!(pts[1], RefGeometry::Point{pt: Point3f::new(0.75, 0.0, 0.0)});
            Ok(())
        }).unwrap();
        let event = UndoEventID::new_v4();
        app_state::begin_undo_event(&file, &user, event.clone(), String::from("move obj")).unwrap();
        crate::move_obj(file.clone(), &event, id_1.clone(), &Vector3f::new(0.0, 1.0, 0.0)).unwrap();
        app_state::end_undo_event(&file, event).unwrap();
        empty_receiver(&rcv);
        app_state::get_obj(&file, &id_2, |second| {
            let read = second.query_ref::<ReferTo>().unwrap();
            let pts = read.get_all_results();
            assert_eq!(pts[0], RefGeometry::Point{pt: Point3f::new(0.25, 1.0, 0.0)});
            assert_eq!(pts[1], RefGeometry::Point{pt: Point3f::new(0.75, 1.0, 0.0)});
            Ok(())
        }).unwrap();
    });
}