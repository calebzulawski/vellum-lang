#include <cstddef>
#include <type_traits>

namespace vellum {

template <typename T> struct slice {
  using element_type = T;
  using value_type = std::remove_cv_t<T>;
  using size_type = std::size_t;
  using difference_type = std::ptrdiff_t;
  using pointer = T*;
  using const_pointer = const T*;
  using reference = T&;
  using const_reference = const T&;

  slice() : data_(nullptr), size_(0) {}
  slice(const slice &) noexcept = default;
  slice(slice &&) noexcept = default;

  slice &operator=(const slice &) = default;
  slice &operator=(slice &&) = default;

  constexpr pointer data() const noexcept { return data_; }
  constexpr size_type size() const noexcept { return size_; }

  pointer data_;
  size_type size_;
};

}  // namespace vellum
