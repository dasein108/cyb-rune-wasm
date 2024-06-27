use anyhow::{anyhow, Result};
use rune::alloc::String as RuneString;
use rune::runtime::{Object, Value as RuneValue, VmResult, Vec as RuneVec};//, Vec as RuneVec
use serde_json::{ Map, Value as JsonValue, json};//json,
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::prelude::*;
use gloo_utils::format::JsValueSerdeExt;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[allow(clippy::module_name_repetitions)]
pub trait ToJson {
    fn to_json(&self) -> Result<JsonValue>;
}

fn convert_object(object: &Object) -> Result<JsonValue> {
    let object: Result<Map<_, _>> = object
        .iter()
        .map(|(key, value)| Ok((key.to_string(), value.to_json()?)))
        .collect();
    Ok(json!(object?))
}

fn convert_vec(vec: &RuneVec) -> Result<JsonValue> {
    let vec: Result<Vec<_>> = vec.iter().map(ToJson::to_json).collect();
    Ok(json!(vec?))
}

impl ToJson for RuneValue {
    fn to_json(&self) -> Result<JsonValue> {
        if self.into_unit() == Ok(()) {
            Ok(JsonValue::Null)
        } else if let Ok(b) = self.as_bool() {
            Ok(json!(b))
        } else if let Ok(f) = self.as_float() {
            Ok(json!(f))
        } else if let Ok(i) = self.as_integer() {
            Ok(json!(i))
        } else if let Ok(t) = self.borrow_tuple_ref() {
            if t.len() == 0 {
                Ok(JsonValue::Null)
            } else {
                Err(anyhow!("Unhandled tuple length {}", t.len()))
            }
        } else if let Ok(o) = self.borrow_object_ref() {
            convert_object(&o)
        } else if let Ok(s) = self.borrow_string_ref() {
            Ok(json!(s.to_string()))
        } else if let Ok(v) = self.borrow_vec_ref() {
            convert_vec(&v)
        } else {
            Err(anyhow!("Unhandled Value {:?}", self.type_info()?))
        }
    }
}



pub trait ToRune {
    fn to_rune(&self) -> Result<RuneValue>;
}

pub fn to_vec(array: &JsonValue) -> Result<Vec<RuneValue>> {
    match array {
        JsonValue::Array(a) => Ok(a
            .iter()
            .map(ToRune::to_rune)
            .collect::<Result<Vec<RuneValue>>>()?),
        _ => Err(anyhow!("Invalid JSON value. Expected an array.")),
    }
}

fn parse_array(array: &[JsonValue]) -> Result<RuneValue> {
    Ok(RuneValue::vec(
        array
            .iter()
            .map(ToRune::to_rune)
            .collect::<Result<Vec<RuneValue>>>()?
            .try_into()?,
    )
    .into_result()?)
}

fn parse_map(map: &Map<String, JsonValue>) -> Result<RuneValue> {
    let mut object = Object::new();
    for (key, value) in map {
        let _ = object.insert(
            RuneString::try_from(key.as_str()).map_err(anyhow::Error::from)?,
            value.to_rune()?,
        );
    }

    Ok(object.try_into()?)
}

impl ToRune for JsonValue {
    /// The line `let obj = serde_json::to_value::<HashMap<String,
    /// serde_json::Value>>(params).unwrap();` is converting a serde_json `Value` into a
    /// `HashMap<String, serde_json::Value>`.
    fn to_rune(&self) -> Result<RuneValue> {
        match self {
            Self::Array(a) => parse_array(a),
            Self::Bool(b) => Ok(RuneValue::try_from(*b)?),
            Self::Null => Ok(RuneValue::tuple(rune::alloc::Vec::new()).into_result()?),
            Self::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(RuneValue::try_from(i)?)
                } else if let Some(f) = n.as_f64() {
                    Ok(RuneValue::try_from(f)?)
                } else {
                    Err(anyhow!("{n} is neither an integer nor a float"))
                }
            }
            Self::Object(map) => parse_map(map),
            Self::String(s) => s.to_rune(),
        }
    }
}

impl ToRune for String {
    fn to_rune(&self) -> Result<RuneValue> {
        Ok(RuneValue::try_from(RuneString::try_from(self.as_str())?)?)
    }
}

pub async fn execute_promise<F>(f: F) -> VmResult<RuneValue>
where
    F: Fn() -> js_sys::Promise,
{
    let js_value = JsFuture::from(f()).await;
    match js_value {
        Ok(js_value) => {
            let js_value: JsonValue = JsValueSerdeExt::into_serde(&js_value).unwrap();
             VmResult::Ok(js_value.to_rune().unwrap())
        }
        Err(e) => VmResult::panic(format!("cyb-runtime error: {:?}", e)),
    }
}

pub fn rune_value_to_js(value: RuneValue) -> JsValue
{
    let json_value = value.to_json().unwrap();
    <JsValue as JsValueSerdeExt>::from_serde(&json_value).unwrap()
}
