use rune::{ContextError, Module};
use js_sys::Promise;
use rune::runtime::{Object, VmResult, Vec, Value as VmValue};
use wasm_bindgen::prelude::*;
use serde_json::Value as SerdeValue;
use gloo_utils::format::JsValueSerdeExt;

use crate::helpers::{map_to_rune_value, execute_promise};

#[wasm_bindgen(raw_module = "../../src/services/scripting/wasmBindings.js")]
extern "C" {
    fn jsCyberSearch(query: &str)-> Promise;
    fn jsCyberLink(fromCid: &str, toCid: &str)-> Promise;
    fn jsGetPassportByNickname(nickname: &str)-> Promise;
    fn jsGetIpfsTextContent(cid: &str)-> Promise;
    fn jsAddContenToIpfs(content: &str)-> Promise;
    fn jsEvalScriptFromIpfs(cid: &str, func_name: &str, params: &JsValue)-> Promise;
    fn jsPromptToOpenAI(prompt: &str, api_key: &str)-> Promise;
    // fn js_cyberLinksFrom(cid: &str)-> Promise;
    // fn js_cyberLinksTo(cid: &str)-> Promise;



    // fn js_execUserScript(nickname: &str, scriptName: &str)-> Promise;

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub async fn cyber_search(query: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberSearch(query)).await
}

pub async fn cyber_link(from_cid: &str, to_cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsCyberLink(from_cid, to_cid)).await
}

pub async fn get_passport_by_nickname(nickname: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsGetPassportByNickname(nickname)).await
}

pub async fn get_text_from_ipfs(cid: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsGetIpfsTextContent(cid)).await
}

pub async fn eval_script_from_ipfs(cid: &str, func_name: &str, params: Vec) ->  VmResult<VmValue> {
    // let params: Object = rune::from_value(params.unwrap_or(Vec::new())).unwrap();
    let json_value: SerdeValue = serde_json::to_value(params.into_inner()).unwrap();

    // let json_value: SerdeValue = serde_json::to_value(params.unwrap_or(Vec::new()).into_inner()).unwrap();

    let js_value =  <JsValue as JsValueSerdeExt>::from_serde(&json_value).unwrap();

    execute_promise(|| jsEvalScriptFromIpfs(cid, func_name, &js_value)).await
}

pub async fn add_content_to_ipfs(content: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsAddContenToIpfs(content)).await
}

pub async fn open_ai_prompt(prompt: &str, api_key: &str) ->  VmResult<VmValue> {
    execute_promise(|| jsPromptToOpenAI(prompt, api_key)).await
}

// pub async fn get_cyberlinks_from_cid(cid: &str) ->  VmResult<VmValue> {
//     execute_promise(|| js_cyberLinksFrom(cid)).await
// }

// pub async fn get_cyberlinks_to_cid(cid: &str) ->  VmResult<VmValue> {
//     execute_promise(|| js_cyberLinksTo(cid)).await
// }



/// The wasm 'cyb' module.
pub fn module(params: SerdeValue, read_only: bool) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("cyb");

    module.constant(["context"], map_to_rune_value(&params["app"]))?;

    module.function(["log"], log)?;
    module.function(["cyber_search"], cyber_search)?;

    module.function(["get_passport_by_nickname"], get_passport_by_nickname)?;

    module.function(["get_text_from_ipfs"], get_text_from_ipfs)?;

    module.function(["eval_script_from_ipfs"], eval_script_from_ipfs)?;

    module.function(["open_ai_prompt"], open_ai_prompt)?;
    // module.function(["get_cyberlinks_from_cid"], get_cyberlinks_from_cid)?;
    // module.function(["get_cyberlinks_to_cid"], get_cyberlinks_to_cid)?;

    // non read-only functions
    // if not readOnly param, then trait as read-only
    if !read_only {
        module.function(["cyber_link"], cyber_link)?;
        module.function(["add_content_to_ipfs"], add_content_to_ipfs)?;
    }

    Ok(module)
}