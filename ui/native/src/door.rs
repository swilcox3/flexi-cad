use data_model::*;
use neon::prelude::*;

declare_types! {
    pub class JsDoor for Door {
        init(mut cx) {
            let arg_0 = cx.argument::<JsValue>(0)?;
            let arg_1 = cx.argument::<JsValue>(1)?;
            let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
            let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
            let width = cx.argument::<JsNumber>(2)?.value();
            let height = cx.argument::<JsNumber>(3)?.value();
            Ok(Door::new(point_1, point_2, width, height))
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
                "first_pt" => {
                    let first = {
                        let guard = cx.lock();
                        let this_pt = this.borrow(&guard).dir.geom.pt_1;
                        this_pt.clone()
                    };
                    let first_obj = neon_serde::to_value(&mut cx, &first)?;
                    Ok(first_obj.upcast())
                }
                "second_pt" => {
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
                "first_pt" => {
                    let pt = neon_serde::from_value(&mut cx, arg)?;
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).dir.geom.pt_1 = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "second_pt" => {
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

        method getObj(mut cx) {
            let this = cx.this();
            let door = {
                let guard = cx.lock();
                let door = this.borrow(&guard).clone();
                door
            };
            let val = neon_serde::to_value(&mut cx, &door)?;
            Ok(val.upcast())
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