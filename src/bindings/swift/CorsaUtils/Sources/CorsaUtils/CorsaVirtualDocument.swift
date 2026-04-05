public enum CorsaFfiError: Error {
    case message(String)
}

@_silgen_name("corsa_error_message_take")
func takeErrorMessageNative() -> CorsaString

@_silgen_name("corsa_virtual_document_new")
private func createVirtualDocumentNative(_ uri: CorsaStrRef, _ languageID: CorsaStrRef, _ text: CorsaStrRef) -> UnsafeMutableRawPointer?

@_silgen_name("corsa_virtual_document_untitled")
private func createUntitledVirtualDocumentNative(_ path: CorsaStrRef, _ languageID: CorsaStrRef, _ text: CorsaStrRef) -> UnsafeMutableRawPointer?

@_silgen_name("corsa_virtual_document_in_memory")
private func createInMemoryVirtualDocumentNative(
    _ authority: CorsaStrRef,
    _ path: CorsaStrRef,
    _ languageID: CorsaStrRef,
    _ text: CorsaStrRef
) -> UnsafeMutableRawPointer?

@_silgen_name("corsa_virtual_document_uri")
private func virtualDocumentUriNative(_ value: UnsafeMutableRawPointer?) -> CorsaString

@_silgen_name("corsa_virtual_document_language_id")
private func virtualDocumentLanguageIdNative(_ value: UnsafeMutableRawPointer?) -> CorsaString

@_silgen_name("corsa_virtual_document_text")
private func virtualDocumentTextNative(_ value: UnsafeMutableRawPointer?) -> CorsaString

@_silgen_name("corsa_virtual_document_key")
private func virtualDocumentKeyNative(_ value: UnsafeMutableRawPointer?) -> CorsaString

@_silgen_name("corsa_virtual_document_version")
private func virtualDocumentVersionNative(_ value: UnsafeMutableRawPointer?) -> Int32

@_silgen_name("corsa_virtual_document_replace")
private func virtualDocumentReplaceNative(_ value: UnsafeMutableRawPointer?, _ text: CorsaStrRef) -> Bool

@_silgen_name("corsa_virtual_document_splice")
private func virtualDocumentSpliceNative(
    _ value: UnsafeMutableRawPointer?,
    _ startLine: UInt32,
    _ startCharacter: UInt32,
    _ endLine: UInt32,
    _ endCharacter: UInt32,
    _ text: CorsaStrRef
) -> Bool

@_silgen_name("corsa_virtual_document_free")
private func freeVirtualDocumentNative(_ value: UnsafeMutableRawPointer?)

public final class CorsaVirtualDocument {
    private var handle: UnsafeMutableRawPointer?

    private init(handle: UnsafeMutableRawPointer) {
        self.handle = handle
    }

    public init(uri: String, languageID: String, text: String) throws {
        self.handle = try CorsaVirtualDocument.create(uri: uri, languageID: languageID, text: text)
    }

    public static func untitled(path: String, languageID: String, text: String) throws -> CorsaVirtualDocument {
        try CorsaVirtualDocument(handle: withThreeRefs(path, languageID, text, createUntitledVirtualDocumentNative))
    }

    public static func inMemory(authority: String, path: String, languageID: String, text: String) throws -> CorsaVirtualDocument {
        try CorsaVirtualDocument(handle: withFourRefs(authority, path, languageID, text, createInMemoryVirtualDocumentNative))
    }

    deinit {
        freeVirtualDocumentNative(handle)
    }

    public var uri: String { takeString(virtualDocumentUriNative(handle)) }
    public var languageID: String { takeString(virtualDocumentLanguageIdNative(handle)) }
    public var text: String { takeString(virtualDocumentTextNative(handle)) }
    public var key: String { takeString(virtualDocumentKeyNative(handle)) }
    public var version: Int32 { virtualDocumentVersionNative(handle) }

    public func replace(_ text: String) throws {
        let ok = withStrRef(text) { virtualDocumentReplaceNative(handle, $0) }
        if !ok { throw ffiError() }
    }

    public func splice(
        startLine: UInt32,
        startCharacter: UInt32,
        endLine: UInt32,
        endCharacter: UInt32,
        text: String
    ) throws {
        let ok = withStrRef(text) {
            virtualDocumentSpliceNative(handle, startLine, startCharacter, endLine, endCharacter, $0)
        }
        if !ok { throw ffiError() }
    }

    private static func create(uri: String, languageID: String, text: String) throws -> UnsafeMutableRawPointer {
        try withThreeRefs(uri, languageID, text, createVirtualDocumentNative)
    }
}

private func withThreeRefs(
    _ first: String,
    _ second: String,
    _ third: String,
    _ body: (CorsaStrRef, CorsaStrRef, CorsaStrRef) -> UnsafeMutableRawPointer?
) throws -> UnsafeMutableRawPointer {
    let refs = BorrowedRefs([first, second, third])
    return try refs.refs.withUnsafeBufferPointer {
        guard let handle = body($0[0], $0[1], $0[2]) else { throw ffiError() }
        return handle
    }
}

private func withFourRefs(
    _ first: String,
    _ second: String,
    _ third: String,
    _ fourth: String,
    _ body: (CorsaStrRef, CorsaStrRef, CorsaStrRef, CorsaStrRef) -> UnsafeMutableRawPointer?
) throws -> UnsafeMutableRawPointer {
    let refs = BorrowedRefs([first, second, third, fourth])
    return try refs.refs.withUnsafeBufferPointer {
        guard let handle = body($0[0], $0[1], $0[2], $0[3]) else { throw ffiError() }
        return handle
    }
}

func ffiError() -> CorsaFfiError {
    let message = takeString(takeErrorMessageNative())
    return .message(message.isEmpty ? "corsa ffi error" : message)
}
