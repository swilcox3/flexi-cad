use data_model::*;
use neon::prelude::*;

declare_types! {
    pub class JsDimension for Dimension {
        init(mut cx) {
            let arg_0 = cx.argument::<JsValue>(0)?;
            let arg_1 = cx.argument::<JsValue>(1)?;
            let point_1 = neon_serde::from_value(&mut cx, arg_0)?;
            let point_2 = neon_serde::from_value(&mut cx, arg_1)?;
            let offset = cx.argument::<JsNumber>(2)?.value();
            Ok(Dimension::new(point_1, point_2, offset))
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
                        let this_pt = this.borrow(&guard).first.geom.pt;
                        this_pt.clone()
                    };
                    let first_obj = neon_serde::to_value(&mut cx, &first)?;
                    Ok(first_obj.upcast())
                }
                "second_pt" => {
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
                "first_pt" => {
                    let pt = neon_serde::from_value(&mut cx, arg)?;
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).first.geom.pt = pt;
                    }
                    Ok(cx.undefined().upcast())
                }
                "second_pt" => {
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

        method getObj(mut cx) {
            let this = cx.this();
            let dim = {
                let guard = cx.lock();
                let dim = this.borrow(&guard).clone();
                dim
            };
            let val = neon_serde::to_value(&mut cx, &dim)?;
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