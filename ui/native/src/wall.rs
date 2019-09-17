use data_model::*;
use neon::prelude::*;

declare_types! {
    pub class JsWall for Wall {
        init(mut cx) {
            let arg_0 = cx.argument::<JsValue>(0)?;
            let arg_1 = cx.argument::<JsValue>(1)?;
            let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
            let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
            let width = cx.argument::<JsNumber>(2)?.value();
            let height = cx.argument::<JsNumber>(3)?.value();
            Ok(Wall::new(point_1, point_2, width, height))
        }

        method id(mut cx) {
            let this = cx.this();
            let id = {
                let guard = cx.lock();
                let this_id = this.borrow(&guard).get_id().clone();
                this_id
            };
            Ok(cx.string(id.to_string()).upcast())
        }

        method first_pt(mut cx) {
            let this = cx.this();
            let first = {
                let guard = cx.lock();
                let this_pt = this.borrow(&guard).first_pt.geom.pt.clone();
                this_pt.clone()
            };
            let first_obj = neon_serde::to_value(&mut cx, &first)?;
            Ok(first_obj.upcast())
        }

        method set_first_pt(mut cx) {
            let arg = cx.argument::<JsValue>(0)?;
            let mut this = cx.this();
            let pt = neon_serde::from_value(&mut cx, arg)?;
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).first_pt.geom.pt = pt;
            }
            Ok(cx.undefined().upcast())
        }

        method second_pt(mut cx) {
            let this = cx.this();
            let second = {
                let guard = cx.lock();
                let this_pt = this.borrow(&guard).second_pt.geom.pt.clone();
                this_pt.clone()
            };
            let second_obj = neon_serde::to_value(&mut cx, &second)?;
            Ok(second_obj.upcast())
        }

        method set_second_pt(mut cx) {
            let arg = cx.argument::<JsValue>(0)?;
            let mut this = cx.this();
            let pt = neon_serde::from_value(&mut cx, arg)?;
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).second_pt.geom.pt = pt;
            }
            Ok(cx.undefined().upcast())
        }

        method width(mut cx) {
            let this = cx.this();
            let width = {
                let guard = cx.lock();
                let this_width = this.borrow(&guard).width;
                this_width
            };
            Ok(cx.number(width).upcast())
        }

        method set_width(mut cx) {
            let arg = cx.argument::<JsValue>(0)?;
            let mut this = cx.this();
            let width = arg.downcast::<JsNumber>().or_throw(&mut cx)?.value();
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).width = width;
            }
            Ok(cx.undefined().upcast())
        }

        method height(mut cx) {
            let this = cx.this();
            let height = {
                let guard = cx.lock();
                let this_height = this.borrow(&guard).height;
                this_height
            };
            Ok(cx.number(height).upcast())
        }

        method set_height(mut cx) {
            let arg = cx.argument::<JsValue>(0)?;
            let mut this = cx.this();
            let height = arg.downcast::<JsNumber>().or_throw(&mut cx)?.value();
            {
                let guard = cx.lock();
                this.borrow_mut(&guard).height = height; 
            }
            Ok(cx.undefined().upcast())
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
            let wall = {
                let guard = cx.lock();
                let wall = this.borrow(&guard).clone();
                wall
            };
            let val = neon_serde::to_value(&mut cx, &wall)?;
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