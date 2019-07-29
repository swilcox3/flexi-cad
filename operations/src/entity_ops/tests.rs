use super::*;
use crate::*;
use crate::tests::*;
use crossbeam_channel::{Receiver};

fn test_setup(desc: &str, callback: impl Fn(PathBuf, Receiver<UpdateMsg>)) {
    let file = PathBuf::from(desc);
    let (s, r) = crossbeam_channel::unbounded();
    app_state::init_file(file.clone(), s);
    callback(file, r);
}

//This makes sure that all the background updates have completed
fn empty_receiver(rcv: &Receiver<UpdateMsg>) {
    while let Ok(_) = rcv.recv_timeout(std::time::Duration::from_millis(500)) {
        //Do nothing
    }
}

#[test]
fn test_copy_objs() {
    test_setup("copy_objs", |file, rcv| {
        let mut first = Box::new(TestObj::new("first"));
        let id_1 = first.get_id().clone();
        first.move_obj(&Vector3f::new(1.0, 2.0, 3.0));
        let mut second = Box::new(TestObj::new("second"));
        let id_2 = second.get_id().clone();
        second.set_ref(&RefType::Point{which_pt: 0}, &RefResult::Point{pt:Point3f::new(1.0, 2.0, 3.0)}, Reference{id: id_1.clone(), ref_type: RefType::Point{which_pt: 0}});

        let event = app_state::begin_undo_event(&file, String::from("add objs")).unwrap();
        app_state::add_obj(&file, &event, first).unwrap();
        app_state::add_obj(&file, &event, second).unwrap();
        app_state::end_undo_event(&file, event).unwrap();
        let mut copy_set = HashSet::new();
        copy_set.insert(id_1);
        copy_set.insert(id_2);
        let event = app_state::begin_undo_event(&file, String::from("copy objs")).unwrap();
        let orig_to_dups = copy_objs(file.clone(), event.clone(), copy_set).unwrap();
        assert_eq!(orig_to_dups.len(), 2);
        move_obj(file.clone(), event.clone(), orig_to_dups.get(&id_1).unwrap().clone(), Vector3f::new(0.0, 0.0, 1.0)).unwrap();
        empty_receiver(&rcv);
        app_state::get_obj(&file, orig_to_dups.get(&id_1).unwrap(), |obj| {
            let point_ref = obj.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(&RefType::Point{which_pt:0}), Some(RefResult::Point{pt: Point3f::new(1.0, 2.0, 4.0)}));
            Ok(())
        }).unwrap();
        app_state::get_obj(&file, orig_to_dups.get(&id_2).unwrap(), |obj| {
            let point_ref = obj.query_ref::<ReferTo>().unwrap();
            assert_eq!(point_ref.get_result(&RefType::Point{which_pt:0}), Some(RefResult::Point{pt: Point3f::new(1.0, 2.0, 4.0)}));
            Ok(())
        }).unwrap();
    });
}

#[test]
fn test_snap_ref() {
    test_setup("snap_ref", |file, rcv| {
        let mut first = Box::new(TestObj::new("first"));
        let id_1 = first.get_id().clone();
        first.move_obj(&Vector3f::new(1.0, 2.0, 3.0));
        let second = Box::new(TestObj::new("second"));
        let id_2 = second.get_id().clone();

        let event = app_state::begin_undo_event(&file, String::from("add objs")).unwrap();
        app_state::add_obj(&file, &event, first).unwrap();
        app_state::add_obj(&file, &event, second).unwrap();
        app_state::end_undo_event(&file, event).unwrap();

        let event = app_state::begin_undo_event(&file, String::from("snap objs")).unwrap();
        let snapped = snap_ref_to_result(file.clone(), event.clone(), &id_2, 
            Reference{ 
                id: id_1, 
                ref_type: RefType::Point{ which_pt: 0 }
            },
            RefResult::Point{
                pt: Point3f::new(0.0, 0.0, 0.0)
            }).unwrap();
        assert_eq!(snapped, Some(RefResult::Point{pt: Point3f::new(1.0, 2.0, 3.0)}));
        empty_receiver(&rcv);
    });
}