use std::str::FromStr;

use corsa_lsp::{VirtualChange, VirtualDocument};
use lsp_types::{Position, Range, Uri};

use crate::{
    error::{clear_last_error, set_last_error},
    types::{CorsaStrRef, CorsaString, into_c_string},
};

pub struct CorsaVirtualDocument {
    inner: VirtualDocument,
}

fn read_text(input: CorsaStrRef, label: &str) -> Option<String> {
    let Some(text) = (unsafe { input.as_str() }) else {
        set_last_error(format!("{label} must be valid UTF-8"));
        return None;
    };
    Some(text.to_owned())
}

unsafe fn document_ref<'a>(value: *const CorsaVirtualDocument) -> Option<&'a VirtualDocument> {
    let Some(value) = (unsafe { value.as_ref() }) else {
        set_last_error("virtual document handle is null");
        return None;
    };
    Some(&value.inner)
}

unsafe fn document_mut<'a>(value: *mut CorsaVirtualDocument) -> Option<&'a mut VirtualDocument> {
    let Some(value) = (unsafe { value.as_mut() }) else {
        set_last_error("virtual document handle is null");
        return None;
    };
    Some(&mut value.inner)
}

fn into_document_ptr(document: VirtualDocument) -> *mut CorsaVirtualDocument {
    clear_last_error();
    Box::into_raw(Box::new(CorsaVirtualDocument { inner: document }))
}

fn parse_uri(input: CorsaStrRef) -> Option<Uri> {
    let uri = read_text(input, "uri")?;
    match Uri::from_str(uri.as_str()) {
        Ok(uri) => Some(uri),
        Err(error) => {
            set_last_error(format!("invalid uri: {error}"));
            None
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_new(
    uri: CorsaStrRef,
    language_id: CorsaStrRef,
    text: CorsaStrRef,
) -> *mut CorsaVirtualDocument {
    let Some(uri) = parse_uri(uri) else {
        return std::ptr::null_mut();
    };
    let Some(language_id) = read_text(language_id, "language_id") else {
        return std::ptr::null_mut();
    };
    let Some(text) = read_text(text, "text") else {
        return std::ptr::null_mut();
    };
    into_document_ptr(VirtualDocument::new(uri, language_id, text))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_untitled(
    path: CorsaStrRef,
    language_id: CorsaStrRef,
    text: CorsaStrRef,
) -> *mut CorsaVirtualDocument {
    let Some(path) = read_text(path, "path") else {
        return std::ptr::null_mut();
    };
    let Some(language_id) = read_text(language_id, "language_id") else {
        return std::ptr::null_mut();
    };
    let Some(text) = read_text(text, "text") else {
        return std::ptr::null_mut();
    };
    match VirtualDocument::untitled(path, language_id, text) {
        Ok(document) => into_document_ptr(document),
        Err(error) => {
            set_last_error(error);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_in_memory(
    authority: CorsaStrRef,
    path: CorsaStrRef,
    language_id: CorsaStrRef,
    text: CorsaStrRef,
) -> *mut CorsaVirtualDocument {
    let Some(authority) = read_text(authority, "authority") else {
        return std::ptr::null_mut();
    };
    let Some(path) = read_text(path, "path") else {
        return std::ptr::null_mut();
    };
    let Some(language_id) = read_text(language_id, "language_id") else {
        return std::ptr::null_mut();
    };
    let Some(text) = read_text(text, "text") else {
        return std::ptr::null_mut();
    };
    match VirtualDocument::in_memory(authority, path, language_id, text) {
        Ok(document) => into_document_ptr(document),
        Err(error) => {
            set_last_error(error);
            std::ptr::null_mut()
        }
    }
}

fn read_document_text(
    value: *const CorsaVirtualDocument,
    select: impl FnOnce(&VirtualDocument) -> &str,
) -> CorsaString {
    let Some(document) = (unsafe { document_ref(value) }) else {
        return CorsaString::default();
    };
    clear_last_error();
    into_c_string(select(document))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_uri(
    value: *const CorsaVirtualDocument,
) -> CorsaString {
    read_document_text(value, |document| document.uri.as_str())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_language_id(
    value: *const CorsaVirtualDocument,
) -> CorsaString {
    read_document_text(value, |document| document.language_id.as_str())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_text(
    value: *const CorsaVirtualDocument,
) -> CorsaString {
    read_document_text(value, |document| document.text.as_str())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_key(
    value: *const CorsaVirtualDocument,
) -> CorsaString {
    let Some(document) = (unsafe { document_ref(value) }) else {
        return CorsaString::default();
    };
    clear_last_error();
    into_c_string(document.key().as_str())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_version(value: *const CorsaVirtualDocument) -> i32 {
    let Some(document) = (unsafe { document_ref(value) }) else {
        return 0;
    };
    clear_last_error();
    document.version
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_replace(
    value: *mut CorsaVirtualDocument,
    text: CorsaStrRef,
) -> bool {
    let Some(document) = (unsafe { document_mut(value) }) else {
        return false;
    };
    let Some(text) = read_text(text, "text") else {
        return false;
    };
    match document.apply_changes(&[VirtualChange::replace(text)]) {
        Ok(_) => {
            clear_last_error();
            true
        }
        Err(error) => {
            set_last_error(error);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_splice(
    value: *mut CorsaVirtualDocument,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
    text: CorsaStrRef,
) -> bool {
    let Some(document) = (unsafe { document_mut(value) }) else {
        return false;
    };
    let Some(text) = read_text(text, "text") else {
        return false;
    };
    let range = Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    );
    match document.apply_changes(&[VirtualChange::splice(range, text)]) {
        Ok(_) => {
            clear_last_error();
            true
        }
        Err(error) => {
            set_last_error(error);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_virtual_document_free(value: *mut CorsaVirtualDocument) {
    if value.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(value) };
}
