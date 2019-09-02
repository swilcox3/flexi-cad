use data_model::*;
use neon::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;

pub fn handle_conn(cx: &mut CallContext<JsDoor>, index: i32) -> Option<String> {
    if let Some(conn_arg) = cx.argument_opt(index) {
        if conn_arg.is_a::<JsString>() {
            let connection = conn_arg.downcast::<JsString>().unwrap().value();
            return Some(connection);
        }
    }
    return None;
}

declare_types! {
    pub class JsDoor for Door {
        init(mut cx) {
            let arg_0 = cx.argument::<JsValue>(0)?;
            let arg_1 = cx.argument::<JsValue>(1)?;
            let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
            let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
            let width = cx.argument::<JsNumber>(2)?.value();
            let height = cx.argument::<JsNumber>(3)?.value();
            let id = match cx.argument_opt(5) {
                Some(arg) => {
                    let id_str = arg.downcast::<JsString>().or_throw(&mut cx)?.value();
                    RefID::from_str(&id_str).unwrap()
                }
                None => RefID::new_v4()
            };
            Ok(Door::new(id, point_1, point_2, width, height))
        }

        method set_dir(mut cx) {
            let mut this = cx.this();
            let arg = cx.argument::<JsValue>(0)?;
            let dir = neon_serde::from_value(&mut cx, arg)?;
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).dir.geom.set_dir(&dir);
            }
            Ok(cx.undefined().upcast())
        }

        method get(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let this = cx.this();

            match &attr[..] {
                "id" => {
                    let id = {
                        let guard = cx.lock();
                        let this_id = this.borrow(&guard).get_id().clone();
                        this_id
                    };
                    Ok(cx.string(id.to_string()).upcast())
                }
                "first" => {
                    let first = {
                        let guard = cx.lock();
                        let this_pt = this.borrow(&guard).dir.geom.pt_1;
                        this_pt.clone()
                    };
                    let first_obj = neon_serde::to_value(&mut cx, &first)?;
                    Ok(first_obj.upcast())
                }
                "second" => {
                    let second = {
                        let guard = cx.lock();
                        let this_pt = this.borrow(&guard).dir.geom.pt_2;
                        this_pt.clone()
                    };
                    let second_obj = neon_serde::to_value(&mut cx, &second)?;
                    Ok(second_obj.upcast())
                }
                "width" => {
                    let width = {
                        let guard = cx.lock();
                        let this_width = this.borrow(&guard).width;
                        this_width
                    };
                    Ok(cx.number(width).upcast())
                }
                "height" => {
                    let height = {
                        let guard = cx.lock();
                        let this_height = this.borrow(&guard).height;
                        this_height
                    };
                    Ok(cx.number(height).upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }

        method set(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let arg = cx.argument::<JsValue>(1)?;
            let mut this = cx.this();

            match &attr[..] {
                "id" => {
                    let id = arg.downcast::<JsString>().or_throw(&mut cx)?.value();
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).set_id(RefID::from_str(&id).unwrap());
                    }
                    Ok(cx.undefined().upcast())
                }
                "first" => {
                    let pt = neon_serde::from_value(&mut cx, arg)?;
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).dir.geom.pt_1 = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "second" => {
                    let pt = neon_serde::from_value(&mut cx, arg)?;
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).dir.geom.pt_2 = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "width" => {
                    let width = arg.downcast::<JsNumber>().or_throw(&mut cx)?.value();
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).width = width;
                    };
                    Ok(cx.undefined().upcast())
                }
                "height" => {
                    let height = arg.downcast::<JsNumber>().or_throw(&mut cx)?.value();
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).height = height; 
                    };
                    Ok(cx.undefined().upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }


        method getTempRepr(mut cx) {
            let this = cx.this();
            let msg = {
                let guard = cx.lock();
                let this_msg = this.borrow(&guard).get_temp_repr().unwrap();
                this_msg
            };
            let val = neon_serde::to_value(&mut cx, &msg)?;
            Ok(val.upcast())
        }

        method addObject(mut cx) {
            let this = cx.this();
            let path = cx.argument::<JsString>(0)?.value();
            let event = cx.argument::<JsString>(1)?.value();
            {
                let guard = cx.lock();
                let door = this.borrow(&guard).clone();
                match handle_conn(&mut cx, 2) {
                    Some(connection) => crate::send_msg(connection, "add_door", vec![json!(path), json!(event), json!(door)]),
                    #[cfg(feature = "kernel")]
                    None => operations_kernel::add_obj(&PathBuf::from(path), &RefID::from_str(&event).unwrap(), Box::new(door)).unwrap(),
                    #[cfg(not(feature = "kernel"))]
                    None => panic("No connection"),
                }
            }
            Ok(cx.undefined().upcast())
        }

        method moveObj(mut cx) {
            let mut this = cx.this();
            let arg_0 = cx.argument::<JsValue>(0)?;
            let delta = neon_serde::from_value(&mut cx, arg_0)?;
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).move_obj(&delta);
            }
            Ok(cx.undefined().upcast())
        }
    }
}