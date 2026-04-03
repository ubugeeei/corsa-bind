use corsa_bind_rs::utils::{CorsaUtils, UnsafeTypeFlowInput};
use napi::Result;
use napi_derive::napi;

use crate::util::parse_json;

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_assignment(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(CorsaUtils::is_unsafe_assignment(&input))
}

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_return(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(CorsaUtils::is_unsafe_return(&input))
}
