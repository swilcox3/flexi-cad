use data_model::*;
use neon::prelude::*;

declare_types! {
    pub class JsSlab for Slab {
        init(mut cx) {
            Ok(Slab::new())
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
            let slab = {
                let guard = cx.lock();
                let slab = this.borrow(&guard).clone();
                slab
            };
            let val = neon_serde::to_value(&mut cx, &json!({
                "type": "Slab",
                "obj": slab
            }))?;
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
