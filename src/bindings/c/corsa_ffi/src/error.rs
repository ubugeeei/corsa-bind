use std::cell::RefCell;

use crate::types::{CorsaString, into_c_string};

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn set_last_error(error: impl ToString) {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(error.to_string()));
}

pub fn clear_last_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

#[unsafe(no_mangle)]
pub extern "C" fn corsa_error_message_take() -> CorsaString {
    LAST_ERROR.with(|slot| {
        let message = slot.borrow_mut().take().unwrap_or_default();
        into_c_string(message.as_str())
    })
}
