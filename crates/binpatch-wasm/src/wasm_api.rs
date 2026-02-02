use binpatch_core::{PatchOptions, Profile};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct VerifyOutput {
    valid_offsets: Vec<usize>,
}

#[wasm_bindgen]
pub fn verify(profile_toml: &str, firmware: js_sys::Uint8Array) -> Result<JsValue, JsValue> {
    let profile = Profile::parse_toml(profile_toml).map_err(to_js_error)?;
    let firmware_vec = firmware.to_vec();
    let report = profile.verify(&firmware_vec).map_err(to_js_error)?;

    let out = VerifyOutput {
        valid_offsets: report.valid.into_iter().map(|c| c.offset).collect(),
    };
    serde_wasm_bindgen::to_value(&out).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn patch(
    profile_toml: &str,
    firmware: js_sys::Uint8Array,
    ssid: &str,
    psk: &str,
    select_index: Option<usize>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let profile = Profile::parse_toml(profile_toml).map_err(to_js_error)?;
    let firmware_vec = firmware.to_vec();

    let patched = profile
        .patch(
            &firmware_vec,
            PatchOptions {
                ssid: ssid.to_string(),
                psk: psk.to_string(),
                select_index,
            },
        )
        .map_err(to_js_error)?;

    Ok(js_sys::Uint8Array::from(patched.as_slice()))
}

fn to_js_error<E: core::fmt::Display>(error: E) -> JsValue {
    JsValue::from_str(&error.to_string())
}
