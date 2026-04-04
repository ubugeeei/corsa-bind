const std = @import("std");
const utils = @import("corsa_utils.zig");

pub const c = utils.c;

pub const VirtualDocument = struct {
    handle: ?*c.CorsaVirtualDocument,

    pub fn init(uri: []const u8, language_id: []const u8, text: []const u8) !VirtualDocument {
        return .{ .handle = c.corsa_virtual_document_new(utils.toRef(uri), utils.toRef(language_id), utils.toRef(text)) orelse return error.CorsaFfiError };
    }

    pub fn untitled(path: []const u8, language_id: []const u8, text: []const u8) !VirtualDocument {
        return .{ .handle = c.corsa_virtual_document_untitled(utils.toRef(path), utils.toRef(language_id), utils.toRef(text)) orelse return error.CorsaFfiError };
    }

    pub fn inMemory(authority: []const u8, path: []const u8, language_id: []const u8, text: []const u8) !VirtualDocument {
        return .{ .handle = c.corsa_virtual_document_in_memory(utils.toRef(authority), utils.toRef(path), utils.toRef(language_id), utils.toRef(text)) orelse return error.CorsaFfiError };
    }

    pub fn deinit(self: *VirtualDocument) void {
        if (self.handle) |handle| c.corsa_virtual_document_free(handle);
        self.handle = null;
    }

    pub fn uri(self: VirtualDocument, allocator: std.mem.Allocator) ![]u8 {
        return utils.takeString(allocator, c.corsa_virtual_document_uri(self.handle));
    }

    pub fn languageId(self: VirtualDocument, allocator: std.mem.Allocator) ![]u8 {
        return utils.takeString(allocator, c.corsa_virtual_document_language_id(self.handle));
    }

    pub fn text(self: VirtualDocument, allocator: std.mem.Allocator) ![]u8 {
        return utils.takeString(allocator, c.corsa_virtual_document_text(self.handle));
    }

    pub fn key(self: VirtualDocument, allocator: std.mem.Allocator) ![]u8 {
        return utils.takeString(allocator, c.corsa_virtual_document_key(self.handle));
    }

    pub fn version(self: VirtualDocument) i32 {
        return c.corsa_virtual_document_version(self.handle);
    }

    pub fn replace(self: VirtualDocument, text: []const u8) !void {
        if (!c.corsa_virtual_document_replace(self.handle, utils.toRef(text))) return error.CorsaFfiError;
    }

    pub fn splice(
        self: VirtualDocument,
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
        text: []const u8,
    ) !void {
        if (!c.corsa_virtual_document_splice(
            self.handle,
            start_line,
            start_character,
            end_line,
            end_character,
            utils.toRef(text),
        )) return error.CorsaFfiError;
    }
};

pub fn takeLastError(allocator: std.mem.Allocator) ![]u8 {
    return utils.takeString(allocator, c.corsa_error_message_take());
}
