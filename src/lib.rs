//! <img alt="rune logo" src="https://raw.githubusercontent.com/rune-rs/rune/main/assets/icon.png" />
//! <br>
//! <a href="https://github.com/rune-rs/rune"><img alt="github" src="https://img.shields.io/badge/github-rune--rs/rune-8da0cb?style=for-the-badge&logo=github" height="20"></a>
//! <a href="https://crates.io/crates/rune-wasm"><img alt="crates.io" src="https://img.shields.io/crates/v/rune-wasm.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>
//! <a href="https://docs.rs/rune-wasm"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rune--wasm-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20"></a>
//! <a href="https://discord.gg/v5AeNkT"><img alt="chat on discord" src="https://img.shields.io/discord/558644981137670144.svg?logo=discord&style=flat-square" height="20"></a>
//! <br>
//! Minimum support: Rust <b>1.65+</b>.
//! <br>
//! <br>
//! <a href="https://rune-rs.github.io"><b>Visit the site 🌐</b></a>
//! &mdash;
//! <a href="https://rune-rs.github.io/book/"><b>Read the book 📖</b></a>
//! <br>
//! <br>
//!
//! A WASM module for the Rune Language, an embeddable dynamic programming language for Rust.
//!
//! <br>
//!
//! ## Usage
//!
//! This is part of the [Rune Language].
//!
//! [Rune Language]: https://rune-rs.github.io

#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
#![allow(clippy::unused_unit)]

use std::fmt;
use std::sync::Arc;

use anyhow::Context as _;
use gloo_utils::format::JsValueSerdeExt;
use helpers::{map_to_rune_value,map_params_to_vec};
use rune::ast::Spanned;
use rune::compile::LinkerError;
use rune::diagnostics::{Diagnostic, FatalDiagnosticKind};
use rune::modules::capture_io::CaptureIo;
use rune::runtime::{budget, Value, VmResult};
use rune::{Context, ContextError, Options};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use serde_json::Value as SerdeValue;

mod cyb;
mod helpers;

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.
// macro_rules! console_log {
//     ($($t:tt)*) => (cyb::log(&format_args!($($t)*).to_string()))
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompilerParams {
    read_only: bool,
    func_name: String,
    func_params: SerdeValue,
    execute: bool,
    config: Config
}

#[derive(Default, Serialize)]
struct WasmPosition {
    line: u32,
    character: u32,
}

impl From<(usize, usize)> for WasmPosition {
    fn from((line, col): (usize, usize)) -> Self {
        Self {
            line: line as u32,
            character: col as u32,
        }
    }
}

#[derive(Deserialize)]
struct Config {
    /// Budget.
    #[serde(default)]
    budget: Option<usize>,
    /// Compiler options.
    #[serde(default)]
    options: Vec<String>,
    /// Include the `std::experiments` package.
    #[serde(default)]
    experimental: bool,
    /// Show instructions.
    #[serde(default)]
    instructions: bool,
    /// Suppress text warnings.
    #[serde(default)]
    suppress_text_warnings: bool,
}

#[derive(Serialize)]
enum WasmDiagnosticKind {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
}

#[derive(Serialize)]
struct WasmDiagnostic {
    kind: WasmDiagnosticKind,
    start: WasmPosition,
    end: WasmPosition,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmCompileResult {
    error: Option<String>,
    diagnostics_output: Option<String>,
    diagnostics: Vec<WasmDiagnostic>,
    result: Option<String>,
    output: Option<String>,
    instructions: Option<String>,
}

impl WasmCompileResult {
    /// Construct output from compile result.
    fn output(
        io: &CaptureIo,
        output: Value,
        diagnostics_output: Option<String>,
        diagnostics: Vec<WasmDiagnostic>,
        instructions: Option<String>,
    ) -> Self {
        Self {
            error: None,
            diagnostics_output,
            diagnostics,
            result: Some(format!("{:?}", output)),
            output: io.drain_utf8().ok(),
            instructions,
        }
    }

    /// Construct a result from an error.
    fn from_error<E>(
        io: &CaptureIo,
        error: E,
        diagnostics_output: Option<String>,
        diagnostics: Vec<WasmDiagnostic>,
        instructions: Option<String>,
    ) -> Self
    where
        E: fmt::Display,
    {
        Self {
            error: Some(error.to_string()),
            diagnostics_output,
            diagnostics,
            result: None,
            output: io.drain_utf8().ok(),
            instructions,
        }
    }
}

/// Setup a wasm-compatible context.
fn setup_context(experimental: bool, io: &CaptureIo, params: SerdeValue, read_only: bool) -> Result<Context, ContextError> {
    let mut context = Context::with_config(false)?;

    context.install(rune::modules::capture_io::module(io)?)?;
    context.install(cyb::module(params, read_only)?)?;
    context.install(rune_modules::http::module(true)?)?;
    context.install(rune_modules::json::module(true)?)?;
    context.install(rune_modules::toml::module(false)?)?;
    context.install(rune_modules::rand::module(false)?)?;

    if experimental {
        context.install(rune_modules::experiments::module(false)?)?;
    }

    Ok(context)
}

async fn inner_compile(
    input: String,
    io: &CaptureIo,
    scripts: String,
    params: JsValue,
    compiler_params: JsValue
) -> Result<WasmCompileResult, anyhow::Error> {
    // console_log!("compile: {:?}", JSON::stringify(&compiler_params));

    let compiler_params: CompilerParams = JsValueSerdeExt::into_serde(&compiler_params)?;
    let instructions = None;
    let config = compiler_params.config;
    let params: SerdeValue = JsValueSerdeExt::into_serde(&params)?;
    let budget = config.budget.unwrap_or(1_000_000);
    let mut sources = rune::Sources::new();

    sources.insert(rune::Source::new("entry", input));

    if scripts.len() > 0 {
        sources.insert(rune::Source::new("entry", scripts));
    }

    let context = setup_context(config.experimental, io, params, compiler_params.read_only)?;

    let mut options = Options::default();

    for option in &config.options {
        options.parse_option(option)?;
    }

    let mut d = rune::Diagnostics::new();
    let mut diagnostics = Vec::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut d)
        .with_options(&options)
        .build();
    for diagnostic in d.diagnostics() {
        match diagnostic {
            Diagnostic::Fatal(error) => {
                if let Some(source) = sources.get(error.source_id()) {
                    match error.kind() {
                        FatalDiagnosticKind::CompileError(error) => {
                            let span = error.span();

                            let start = WasmPosition::from(
                                source.pos_to_utf8_linecol(span.start.into_usize()),
                            );
                            let end = WasmPosition::from(
                                source.pos_to_utf8_linecol(span.end.into_usize()),
                            );

                            diagnostics.push(WasmDiagnostic {
                                kind: WasmDiagnosticKind::Error,
                                start,
                                end,
                                message: error.to_string(),
                            });
                        }
                        FatalDiagnosticKind::LinkError(error) => match error {
                            LinkerError::MissingFunction { hash, spans } => {
                                for (span, _) in spans {
                                    let start = WasmPosition::from(
                                        source.pos_to_utf8_linecol(span.start.into_usize()),
                                    );
                                    let end = WasmPosition::from(
                                        source.pos_to_utf8_linecol(span.end.into_usize()),
                                    );

                                    diagnostics.push(WasmDiagnostic {
                                        kind: WasmDiagnosticKind::Error,
                                        start,
                                        end,
                                        message: format!("missing function (hash: {})", hash),
                                    });
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            Diagnostic::Warning(warning) => {
                let span = warning.span();

                if let Some(source) = sources.get(warning.source_id()) {
                    let start =
                        WasmPosition::from(source.pos_to_utf8_linecol(span.start.into_usize()));
                    let end = WasmPosition::from(source.pos_to_utf8_linecol(span.end.into_usize()));

                    diagnostics.push(WasmDiagnostic {
                        kind: WasmDiagnosticKind::Warning,
                        start,
                        end,
                        message: warning.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    let mut writer = rune::termcolor::Buffer::no_color();

    if !config.suppress_text_warnings {
        d.emit(&mut writer, &sources)
            .context("emitting to buffer should never fail")?;
    }

    if !compiler_params.execute {
        return Ok(WasmCompileResult::output(
            io,
            Value::from(String::from("")),
            diagnostics_output(writer),
            diagnostics,
            instructions,
        ))
    }

    let unit = match result {
        Ok(unit) => Arc::new(unit),
        Err(error) => {
            return Ok(WasmCompileResult::from_error(
                io,
                error,
                diagnostics_output(writer),
                diagnostics,
                instructions,
            ));
        }
    };
    let instructions = if config.instructions {
        let mut out = rune::termcolor::Buffer::no_color();
        unit.emit_instructions(&mut out, &sources, false)
            .expect("dumping to string shouldn't fail");
        Some(diagnostics_output(out).context("converting instructions to UTF-8")?)
    } else {
        None
    };

    let mut vm = rune::Vm::new(Arc::new(context.runtime()), unit);

    // let mut params:  Vec<Value> = Vec::new();

    // if ref_id.len() > 0 {
    //     params.push(Value::from(ref_id));
    // }

    let params_vec = map_params_to_vec(&compiler_params.func_params);

    let mut execution = match vm.execute([&compiler_params.func_name], params_vec) {
        Ok(execution) => execution,
        Err(error) => {
            error
                .emit(&mut writer, &sources)
                .context("emitting to buffer should never fail")?;

            return Ok(WasmCompileResult::from_error(
                io,
                error,
                diagnostics_output(writer),
                diagnostics,
                instructions,
            ));
        }
    };

    let future = budget::with(budget, execution.async_complete());

    let output = match future.await {
        VmResult::Ok(output) => output,
        VmResult::Err(error) => {
            let vm = execution.vm();

            let (unit, ip) = match error.first_location() {
                Some(loc) => (&loc.unit, loc.ip),
                None => (vm.unit(), vm.ip()),
            };

            // NB: emit diagnostics if debug info is available.
            if let Some(debug) = unit.debug_info() {
                if let Some(inst) = debug.instruction_at(ip) {
                    if let Some(source) = sources.get(inst.source_id) {
                        let start = WasmPosition::from(
                            source.pos_to_utf8_linecol(inst.span.start.into_usize()),
                        );
                        let end = WasmPosition::from(
                            source.pos_to_utf8_linecol(inst.span.end.into_usize()),
                        );

                        diagnostics.push(WasmDiagnostic {
                            kind: WasmDiagnosticKind::Error,
                            start,
                            end,
                            message: error.to_string(),
                        });
                    }
                }
            }

            error
                .emit(&mut writer, &sources)
                .context("emitting to buffer should never fail")?;

            return Ok(WasmCompileResult::from_error(
                io,
                error,
                diagnostics_output(writer),
                diagnostics,
                instructions,
            ));
        }
    };

    Ok(WasmCompileResult::output(
        io,
        output,
        diagnostics_output(writer),
        diagnostics,
        instructions,
    ))
}

fn diagnostics_output(writer: rune::termcolor::Buffer) -> Option<String> {
    let mut string = String::from_utf8(writer.into_inner()).ok()?;
    let new_len = string.trim_end().len();
    string.truncate(new_len);
    Some(string)
}


#[wasm_bindgen]
pub async fn compile(input: String, scripts: String, params: JsValue, compiler_params: JsValue) -> JsValue {
    let io = CaptureIo::new();

    let result = match inner_compile(input, &io, scripts, params, compiler_params).await {
        Ok(result) => result,
        Err(error) => WasmCompileResult::from_error(&io, error, None, Vec::new(), None),
    };

    <JsValue as JsValueSerdeExt>::from_serde(&result).unwrap()
}


// pub async fn execute() -> JsValue {
//     let context = rune_modules::default_context()?;

//     let mut sources = rune::sources!(
//         entry => {
//             pub fn main(number) {
//                 number + 10
//             }
//         }
//     );

//     let mut diagnostics = Diagnostics::new();

//     let result = rune::prepare(&mut sources)
//         .with_context(&context)
//         .with_diagnostics(&mut diagnostics)
//         .build();

//     if !diagnostics.is_empty() {
//         let mut writer = rune::termcolor::Buffer::no_color();
//         diagnostics.emit(&mut writer, &sources)?;
//     }

//     let unit = result?;

//     let mut vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
//     let output = vm.execute(["main"], (33i64,))?.complete().into_result()?;
//     let output: i64 = rune::from_value(output)?;

//     println!("output: {}", output);
//     <JsValue as JsValueSerdeExt>::from_serde(&result).unwrap()
// }