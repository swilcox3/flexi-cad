use neon::prelude::*;
use data_model::{Point3f, Vector3f};

declare_types! {
    pub class Point3d for Point3f {
        init(mut cx)  {
            let arg_0 = cx.argument::<JsNumber>(0)?.value();
            let arg_1 = cx.argument::<JsNumber>(1)?.value();
            let arg_2 = cx.argument::<JsNumber>(2)?.value();
            Ok(Point3f::new(arg_0, arg_1, arg_2))
        }

        method get(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let this = cx.this();

            match &attr[..] {
                "x" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).x;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                "y" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).y;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                "z" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).z;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }

        method set(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let arg = cx.argument::<JsNumber>(1)?.value();
            let mut this = cx.this();

            match &attr[..] {
                "x" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).x = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                "y" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).y = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                "z" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).z = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }
    }

    pub class Vector3d for Vector3f {
        init(mut cx)  {
            let arg_0 = cx.argument::<JsNumber>(0)?.value();
            let arg_1 = cx.argument::<JsNumber>(1)?.value();
            let arg_2 = cx.argument::<JsNumber>(2)?.value();
            Ok(Vector3f::new(arg_0, arg_1, arg_2))
        }

        method get(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let this = cx.this();

            match &attr[..] {
                "x" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).x;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                "y" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).y;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                "z" => {
                    let val = {
                        let guard = cx.lock();
                        let this_val = this.borrow(&guard).z;
                        this_val
                    };
                    Ok(cx.number(val).upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }

        method set(mut cx) {
            let attr: String = cx.argument::<JsString>(0)?.value();
            let arg = cx.argument::<JsNumber>(1)?.value();
            let mut this = cx.this();

            match &attr[..] {
                "x" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).x = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                "y" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).y = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                "z" => {
                    {
                        let guard = cx.lock();
                        this.borrow_mut(&guard).z = arg;
                    }
                    Ok(cx.undefined().upcast())
                }
                _ => cx.throw_type_error("property does not exist")
            }
        }
    }
}

pub fn project_on_line(mut cx: FunctionContext) -> JsResult<JsValue> {
    let arg_0 = cx.argument::<JsValue>(0)?;
    let first = neon_serde::from_value(&mut cx, arg_0)?;
    let arg_1 = cx.argument::<JsValue>(1)?;
    let second = neon_serde::from_value(&mut cx, arg_1)?;
    let arg_2 = cx.argument::<JsValue>(2)?;
    let project = neon_serde::from_value(&mut cx, arg_2)?;
    let pt = data_model::project_on_line(&first, &second, &project);
    let obj = neon_serde::to_value(&mut cx, &pt)?;
    Ok(obj)
}