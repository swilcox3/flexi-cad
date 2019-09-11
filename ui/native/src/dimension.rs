use data_model::*;
use neon::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;


pub fn handle_conn(cx: &mut CallContext<JsDimension>, index: i32) -> Option<String> {
    if let Some(conn_arg) = cx.argument_opt(index) {
        if conn_arg.is_a::<JsString>() {
            let connection = conn_arg.downcast::<JsString>().unwrap().value();
            return Some(connection);
        }
    }
    return None;
}

declare_types! {
    pub class JsDimension for Dimension {
        init(mut cx) {
            let arg_0 = cx.argument::<JsValue>(0)?;
            let arg_1 = cx.argument::<JsValue>(1)?;
            let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
            let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
            let offset = cx.argument::<JsNumber>(2)?.value();
            let id = match cx.argument_opt(3) {
                Some(arg) => {
                    let id_str = arg.downcast::<JsString>().or_throw(&mut cx)?.value();
                    RefID::from_str(&id_str).unwrap()
                }
                None => RefID::new_v4()
            };
            Ok(Dimension::new(id, point_1, point_2, offset))
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
                        let this_pt = this.borrow(&guard).first.geom.pt;
                        this_pt.clone()
                    };
                    let first_obj = neon_serde::to_value(&mut cx, &first)?;
                    Ok(first_obj.upcast())
                }
                "second" => {
                    let second = {
                        let guard = cx.lock();
                        let this_pt = this.borrow(&guard).second.geom.pt;
                        this_pt.clone()
                    };
                    let second_obj = neon_serde::to_value(&mut cx, &second)?;
                    Ok(second_obj.upcast())
                }
                "offset" => {
                    let offset = {
                        let guard = cx.lock();
                        let this_offset = this.borrow(&guard).offset;
                        this_offset
                    };
                    Ok(cx.number(offset).upcast())
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
                        this.borrow_mut(&guard).first.geom.pt = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "second" => {
                    let pt = neon_serde::from_value(&mut cx, arg)?;
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).second.geom.pt = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "offset" => {
                    let offset = arg.downcast::<JsNumber>().or_throw(&mut cx)?.value();
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).offset = offset;
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
                let dim = this.borrow(&guard).clone();
                match handle_conn(&mut cx, 2) {
                    Some(connection) => crate::send_msg(connection, "add_dimension", vec![json!(path), json!(event), json!(dim)]),
                    #[cfg(feature = "kernel")]
                    None => operations_kernel::add_obj(&PathBuf::from(path), &RefID::from_str(&event).unwrap(), Box::new(dim)).unwrap(),
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