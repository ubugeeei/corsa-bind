package corsautils

/*
#cgo CFLAGS: -I${SRCDIR}/../../c/corsa_ffi/include
#cgo LDFLAGS: -L${SRCDIR}/../../../../target/debug -lcorsa_ffi
#include "corsa_utils.h"
*/
import "C"

import "fmt"

type VirtualDocument struct {
	ptr *C.CorsaVirtualDocument
}

func takeError() error {
	message := takeString(C.corsa_error_message_take())
	if message == "" {
		return nil
	}
	return fmt.Errorf("%s", message)
}

func NewVirtualDocument(uri string, languageID string, text string) (*VirtualDocument, error) {
	uriValue := newBorrowedString(uri)
	defer uriValue.free()
	languageValue := newBorrowedString(languageID)
	defer languageValue.free()
	textValue := newBorrowedString(text)
	defer textValue.free()
	ptr := C.corsa_virtual_document_new(uriValue.ref, languageValue.ref, textValue.ref)
	if ptr == nil {
		return nil, takeError()
	}
	return &VirtualDocument{ptr: ptr}, nil
}

func NewUntitledVirtualDocument(path string, languageID string, text string) (*VirtualDocument, error) {
	pathValue := newBorrowedString(path)
	defer pathValue.free()
	languageValue := newBorrowedString(languageID)
	defer languageValue.free()
	textValue := newBorrowedString(text)
	defer textValue.free()
	ptr := C.corsa_virtual_document_untitled(pathValue.ref, languageValue.ref, textValue.ref)
	if ptr == nil {
		return nil, takeError()
	}
	return &VirtualDocument{ptr: ptr}, nil
}

func NewInMemoryVirtualDocument(authority string, path string, languageID string, text string) (*VirtualDocument, error) {
	authorityValue := newBorrowedString(authority)
	defer authorityValue.free()
	pathValue := newBorrowedString(path)
	defer pathValue.free()
	languageValue := newBorrowedString(languageID)
	defer languageValue.free()
	textValue := newBorrowedString(text)
	defer textValue.free()
	ptr := C.corsa_virtual_document_in_memory(authorityValue.ref, pathValue.ref, languageValue.ref, textValue.ref)
	if ptr == nil {
		return nil, takeError()
	}
	return &VirtualDocument{ptr: ptr}, nil
}

func (value *VirtualDocument) Close() {
	if value == nil || value.ptr == nil {
		return
	}
	C.corsa_virtual_document_free(value.ptr)
	value.ptr = nil
}

func (value *VirtualDocument) URI() string {
	return takeString(C.corsa_virtual_document_uri(value.ptr))
}
func (value *VirtualDocument) LanguageID() string {
	return takeString(C.corsa_virtual_document_language_id(value.ptr))
}
func (value *VirtualDocument) Text() string {
	return takeString(C.corsa_virtual_document_text(value.ptr))
}
func (value *VirtualDocument) Key() string {
	return takeString(C.corsa_virtual_document_key(value.ptr))
}
func (value *VirtualDocument) Version() int32 {
	return int32(C.corsa_virtual_document_version(value.ptr))
}

func (value *VirtualDocument) Replace(text string) error {
	textValue := newBorrowedString(text)
	defer textValue.free()
	if !bool(C.corsa_virtual_document_replace(value.ptr, textValue.ref)) {
		return takeError()
	}
	return nil
}

func (value *VirtualDocument) Splice(startLine uint32, startCharacter uint32, endLine uint32, endCharacter uint32, text string) error {
	textValue := newBorrowedString(text)
	defer textValue.free()
	if !bool(C.corsa_virtual_document_splice(
		value.ptr,
		C.uint32_t(startLine),
		C.uint32_t(startCharacter),
		C.uint32_t(endLine),
		C.uint32_t(endCharacter),
		textValue.ref,
	)) {
		return takeError()
	}
	return nil
}
