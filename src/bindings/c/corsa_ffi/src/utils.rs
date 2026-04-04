use corsa_core::utils;

use crate::types::{
    CorsaStrRef, CorsaString, CorsaStringList, collect_strings, into_c_string, into_c_string_list,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_utils_classify_type_text(text: CorsaStrRef) -> CorsaString {
    let text = unsafe { text.as_str() };
    into_c_string(utils::classify_type_text(text).as_str())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_utils_split_top_level_type_text(
    text: CorsaStrRef,
    delimiter: u32,
) -> CorsaStringList {
    let Some(text) = (unsafe { text.as_str() }) else {
        return CorsaStringList::default();
    };
    let Some(delimiter) = char::from_u32(delimiter) else {
        return CorsaStringList::default();
    };
    into_c_string_list(utils::split_top_level_type_text(text, delimiter))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_utils_split_type_text(text: CorsaStrRef) -> CorsaStringList {
    let Some(text) = (unsafe { text.as_str() }) else {
        return CorsaStringList::default();
    };
    into_c_string_list(utils::split_type_text(text))
}

macro_rules! single_slice_predicate {
    ($name:ident, $predicate:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            type_texts: *const CorsaStrRef,
            type_texts_len: usize,
        ) -> bool {
            let Some(type_texts) = (unsafe { collect_strings(type_texts, type_texts_len) }) else {
                return false;
            };
            utils::$predicate(&type_texts)
        }
    };
}

macro_rules! dual_slice_predicate {
    ($name:ident, $predicate:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            type_texts: *const CorsaStrRef,
            type_texts_len: usize,
            property_names: *const CorsaStrRef,
            property_names_len: usize,
        ) -> bool {
            let Some(type_texts) = (unsafe { collect_strings(type_texts, type_texts_len) }) else {
                return false;
            };
            let Some(property_names) =
                (unsafe { collect_strings(property_names, property_names_len) })
            else {
                return false;
            };
            utils::$predicate(&type_texts, &property_names)
        }
    };
}

macro_rules! flow_predicate {
    ($name:ident, $predicate:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            source_texts: *const CorsaStrRef,
            source_texts_len: usize,
            target_texts: *const CorsaStrRef,
            target_texts_len: usize,
        ) -> bool {
            let Some(source_texts) = (unsafe { collect_strings(source_texts, source_texts_len) })
            else {
                return false;
            };
            let Some(target_texts) = (unsafe { collect_strings(target_texts, target_texts_len) })
            else {
                return false;
            };
            utils::$predicate(&source_texts, &target_texts)
        }
    };
}

single_slice_predicate!(
    corsa_utils_is_string_like_type_texts,
    is_string_like_type_texts
);
single_slice_predicate!(
    corsa_utils_is_number_like_type_texts,
    is_number_like_type_texts
);
single_slice_predicate!(
    corsa_utils_is_bigint_like_type_texts,
    is_bigint_like_type_texts
);
single_slice_predicate!(corsa_utils_is_any_like_type_texts, is_any_like_type_texts);
single_slice_predicate!(
    corsa_utils_is_unknown_like_type_texts,
    is_unknown_like_type_texts
);
single_slice_predicate!(
    corsa_utils_is_array_like_type_texts,
    is_array_like_type_texts
);
dual_slice_predicate!(
    corsa_utils_is_promise_like_type_texts,
    is_promise_like_type_texts
);
dual_slice_predicate!(
    corsa_utils_is_error_like_type_texts,
    is_error_like_type_texts
);
flow_predicate!(corsa_utils_has_unsafe_any_flow, has_unsafe_any_flow);
flow_predicate!(corsa_utils_is_unsafe_assignment, is_unsafe_assignment);
flow_predicate!(corsa_utils_is_unsafe_return, is_unsafe_return);
