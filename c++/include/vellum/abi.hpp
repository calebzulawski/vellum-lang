#pragma once

#include <cstddef>
#include <iterator>
#include <stdexcept>
#include <type_traits>

namespace vellum {

// Forward declarations for RAII wrappers used in conversions
template <typename> struct owned_ptr;
template <typename> struct owned_slice;
template <typename> struct closure;

namespace detail {
namespace abi {

// Slice POD (fat pointer): pointer + length
template <typename T> struct slice {
  using element_type = T;
  using value_type = std::remove_cv_t<T>;
  using size_type = std::size_t;
  using difference_type = std::ptrdiff_t;
  using pointer = T *;
  using const_pointer = const T *;
  using reference = T &;
  using const_reference = const T &;
  using iterator = T *;
  using const_iterator = const T *;
  using reverse_iterator = std::reverse_iterator<iterator>;
  using const_reverse_iterator = std::reverse_iterator<const_iterator>;

  pointer data;
  size_type len;

  slice() noexcept : data(nullptr), len(0) {}
  slice(pointer ptr, size_type length) noexcept : data(ptr), len(length) {}

  reference operator[](size_type idx) const noexcept { return data[idx]; }
  reference at(size_type idx) const {
    if (idx >= len)
      throw std::out_of_range("slice::at");
    return data[idx];
  }
  iterator begin() const noexcept { return data; }
  iterator end() const noexcept { return data + len; }
  bool empty() const noexcept { return len == 0; }
  size_type size() const noexcept { return len; }
};

// Owned pointer POD: pointer + deleter(data)
template <typename T> struct [[nodiscard]] owned_ptr {
  using element_type = T;
  using pointer = T *;
  using deleter_type = void (*)(T *);

  pointer data;
  deleter_type deleter;

  // Conversions to/from RAII wrappers (definitions in pointers.hpp)
  owned_ptr(::vellum::owned_ptr<T> &&other) noexcept;
  operator ::vellum::owned_ptr<T>() && noexcept;
};

// Owned slice POD: slice + deleter(slice)
template <typename T> struct [[nodiscard]] owned_slice {
  using element_type = T;
  using pointer = T *;
  using size_type = std::size_t;
  using deleter_type = void (*)(slice<T>);

  slice<T> slice_data;
  deleter_type deleter;

  // Conversions to/from RAII wrappers (definitions in pointers.hpp)
  owned_slice(::vellum::owned_slice<T> &&other) noexcept;
  operator ::vellum::owned_slice<T>() && noexcept;
};

// Closure POD
template <typename> struct [[nodiscard]] closure;

template <typename R, typename... Args>
struct [[nodiscard]] closure<R(Args...)> {
  using result_type = R;
  using function_type = R(Args...);
  using caller_type = R (*)(void *, Args...);
  using deleter_type = void (*)(void *);

  caller_type caller;
  void *state;
  deleter_type deleter;

  // Conversions to/from RAII closure (definitions in functions.hpp)
  closure(::vellum::closure<R(Args...)> &&other) noexcept;
  operator ::vellum::closure<R(Args...)>() && noexcept;
};

// owned<T> selector similar to RAII, but resolves to POD owned_* types
template <typename T> struct is_slice : std::false_type {};
template <typename T> struct is_slice<slice<T>> : std::true_type {};

template <typename T> struct element {
  using type = void;
};
template <typename T> struct element<slice<T>> {
  using type = T;
};

template <typename T>
using owned_selector =
    std::conditional_t<is_slice<T>::value,
                       owned_slice<typename element<T>::type>,
                       owned_ptr<std::remove_pointer_t<T>>>;

template <typename T> using owned = owned_selector<T>;

// ABI invariants
static_assert(std::is_standard_layout_v<owned_ptr<int>>);
static_assert(std::is_trivially_copyable_v<owned_ptr<int>>);
static_assert(std::is_standard_layout_v<owned_slice<int>>);
static_assert(std::is_trivially_copyable_v<owned_slice<int>>);
static_assert(std::is_standard_layout_v<closure<int(int)>>);
static_assert(std::is_trivially_copyable_v<closure<int(int)>>);

} // namespace abi
} // namespace detail

} // namespace vellum
