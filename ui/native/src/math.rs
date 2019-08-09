use neon::prelude::*;

pub fn project_on_line(mut cx: FunctionContext) -> JsResult<JsValue> {
    let arg_0 = cx.argument::<JsValue>(0)?;
    let first = neon_serde::from_value(&mut cx, arg_0)?;
    let arg_1 = cx.argument::<JsValue>(1)?;
    let second = neon_serde::from_value(&mut cx, arg_1)?;
    let arg_2 = cx.argument::<JsValue>(2)?;
    let project = neon_serde::from_value(&mut cx, arg_2)?;
    let (pt, _) = data_model::project_on_line(&first, &second, &project);
    let obj = neon_serde::to_value(&mut cx, &pt)?;
    Ok(obj)
}