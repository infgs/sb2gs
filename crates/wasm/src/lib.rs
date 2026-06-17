use std::io::Cursor;

use eyre::Result;
use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use zip::ZipArchive;

mod errors;

use crate::errors::result_to_js_result;

/// Decompiles `.sb3` bytes into asset file bytes keyed by goboscript project path.
#[wasm_bindgen(js_name = decompileAssetFiles)]
pub fn decompile_asset_files(input: &[u8]) -> JsValue {
    result_to_js_result((|| {
        let files = sb2gs_decompiler::decompile_asset_files(input)?;
        byte_files_to_js_value(files)
    })())
}

/// Decompiles `.sb3` bytes into goboscript text files keyed by project path.
#[wasm_bindgen(js_name = decompileCodeFiles)]
pub fn decompile_code_files(input: &[u8]) -> JsValue {
    result_to_js_result((|| {
        let mut archive = ZipArchive::new(Cursor::new(input))?;
        let assets = sb2gs_decompiler::decompile_assets(&mut archive)?;
        text_files_to_js_value(sb2gs_decompiler::decompile_code(&assets)?)
    })())
}

fn byte_files_to_js_value(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Result<JsValue> {
    let out = Object::new();
    for (path, bytes) in files {
        let array = Uint8Array::from(bytes.as_slice());
        Reflect::set(&out, &JsValue::from_str(&path), &array).map_err(js_error)?;
    }
    Ok(out.into())
}

fn text_files_to_js_value(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Result<JsValue> {
    let out = Object::new();
    for (path, bytes) in files {
        let text = String::from_utf8(bytes)?;
        Reflect::set(&out, &JsValue::from_str(&path), &JsValue::from_str(&text))
            .map_err(js_error)?;
    }
    Ok(out.into())
}

fn js_error(error: JsValue) -> eyre::Report {
    eyre::eyre!("{error:?}")
}
