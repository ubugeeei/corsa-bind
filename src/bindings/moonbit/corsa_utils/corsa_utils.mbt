#borrow(text)
extern "c" fn classify_type_text_native(text : Bytes) -> Bytes = "corsa_utils_classify_type_text_mbt"

#borrow(text)
extern "c" fn is_string_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_string_like_type_texts_mbt"

#borrow(text)
extern "c" fn is_number_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_number_like_type_texts_mbt"

#borrow(text)
extern "c" fn is_bigint_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_bigint_like_type_texts_mbt"

#borrow(text)
extern "c" fn is_any_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_any_like_type_texts_mbt"

#borrow(text)
extern "c" fn is_unknown_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_unknown_like_type_texts_mbt"

#borrow(text)
extern "c" fn is_array_like_type_texts_native(text : Bytes) -> Bool = "corsa_utils_is_array_like_type_texts_mbt"

#borrow(type_text, property_name)
extern "c" fn is_promise_like_type_texts_native(type_text : Bytes, property_name : Bytes) -> Bool = "corsa_utils_is_promise_like_type_texts_mbt"

#borrow(type_text, property_name_a, property_name_b)
extern "c" fn is_error_like_type_texts_native(
  type_text : Bytes,
  property_name_a : Bytes,
  property_name_b : Bytes
) -> Bool = "corsa_utils_is_error_like_type_texts_mbt"

#borrow(source_text, target_text)
extern "c" fn has_unsafe_any_flow_native(source_text : Bytes, target_text : Bytes) -> Bool = "corsa_utils_has_unsafe_any_flow_mbt"

#borrow(source_text, target_text)
extern "c" fn is_unsafe_assignment_native(source_text : Bytes, target_text : Bytes) -> Bool = "corsa_utils_is_unsafe_assignment_mbt"

#borrow(source_text, target_text)
extern "c" fn is_unsafe_return_native(source_text : Bytes, target_text : Bytes) -> Bool = "corsa_utils_is_unsafe_return_mbt"

extern "c" fn error_message_take_native() -> Bytes = "corsa_error_message_take_mbt"

#borrow(path, language_id, text)
extern "c" fn virtual_document_untitled_native(path : Bytes, language_id : Bytes, text : Bytes) -> Int64 = "corsa_virtual_document_untitled_mbt"

extern "c" fn virtual_document_text_native(handle : Int64) -> Bytes = "corsa_virtual_document_text_mbt"

extern "c" fn virtual_document_version_native(handle : Int64) -> Int = "corsa_virtual_document_version_mbt"

#borrow(text)
extern "c" fn virtual_document_splice_native(
  handle : Int64,
  start_line : Int,
  start_character : Int,
  end_line : Int,
  end_character : Int,
  text : Bytes
) -> Bool = "corsa_virtual_document_splice_mbt"

extern "c" fn virtual_document_free_native(handle : Int64) -> Unit = "corsa_virtual_document_free_mbt"

fn string_to_bytes(text : String) -> Bytes {
  let bytes = @utf8.encode(text).to_array()
  bytes.push(0)
  Bytes::from_array(bytes)
}

fn bytes_to_string(bytes : Bytes) -> String {
  @utf8.decode_lossy(bytes[:])
}

pub type VirtualDocumentHandle = Int64

pub fn classify_type_text(text : String) -> String {
  bytes_to_string(classify_type_text_native(string_to_bytes(text)))
}

pub fn is_string_like_type_texts(text : String) -> Bool {
  is_string_like_type_texts_native(string_to_bytes(text))
}

pub fn is_number_like_type_texts(text : String) -> Bool {
  is_number_like_type_texts_native(string_to_bytes(text))
}

pub fn is_bigint_like_type_texts(text : String) -> Bool {
  is_bigint_like_type_texts_native(string_to_bytes(text))
}

pub fn is_any_like_type_texts(text : String) -> Bool {
  is_any_like_type_texts_native(string_to_bytes(text))
}

pub fn is_unknown_like_type_texts(text : String) -> Bool {
  is_unknown_like_type_texts_native(string_to_bytes(text))
}

pub fn is_array_like_type_texts(text : String) -> Bool {
  is_array_like_type_texts_native(string_to_bytes(text))
}

pub fn is_promise_like_type_texts(type_text : String, property_name : String) -> Bool {
  is_promise_like_type_texts_native(string_to_bytes(type_text), string_to_bytes(property_name))
}

pub fn is_error_like_type_texts(type_text : String, property_name_a : String, property_name_b : String) -> Bool {
  is_error_like_type_texts_native(
    string_to_bytes(type_text),
    string_to_bytes(property_name_a),
    string_to_bytes(property_name_b),
  )
}

pub fn has_unsafe_any_flow(source_text : String, target_text : String) -> Bool {
  has_unsafe_any_flow_native(string_to_bytes(source_text), string_to_bytes(target_text))
}

pub fn is_unsafe_assignment(source_text : String, target_text : String) -> Bool {
  is_unsafe_assignment_native(string_to_bytes(source_text), string_to_bytes(target_text))
}

pub fn is_unsafe_return(source_text : String, target_text : String) -> Bool {
  is_unsafe_return_native(string_to_bytes(source_text), string_to_bytes(target_text))
}

pub fn take_last_error() -> String {
  bytes_to_string(error_message_take_native())
}

pub fn virtual_document_untitled(path : String, language_id : String, text : String) -> VirtualDocumentHandle {
  virtual_document_untitled_native(
    string_to_bytes(path),
    string_to_bytes(language_id),
    string_to_bytes(text),
  )
}

pub fn virtual_document_text(handle : VirtualDocumentHandle) -> String {
  bytes_to_string(virtual_document_text_native(handle))
}

pub fn virtual_document_version(handle : VirtualDocumentHandle) -> Int {
  virtual_document_version_native(handle)
}

pub fn virtual_document_splice(
  handle : VirtualDocumentHandle,
  start_line : Int,
  start_character : Int,
  end_line : Int,
  end_character : Int,
  text : String
) -> Bool {
  virtual_document_splice_native(
    handle,
    start_line,
    start_character,
    end_line,
    end_character,
    string_to_bytes(text),
  )
}

pub fn virtual_document_free(handle : VirtualDocumentHandle) -> Unit {
  virtual_document_free_native(handle)
}
