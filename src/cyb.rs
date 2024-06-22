use rune::{ ContextError, Module};
use js_sys::{Function, Promise};
use rune::runtime::{VmResult, Ref, Value as VmValue, Vec as VmVec, EmptyStruct};
use wasm_bindgen::prelude::*;
use crate::json::{ToRune, ToJson};
use crate::json::execute_promise;
use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value as JsonValue;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "AsyncIterator<string>")]
    pub type JsAsyncIterator;

    #[wasm_bindgen(method, structural, js_class = "Object", js_name = next)]
    fn next(this: &JsAsyncIterator) -> Promise;
}

#[wasm_bindgen(raw_module = "../../src/services/scripting/wasmBindings.js")]
extern "C" {
    fn jsCyberSearch(query: &str)-> Promise;
    fn jsCyberLink(fromCid: &str, toCid: &str)-> Promise;
    fn jsGetPassportByNickname(nickname: &str)-> Promise;
    fn jsGetIpfsTextContent(cid: &str)-> Promise;
    fn jsAddContenToIpfs(content: &str)-> Promise;
    fn jsEvalScriptFromIpfs(cid: &str, func_name: &str, params: &JsValue)-> Promise;
    fn jsPromptToOpenAI(prompt: &str, api_key: &str, params: &JsValue, callback: Option<Function>)-> Promise;
    fn jsSearchByEmbedding(text: &str, count: usize)-> Promise;
    fn jsCyberLinksFrom(cid: &str)-> Promise;
    fn jsCyberLinksTo(cid: &str)-> Promise;
    fn jsExecuteScriptCallback(ref_id: &str, result: &JsValue)-> Promise;



    // fn js_execUserScript(nickname: &str, scriptName: &str)-> Promise;

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


pub async fn cyber_search(query: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberSearch(&query)).await
}

pub async fn get_text_from_ipfs(cid: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsGetIpfsTextContent(&cid)).await

}

pub async fn search_by_embedding(text: Ref<str>, count: usize) ->  VmResult<VmValue> {
    execute_promise(|| jsSearchByEmbedding(&text, count)).await
}

pub async fn cyber_link(from_cid: Ref<str>, to_cid: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberLink(&from_cid, &to_cid)).await
}

pub async fn get_passport_by_nickname(nickname: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsGetPassportByNickname(&nickname)).await
}



pub async fn eval_script_from_ipfs(cid: Ref<str>, func_name: Ref<str>, params: VmVec) ->  VmResult<VmValue> {
    // TODO:
    // let js_value: JsValue = params.into_inner();// object_to_js_value(params);
    let json_value = serde_json::to_value(params.into_inner()).unwrap();
    let js_value =  <JsValue as JsValueSerdeExt>::from_serde(&json_value).unwrap();

    execute_promise(|| jsEvalScriptFromIpfs(&cid, &func_name, &js_value)).await
}

pub async fn add_content_to_ipfs(content: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsAddContenToIpfs(&content)).await
}


// pub async fn open_ai_prompt(prompt: &str, api_key: &str, params: Object, callback: VmFunction) ->  VmResult<VmValue> {
//     let closure = Closure::wrap(Box::new(move |args: JsValue| {
//         // Your logic to convert JsValue to the appropriate arguments for the function
//         // TODO: implement
//         // callback.call(args);

//         // Convert the result back to JsValue
//     }) as Box<dyn FnMut(JsValue) -> JsValue>);

//     let js_function = closure.as_ref().unchecked_ref::<Function>().clone();
//     closure.forget(); // Prevent the closure from being deallocated

//     // let js_value =  <JsValue as JsValueSerdeExt>::from_serde(&json_value).unwrap();
//     let js_value: JsValue = object_to_js_value(params);

//     execute_promise(|| jsPromptToOpenAI(prompt, api_key, &js_value, closure)).await
// }

pub async fn get_cyberlinks_from_cid(cid: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberLinksFrom(&cid)).await
}

pub async fn get_cyberlinks_to_cid(cid: Ref<str>) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberLinksTo(&cid)).await
}

// pub async fn execute_callback(ref_id: Ref<str>, data: Object) ->  VmResult<VmValue> {
//     let js_value: JsValue = object_to_js_value(data);

//     execute_promise(|| jsExecuteScriptCallback(&ref_id, &js_value)).await
// }




/// The wasm 'cyb' module.
pub fn module(params: JsonValue, read_only: bool) -> Result<Module, ContextError> {
    console_log!("Creating module ");
    let mut module = Module::with_crate("cyb")?;

    let app:JsonValue = params.get("app").unwrap().clone();
    let ctx = app.to_rune().unwrap();

    let ref_id = params.get("refId").unwrap().clone().to_rune().unwrap();

    module.constant("context", ctx).build()?;
    module.constant("ref_id", ref_id).build()?;

    console_log!("Creating functions ");

    module.function("get_text_from_ipfs", get_text_from_ipfs).build()?;

    // module.function(["callback"], execute_callback).build()?;
    module.function("log", log).build()?;
    module.function("cyber_search", cyber_search).build()?;

    module.function("get_passport_by_nickname", get_passport_by_nickname).build()?;


    module.function(["eval_script_from_ipfs"], eval_script_from_ipfs).build()?;

    // module.function(["open_ai_prompt"], open_ai_prompt).build()?;

    module.function("search_by_embedding", search_by_embedding).build()?;
    module.function("get_cyberlinks_from_cid", get_cyberlinks_from_cid).build()?;
    module.function("get_cyberlinks_to_cid", get_cyberlinks_to_cid).build()?;

    // // non read-only functions
    // // if not read_only param, then trait as read-only
    if !read_only {
        module.function("cyber_link", cyber_link).build()?;
        module.function("add_content_to_ipfs", add_content_to_ipfs).build()?;
    }

    console_log!("Creating module OKKKK ");

    Ok(module)
}



