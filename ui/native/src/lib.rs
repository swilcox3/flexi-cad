#[macro_use]
extern crate neon;
extern crate operations_kernel;
extern crate data_model;
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

mod wall;
mod door;
mod dimension;
mod math;

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

fn join_at_points(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let id_2 = RefID::from_str(&cx.argument::<JsString>(3)?.value()).unwrap();
    let arg_4 = cx.argument::<JsValue>(4)?;
    let point = neon_serde::from_value(&mut cx, arg_4)?;
    operations_kernel::join_objs(PathBuf::from(&path), &event, id_1, id_2, &RefType::Point, &RefType::Point, &point).unwrap();
    Ok(cx.undefined())
}

fn snap_to_line(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let id_2 = RefID::from_str(&cx.argument::<JsString>(3)?.value()).unwrap();
    let arg_4 = cx.argument::<JsValue>(4)?;
    let point = neon_serde::from_value(&mut cx, arg_4)?;
    operations_kernel::join_objs(PathBuf::from(&path), &event, id_1, id_2, &RefType::Rect, &RefType::Line, &point).unwrap();
    Ok(cx.undefined())
}

fn snap_to_point(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(2)?.value()).unwrap();
    let id_2 = RefID::from_str(&cx.argument::<JsString>(3)?.value()).unwrap();
    let arg_4 = cx.argument::<JsValue>(4)?;
    let point = neon_serde::from_value(&mut cx, arg_4)?;
    operations_kernel::snap_obj_to_other(PathBuf::from(&path), &event, id_1, &id_2, &RefType::Point, &point).unwrap();
    Ok(cx.undefined())
}

fn can_refer_to(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let path = cx.argument::<JsString>(0)?.value();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let result = operations_kernel::can_refer_to(&PathBuf::from(path), &id_1).unwrap();
    Ok(cx.boolean(result))
}

fn get_closest_point(mut cx: FunctionContext) -> JsResult<JsValue> {
    let path = cx.argument::<JsString>(0)?.value();
    let id_1 = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsValue>(2)?;
    let point = neon_serde::from_value(&mut cx, arg_2)?;
    let (_, ref_pt) = operations_kernel::get_closest_result(&PathBuf::from(&path), &id_1, &RefType::Point, &point).unwrap().unwrap();
    match ref_pt {
        RefGeometry::Point{pt} => Ok(neon_serde::to_value(&mut cx, &pt).unwrap()),
        _ => panic!("Not a point")
    }
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

fn get_object_data(mut cx: FunctionContext) -> JsResult<JsValue> {
    let path = cx.argument::<JsString>(0)?.value();
    let id = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let prop_name = cx.argument::<JsString>(2)?.value();
    let val = operations_kernel::get_obj_data(&PathBuf::from(path), &id, &prop_name).unwrap();
    Ok(neon_serde::to_value(&mut cx, &val)?)
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

fn move_objects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsArray>(2)?;
    let arg_3 = cx.argument::<JsValue>(3)?;
    let delta = neon_serde::from_value(&mut cx, arg_3)?;
    let ret = cx.undefined();
    let mut data = std::collections::HashSet::with_capacity(arg_2.len() as usize);
    for i in 0..arg_2.len() {
        let val = arg_2.get(&mut cx, i).unwrap();
        let val_str:Handle<JsString> = val.downcast().unwrap();
        data.insert(RefID::from_str(&val_str.value()).unwrap());
    }
    operations_kernel::move_objs(PathBuf::from(path), event, data, delta).unwrap();
    Ok(ret)
}

fn copy_objects(mut cx: FunctionContext) -> JsResult<JsArray> {
    let path = cx.argument::<JsString>(0)?.value();
    let event = RefID::from_str(&cx.argument::<JsString>(1)?.value()).unwrap();
    let arg_2 = cx.argument::<JsArray>(2)?;
    let mut data = std::collections::HashSet::with_capacity(arg_2.len() as usize);
    for i in 0..arg_2.len() {
        let val = arg_2.get(&mut cx, i).unwrap();
        let val_str:Handle<JsString> = val.downcast().unwrap();
        data.insert(RefID::from_str(&val_str.value()).unwrap());
    }
    let copy_ids = operations_kernel::copy_objs(PathBuf::from(path), event, data).unwrap();
    let js_array = JsArray::new(&mut cx, copy_ids.len() as u32);
    let mut i: u32 = 0;
    for pair in copy_ids {
        let js_value = neon_serde::to_value(&mut cx, &pair)?;
        js_array.set(&mut cx, i, js_value).unwrap();
        i = i + 1;
    }
    Ok(js_array)
}

fn debug_state(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let debug = operations_kernel::debug_state();
    println!("{}", debug);
    Ok(cx.undefined())
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
    cx.export_function("join_at_points", join_at_points)?;
    cx.export_function("snap_to_line", snap_to_line)?;
    cx.export_function("snap_to_point", snap_to_point)?;
    cx.export_function("move_object", move_object)?;
    cx.export_function("move_objects", move_objects)?;
    cx.export_function("delete_object", delete_object)?;
    cx.export_function("get_object_data", get_object_data)?;
    cx.export_function("set_object_data", set_object_data)?;
    cx.export_function("set_objects_datas", set_objects_datas)?;
    cx.export_function("copy_objects", copy_objects)?;
    cx.export_function("debug_state", debug_state)?;
    cx.export_function("can_refer_to", can_refer_to)?;
    cx.export_function("get_closest_point", get_closest_point)?;
    cx.export_class::<wall::JsWall>("Wall")?;
    cx.export_class::<door::JsDoor>("Door")?;
    cx.export_class::<dimension::JsDimension>("Dimension")?;
    cx.export_function("project_on_line", math::project_on_line)?;
    Ok(())
});
