use rune::runtime::{Shared, Object, VmResult, Value as VmValue, Vec as VmVec};
use serde_json::Value as SerdeValue;
use wasm_bindgen_futures::JsFuture;

pub fn map_to_rune_value(serde_value: &SerdeValue) -> VmValue {
    match serde_value {
        SerdeValue::Null => VmValue::Unit,
        SerdeValue::Bool(b) => VmValue::from(*b),
        SerdeValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                VmValue::from(i)
            } else if let Some(f) = n.as_f64() {
                VmValue::from(f)
            } else {
                VmValue::Unit // or handle this case differently
            }
        },
        SerdeValue::String(s) => VmValue::String(Shared::new(s.clone())),
        SerdeValue::Array(a) => {
            let rune_array: Vec<VmValue> = a.iter().map(|v| map_to_rune_value(v)).collect();
            VmValue::Vec(Shared::new(VmVec::from(rune_array)))
        },
        SerdeValue::Object(o) => {
            let std_object: Object = o.iter().map(|(k, v)| (k.clone(), map_to_rune_value(v))).collect::<Object>();
            VmValue::Object(Shared::new(std_object))
        },
    }
}

pub fn map_params_to_vec(serde_value: &SerdeValue) -> Vec<VmValue> {
    match serde_value {
        SerdeValue::Array(a) => {
            let rune_array: Vec<VmValue> = a.iter().map(|v| map_to_rune_value(v)).collect();
            Vec::from(rune_array)
        },
        _ => {
            Vec::from([map_to_rune_value(serde_value)])
        },
    }
}

pub async fn execute_promise<F>(f: F) -> VmResult<VmValue>
where
    F: Fn() -> js_sys::Promise,
{
    let js_value = JsFuture::from(f()).await;
    match  js_value {
        Ok(js_value) => {
            let v: SerdeValue = serde_wasm_bindgen::from_value(js_value).unwrap();
            VmResult::Ok(map_to_rune_value(&v))
        },
        Err(_) => VmResult::Ok(VmValue::Unit),
    }
}