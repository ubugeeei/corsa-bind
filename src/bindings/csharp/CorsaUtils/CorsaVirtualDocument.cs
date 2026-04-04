using System.Runtime.InteropServices;
using System.Text;

namespace Corsa.Utils;

public sealed class CorsaVirtualDocument : IDisposable
{
    private const string LibraryName = "corsa_ffi";
    private nint _handle;

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct CorsaStrRef(nint ptr, nuint len)
    {
        public readonly nint Ptr = ptr;
        public readonly nuint Len = len;
    }

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct CorsaString
    {
        public readonly nint Ptr;
        public readonly nuint Len;
    }

    [DllImport(LibraryName, EntryPoint = "corsa_error_message_take")]
    private static extern CorsaString TakeErrorMessageNative();
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_new")]
    private static extern nint CreateNative(CorsaStrRef uri, CorsaStrRef languageId, CorsaStrRef text);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_untitled")]
    private static extern nint CreateUntitledNative(CorsaStrRef path, CorsaStrRef languageId, CorsaStrRef text);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_in_memory")]
    private static extern nint CreateInMemoryNative(CorsaStrRef authority, CorsaStrRef path, CorsaStrRef languageId, CorsaStrRef text);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_uri")]
    private static extern CorsaString UriNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_language_id")]
    private static extern CorsaString LanguageIdNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_text")]
    private static extern CorsaString TextNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_key")]
    private static extern CorsaString KeyNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_version")]
    private static extern int VersionNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_replace")]
    [return: MarshalAs(UnmanagedType.I1)]
    private static extern bool ReplaceNative(nint value, CorsaStrRef text);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_splice")]
    [return: MarshalAs(UnmanagedType.I1)]
    private static extern bool SpliceNative(nint value, uint startLine, uint startCharacter, uint endLine, uint endCharacter, CorsaStrRef text);
    [DllImport(LibraryName, EntryPoint = "corsa_virtual_document_free")]
    private static extern void FreeNative(nint value);
    [DllImport(LibraryName, EntryPoint = "corsa_utils_string_free")]
    private static extern void FreeString(CorsaString value);

    private CorsaVirtualDocument(nint handle) => _handle = handle;

    public static CorsaVirtualDocument Create(string uri, string languageId, string text) =>
        new(CallHandle(uri, languageId, text, CreateNative));

    public static CorsaVirtualDocument Untitled(string path, string languageId, string text) =>
        new(CallHandle(path, languageId, text, CreateUntitledNative));

    public static CorsaVirtualDocument InMemory(string authority, string path, string languageId, string text)
    {
        using var authorityValue = Utf8Input.FromString(authority);
        using var pathValue = Utf8Input.FromString(path);
        using var languageValue = Utf8Input.FromString(languageId);
        using var textValue = Utf8Input.FromString(text);
        var handle = CreateInMemoryNative(authorityValue.Ref, pathValue.Ref, languageValue.Ref, textValue.Ref);
        return handle == 0 ? throw CreateException() : new CorsaVirtualDocument(handle);
    }

    public string Uri => TakeString(UriNative(_handle));
    public string LanguageId => TakeString(LanguageIdNative(_handle));
    public string Text => TakeString(TextNative(_handle));
    public string Key => TakeString(KeyNative(_handle));
    public int Version => VersionNative(_handle);

    public void Replace(string text)
    {
        using var value = Utf8Input.FromString(text);
        if (!ReplaceNative(_handle, value.Ref)) throw CreateException();
    }

    public void Splice(uint startLine, uint startCharacter, uint endLine, uint endCharacter, string text)
    {
        using var value = Utf8Input.FromString(text);
        if (!SpliceNative(_handle, startLine, startCharacter, endLine, endCharacter, value.Ref)) throw CreateException();
    }

    public void Dispose()
    {
        if (_handle != 0) FreeNative(_handle);
        _handle = 0;
    }

    private static nint CallHandle(string first, string second, string third, Func<CorsaStrRef, CorsaStrRef, CorsaStrRef, nint> callback)
    {
        using var firstValue = Utf8Input.FromString(first);
        using var secondValue = Utf8Input.FromString(second);
        using var thirdValue = Utf8Input.FromString(third);
        var handle = callback(firstValue.Ref, secondValue.Ref, thirdValue.Ref);
        return handle == 0 ? throw CreateException() : handle;
    }

    private static string TakeString(CorsaString value)
    {
        try
        {
            return value.Ptr == 0 || value.Len == 0 ? string.Empty : unsafe { Encoding.UTF8.GetString((byte*)value.Ptr, checked((int)value.Len)) };
        }
        finally
        {
            FreeString(value);
        }
    }

    private static Exception CreateException()
    {
        var message = TakeString(TakeErrorMessageNative());
        return new InvalidOperationException(message.Length == 0 ? "corsa ffi error" : message);
    }

    private sealed class Utf8Input(nint buffer, nuint length) : IDisposable
    {
        public CorsaStrRef Ref { get; } = new(buffer, length);

        public static Utf8Input FromString(string value)
        {
            if (value.Length == 0) return new Utf8Input(0, 0);
            var bytes = Encoding.UTF8.GetBytes(value);
            var buffer = Marshal.AllocHGlobal(bytes.Length);
            Marshal.Copy(bytes, 0, buffer, bytes.Length);
            return new Utf8Input(buffer, (nuint)bytes.Length);
        }

        public void Dispose()
        {
            if (buffer != 0) Marshal.FreeHGlobal(buffer);
        }
    }
}
