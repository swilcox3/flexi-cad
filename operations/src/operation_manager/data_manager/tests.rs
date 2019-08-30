use super::*;
use crate::tests::*;

lazy_static!{
    //It'd techincally be better for test isolation to have one DB for each test, but this is actually a better test.  
    //I want the DB to be stable no matter how many threads are accessing it, no matter how they're messing with it
    static ref DB: DataManager = DataManager::new();
}

#[test]
fn test_add() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.get_obj(&id, |read| {
        let data = read.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("some data"), data);
        DB.end_undo_event(event).unwrap();
        Ok(())
    }).unwrap();
}

#[test]
fn test_delete() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.delete_obj(&event, &id).unwrap();
    DB.end_undo_event(event).unwrap();
    assert!(DB.get_obj(&id, |_|{Ok(())}).is_err());
}

#[test]
fn test_modify() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.get_mut_obj(&event, &id, |write| {
        write.query_mut::<Store>().unwrap().set_store_data(String::from("new data"));
        Ok(())
    }).unwrap();
    DB.get_obj(&id, |read| {
        let data = read.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("new data"), data);
        DB.end_undo_event(event).unwrap();
        Ok(())
    }).unwrap();
}

#[test]
fn test_add_undo() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.end_undo_event(event).unwrap();
    DB.undo_latest(&user).unwrap();
    assert!(DB.get_obj(&id, |_|{Ok(())}).is_err());
}

#[test]
fn test_delete_undo() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.end_undo_event(event).unwrap();
    let event2 = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event2.clone(), String::from("delete obj")).unwrap();
    DB.delete_obj(&event2, &id).unwrap();
    DB.end_undo_event(event2).unwrap();
    DB.undo_latest(&user).unwrap();
    DB.get_obj(&id, |read| {
        let data = read.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("some data"), data);
        Ok(())
    }).unwrap();
}

#[test]
fn test_modify_undo() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.end_undo_event(event).unwrap();
    let event_2 = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event_2.clone(), String::from("modify obj")).unwrap();
    DB.get_mut_obj(&event_2, &id, |write| {
        write.query_mut::<Store>().unwrap().set_store_data(String::from("new data"));
        Ok(())
    }).unwrap();
    DB.end_undo_event(event_2).unwrap();
    DB.undo_latest(&user).unwrap();
    DB.get_obj(&id, |read| {
        let data = read.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("some data"), data);
        Ok(())
    }).unwrap();
}

#[test]
fn test_modify_redo() {
    let user = RefID::new_v4();
    let event = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event.clone(), String::from("add obj")).unwrap();
    let obj = Box::new(TestObj::new("some data"));
    let id = obj.get_id().clone();
    DB.add_obj(&event, obj).unwrap();
    DB.end_undo_event(event).unwrap();
    let event_2 = UndoEventID::new_v4();
    DB.begin_undo_event(&user, event_2.clone(), String::from("modify obj")).unwrap();
    DB.get_mut_obj(&event_2, &id, |write| {
        write.query_mut::<Store>().unwrap().set_store_data(String::from("new data"));
        Ok(())
    }).unwrap();
    DB.end_undo_event(event_2).unwrap();
    DB.undo_latest(&user).unwrap();
    DB.redo_latest(&user).unwrap();
    DB.get_obj(&id, |read| {
        let data = read.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("new data"), data);
        Ok(())
    }).unwrap();
}

#[test]
fn test_contest() {
    let setup_user = RefID::new_v4();
    let setup_event = UndoEventID::new_v4();
    DB.begin_undo_event(&setup_user, setup_event.clone(), String::from("setup")).unwrap();
    let a = Box::new(TestObj::new("A"));
    let a_id = a.get_id().clone();
    DB.add_obj(&setup_event, a).unwrap();
    DB.end_undo_event(setup_event).unwrap();

    let a_clone = a_id.clone();
    let a_clone_2 = a_id.clone();
    let t_1 = std::thread::spawn(move || {
        DB.get_obj(&a_id, |_| {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            Ok(())
        }).unwrap();
    });
    let t_2 = std::thread::spawn(move || {
        let user_2 = RefID::new_v4();
        let event_2 = UndoEventID::new_v4();
        DB.begin_undo_event(&user_2, event_2.clone(), String::from("Op 2")).unwrap();
        DB.get_mut_obj(&event_2, &a_clone, |_| {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            Ok(())
        }).unwrap();
        DB.end_undo_event(event_2).unwrap();
    });
    let t_3 = std::thread::spawn(move || {
        let user_3 = RefID::new_v4();
        let event_3 = UndoEventID::new_v4();
        DB.begin_undo_event(&user_3, event_3.clone(), String::from("Op 3")).unwrap();
        DB.get_mut_obj(&event_3, &a_clone_2, |_| {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            Ok(())
        }).unwrap();
        DB.end_undo_event(event_3).unwrap();
    });
    t_1.join().unwrap();
    t_2.join().unwrap();
    t_3.join().unwrap();
}