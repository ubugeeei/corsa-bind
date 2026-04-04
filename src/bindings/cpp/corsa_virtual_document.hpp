#pragma once

#include <cstdint>
#include <string>
#include <string_view>
#include <utility>

#include "../c/corsa_ffi/include/corsa_utils.h"
#include "corsa_utils.hpp"

namespace corsa::lsp {

inline std::string take_last_error() {
  return utils::take_string(corsa_error_message_take());
}

class virtual_document {
 public:
  virtual_document() = default;
  explicit virtual_document(CorsaVirtualDocument *handle) : handle_(handle) {}

  virtual_document(const virtual_document &) = delete;
  virtual_document &operator=(const virtual_document &) = delete;

  virtual_document(virtual_document &&other) noexcept : handle_(std::exchange(other.handle_, nullptr)) {}
  virtual_document &operator=(virtual_document &&other) noexcept {
    if (this != &other) {
      reset();
      handle_ = std::exchange(other.handle_, nullptr);
    }
    return *this;
  }

  ~virtual_document() { reset(); }

  static virtual_document create(std::string_view uri, std::string_view language_id, std::string_view text) {
    return virtual_document(corsa_virtual_document_new(utils::to_ref(uri), utils::to_ref(language_id), utils::to_ref(text)));
  }

  static virtual_document untitled(std::string_view path, std::string_view language_id, std::string_view text) {
    return virtual_document(corsa_virtual_document_untitled(utils::to_ref(path), utils::to_ref(language_id), utils::to_ref(text)));
  }

  static virtual_document in_memory(
      std::string_view authority,
      std::string_view path,
      std::string_view language_id,
      std::string_view text) {
    return virtual_document(
        corsa_virtual_document_in_memory(utils::to_ref(authority), utils::to_ref(path), utils::to_ref(language_id), utils::to_ref(text)));
  }

  explicit operator bool() const { return handle_ != nullptr; }

  std::string uri() const { return utils::take_string(corsa_virtual_document_uri(handle_)); }
  std::string language_id() const { return utils::take_string(corsa_virtual_document_language_id(handle_)); }
  std::string text() const { return utils::take_string(corsa_virtual_document_text(handle_)); }
  std::string key() const { return utils::take_string(corsa_virtual_document_key(handle_)); }
  std::int32_t version() const { return corsa_virtual_document_version(handle_); }

  bool replace(std::string_view text) { return corsa_virtual_document_replace(handle_, utils::to_ref(text)); }

  bool splice(
      std::uint32_t start_line,
      std::uint32_t start_character,
      std::uint32_t end_line,
      std::uint32_t end_character,
      std::string_view text) {
    return corsa_virtual_document_splice(
        handle_, start_line, start_character, end_line, end_character, utils::to_ref(text));
  }

  void reset() {
    if (handle_ != nullptr) {
      corsa_virtual_document_free(handle_);
      handle_ = nullptr;
    }
  }

 private:
  CorsaVirtualDocument *handle_ = nullptr;
};

}  // namespace corsa::lsp
