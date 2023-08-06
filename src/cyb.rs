use rune::{ContextError, Module};
use js_sys::Promise;
use rune::runtime::{VmResult, Value as VmValue};
use wasm_bindgen::prelude::*;
use serde_json::Value as SerdeValue;
use gloo_utils::format::JsValueSerdeExt;

use rune::runtime::Object;

use crate::helpers::{map_to_rune_value, execute_promise};

#[wasm_bindgen(raw_module = "../../src/wasm_bindings.js")]
extern "C" {
    fn js_getPassportByNickname(nickname: &str)-> Promise;
    fn js_promptToOpenAI(prompt: &str, api_key: &str)-> Promise;
    fn js_getIpfsTextContent(cid: &str)-> Promise;
    fn js_cyberLinksFrom(cid: &str)-> Promise;
    fn js_cyberLinksTo(cid: &str)-> Promise;

    fn js_addContenToIpfs(content: &str)-> Promise;
    fn js_cyberSearch(query: &str)-> Promise;
    fn js_cyberLink(fromCid: &str, toCid: &str)-> Promise;

    fn js_execUserScript(nickname: &str, scriptName: &str)-> Promise;

    fn js_evalScriptFromIpfs(cid: &str, func_name: &str, params: &JsValue)-> Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub async fn get_passport_by_nickname(nickname: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_getPassportByNickname(nickname)).await
}

pub async fn open_ai_prompt(prompt: &str, api_key: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_promptToOpenAI(prompt, api_key)).await
}

pub async fn get_text_from_ipfs(cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_getIpfsTextContent(cid)).await
}

pub async fn add_text_to_ipfs(content: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_addContenToIpfs(content)).await
}

pub async fn get_cyberlinks_from_cid(cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_cyberLinksFrom(cid)).await
}

pub async fn get_cyberlinks_to_cid(cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_cyberLinksTo(cid)).await
}

pub async fn cyber_search(query: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_cyberSearch(query)).await
}

pub async fn cyber_link(from_cid: &str, to_cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| js_cyberLink(from_cid, to_cid)).await
}

pub async fn eval_script_from_ipfs(cid: &str, func_name: &str, params: VmValue) ->  VmResult<VmValue> {
    let params: Object = rune::from_value(params).unwrap();

    let json_value: SerdeValue = serde_json::to_value(params.into_inner()).unwrap();

    let js_value =  <JsValue as JsValueSerdeExt>::from_serde(&json_value).unwrap();

    execute_promise(|| js_evalScriptFromIpfs(cid, func_name, &js_value)).await
}

/// The wasm 'cyb' module.
pub fn module(params: SerdeValue, read_only: bool) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("cyb");

    module.constant(["context"], map_to_rune_value(&params["app"]))?;

    module.function(["log"], log)?;

    module.function(["get_passport_by_nickname"], get_passport_by_nickname)?;
    module.function(["open_ai_prompt"], open_ai_prompt)?;
    module.function(["get_cyberlinks_from_cid"], get_cyberlinks_from_cid)?;
    module.function(["get_cyberlinks_to_cid"], get_cyberlinks_to_cid)?;

    module.function(["get_text_from_ipfs"], get_text_from_ipfs)?;

    module.function(["cyber_search"], cyber_search)?;


    module.function(["eval_script_from_ipfs"], eval_script_from_ipfs)?;

    // non read-only functions
    // if not readOnly param, then trait as read-only
    if !read_only {
        module.function(["cyber_link"], cyber_link)?;
        module.function(["add_text_to_ipfs"], add_text_to_ipfs)?;
    }

    Ok(module)
}