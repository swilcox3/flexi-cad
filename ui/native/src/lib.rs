#[macro_use]
extern crate neon;
extern crate operations_kernel;
extern crate crossbeam_channel;
extern crate ccl;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate lazy_static;

use neon::prelude::*;
use std::path::PathBuf;
use operations_kernel::*;
use operations_kernel::RefID;
use std::str::FromStr;
use crossbeam_channel::Receiver;
use ccl::dhashmap::DHashMap;
use serde::{Serialize, Deserialize};

lazy_static!{
    static ref UPDATES: DHashMap<PathBuf, Receiver<UpdateMsg>> = DHashMap::default();
}

struct GetNextUpdate{
    file: PathBuf
}

impl Task for GetNextUpdate {
    type Output = Vec<UpdateMsg>;
    type Error = String;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        match UPDATES.get(&self.file) {
            Some(r) => {
                if r.len() > 0 {
                    let mut results = Vec::new();
                    for msg in r.try_iter() {
                        results.push(msg)
                    }
                    Ok(results)
                }
                else {
                    Ok(vec!(r.recv().unwrap()))
                }
            }
            None => Err(format!("File {:?} not found", self.file))
        }
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        let val = neon_serde::to_value(&mut cx, &result.unwrap())?;
        Ok(val)
    }
}

fn get_updates(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let f = cx.argument::<JsFunction>(1)?;
    GetNextUpdate{file: PathBuf::from(path)}.schedule(f);
    Ok(cx.undefined())
}

fn init_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let (s, r) = crossbeam_channel::unbounded();
    let pathbuf = PathBuf::from(path);
    operations_kernel::init_file(pathbuf.clone(), s);
    UPDATES.insert(pathbuf, r);
    Ok(cx.undefined())
}

fn open_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let (s, r) = crossbeam_channel::unbounded();
    let pathbuf = PathBuf::from(path);
    operations_kernel::open_file(pathbuf.clone(), s).unwrap();
    UPDATES.insert(pathbuf, r);
    Ok(cx.undefined())
}

fn save_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let pathbuf = PathBuf::from(path);
    operations_kernel::save_file(&pathbuf).unwrap();
    Ok(cx.undefined())
}

fn save_as_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let orig_path = PathBuf::from(cx.argument::<JsString>(0)?.value());
    let new_path = PathBuf::from(cx.argument::<JsString>(1)?.value());
    operations_kernel::save_as_file(&orig_path, new_path.clone()).unwrap();
    let (_, r) = UPDATES.remove(&orig_path).unwrap();
    UPDATES.insert(new_path, r);
    Ok(cx.undefined())
}

fn begin_undo_event(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = cx.argument::<JsString>(0)?.value();
    let desc = cx.argument::<JsString>(1)?.value();
    let event = operations_kernel::begin_undo_event(&PathBuf::from(path), desc).unwrap();
    Ok(cx.string(format!("{:?}", event)))
}

fn end_undo_event(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let id = cx.argument::<JsString>(1)?.value();
    operations_kernel::end_undo_event(&PathBuf::from(path), RefID::from_str(&id).unwrap()).unwrap();
    Ok(cx.undefined())
}

fn undo_latest(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    operations_kernel::undo_latest(&PathBuf::from(path)).unwrap();
    Ok(cx.undefined())
}

fn redo_latest(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    operations_kernel::redo_latest(&PathBuf::from(path)).unwrap();
    Ok(cx.undefined())
}

fn suspend_event(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let id = cx.argument::<JsString>(1)?.value();
    operations_kernel::suspend_event(&PathBuf::from(path), &RefID::from_str(&id).unwrap()).unwrap();
    Ok(cx.undefined())
}

fn resume_event(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let id = cx.argument::<JsString>(1)?.value();
    operations_kernel::resume_event(&PathBuf::from(path), &RefID::from_str(&id).unwrap()).unwrap();
    Ok(cx.undefined())
}

fn cancel_event(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let id = cx.argument::<JsString>(1)?.value();
    operations_kernel::cancel_event(&PathBuf::from(path), &RefID::from_str(&id).unwrap()).unwrap();
    Ok(cx.undefined())
}

fn take_undo_snapshot(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event_id = cx.argument::<JsString>(1)?.value();
    let obj_id = cx.argument::<JsString>(2)?.value();
    operations_kernel::take_undo_snapshot(&PathBuf::from(path), &RefID::from_str(&event_id).unwrap(), &RefID::from_str(&obj_id).unwrap()).unwrap();
    Ok(cx.undefined())
}

fn get_temp_wall(mut cx: FunctionContext) -> JsResult<JsValue> {
    let arg_0 = cx.argument::<JsValue>(0)?;
    let arg_1 = cx.argument::<JsValue>(1)?;
    let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
    let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
    let width = cx.argument::<JsNumber>(2)?.value();
    let height = cx.argument::<JsNumber>(3)?.value();
    let id = match cx.argument_opt(4) {
        Some(arg) => {
            if arg.is_a::<JsUndefined>() {
                RefID::new_v4()
            }
            else {
                let id_str = arg.downcast::<JsString>().or_throw(&mut cx)?.value();
                RefID::from_str(&id_str).unwrap()
            }
        }
        None => RefID::new_v4()
    };
    match operations_kernel::get_temp_wall(id, point_1, point_2, width, height) {
        Ok(data) => {
            let val = neon_serde::to_value(&mut cx, &data)?;
            Ok(val)
        }
        Err(e) => panic!("{:?}", e)
    }
}

fn create_wall(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arg_0 = cx.argument::<JsValue>(0)?;
    let arg_1 = cx.argument::<JsValue>(1)?;
    let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
    let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
    let width = cx.argument::<JsNumber>(2)?.value();
    let height = cx.argument::<JsNumber>(3)?.value();
    let path = cx.argument::<JsString>(4)?.value();
    let event = cx.argument::<JsString>(5)?.value();
    let id = match cx.argument_opt(6) {
        Some(arg) => {
            let id_str = arg.downcast::<JsString>().or_throw(&mut cx)?.value();
            RefID::from_str(&id_str).unwrap()
        }
        None => RefID::new_v4()
    };
    if let Err(e) = operations_kernel::create_wall(PathBuf::from(path), RefID::from_str(&event).unwrap(), id, point_1, point_2, width, height) {
        panic!("{:?}", e);
    }
    else {
        Ok(cx.undefined())
    }
}

fn join_at_point(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let id_2 = RefID::from_str(&cx.argument::<JsString>(3)?.value()).unwrap();
    let arg_4 = cx.argument::<JsValue>(4)?;
    let point = neon_serde::from_value(&mut cx, arg_4)?;
    operations_kernel::join_at_point(PathBuf::from(path), event, id_1, id_2, point).unwrap();
    Ok(cx.undefined())
}

fn move_object(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let arg_3 = cx.argument::<JsValue>(3)?;
    let delta = neon_serde::from_value(&mut cx, arg_3)?;
    operations_kernel::move_obj(PathBuf::from(path), event, id_1, delta).unwrap();
    Ok(cx.undefined())
}

fn delete_object(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let _ = operations_kernel::delete_obj(&PathBuf::from(path), &event, &id_1).unwrap();
    Ok(cx.undefined())
}

fn set_object_data(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let arg_3 = cx.argument::<JsString>(3)?.value();
    let data: serde_json::Value = serde_json::from_str(&arg_3).unwrap();
    operations_kernel::set_obj_data(PathBuf::from(path), event, id_1, data).unwrap();
    Ok(cx.undefined())
}

#[derive(Serialize, Deserialize)]
struct UpdateEntry(RefID, serde_json::Value);

fn set_objects_datas(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsArray>(2)?;
    let mut data = Vec::with_capacity(arg_2.len() as usize);
    for i in 0..arg_2.len() {
        let val = arg_2.get(&mut cx, i)?;
        data.push(neon_serde::from_value(&mut cx, val)?);
    }
    operations_kernel::set_objs_data(PathBuf::from(path), event, data).unwrap();
    Ok(cx.undefined())
}

fn get_hash_set_from_arg(mut cx: FunctionContext, input: Handle<JsArray>) -> HashSet<RefID> {
    let mut data = std::collections::HashSet::with_capacity(input.len() as usize);
    for i in 0..input.len() {
        let val = input.get(&mut cx, i).unwrap();
        let val_str:Handle<JsString> = val.downcast().unwrap();
        data.insert(RefID::from_str(&val_str.value()).unwrap());
    }
    data
}

fn move_objects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsArray>(2)?;
    let arg_3 = cx.argument::<JsValue>(3)?;
    let delta = neon_serde::from_value(&mut cx, arg_3)?;
    let ret = cx.undefined();
    let data = get_hash_set_from_arg(cx, arg_2);
    operations_kernel::move_objs(PathBuf::from(path), event, data, delta).unwrap();
    Ok(ret)
}

fn copy_objects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsArray>(2)?;
    let arg_3 = cx.argument::<JsValue>(3)?;
    let delta = neon_serde::from_value(&mut cx, arg_3)?;
    let ret = cx.undefined();
    let data = get_hash_set_from_arg(cx, arg_2);
    operations_kernel::copy_objs(PathBuf::from(path), event, data, delta).unwrap();
    Ok(ret)
}

register_module!(mut cx, {
    cx.export_function("get_updates", get_updates)?;
    cx.export_function("init_file", init_file)?;
    cx.export_function("open_file", open_file)?;
    cx.export_function("save_file", save_file)?;
    cx.export_function("save_as_file", save_as_file)?;
    cx.export_function("begin_undo_event", begin_undo_event)?;
    cx.export_function("end_undo_event", end_undo_event)?;
    cx.export_function("undo_latest", undo_latest)?;
    cx.export_function("redo_latest", redo_latest)?;
    cx.export_function("take_undo_snapshot", take_undo_snapshot)?;
    cx.export_function("suspend_event", suspend_event)?;
    cx.export_function("resume_event", resume_event)?;
    cx.export_function("cancel_event", cancel_event)?;
    cx.export_function("get_temp_wall", get_temp_wall)?;
    cx.export_function("create_wall", create_wall)?;
    cx.export_function("join_at_point", join_at_point)?;
    cx.export_function("move_object", move_object)?;
    cx.export_function("move_objects", move_objects)?;
    cx.export_function("delete_object", delete_object)?;
    cx.export_function("set_object_data", set_object_data)?;
    cx.export_function("set_objects_datas", set_objects_datas)?;
    cx.export_function("copy_objects", copy_objects)?;
    Ok(())
});
