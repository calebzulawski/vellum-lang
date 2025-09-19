#include <cstddef>
#include <iterator>
#include <memory>
#include <type_traits>
#include <utility>
#include <vector>

#include "vellum/abi.hpp"

namespace vellum {

// Bring slice into this namespace for ergonomic use
using detail::abi::slice;

template <typename T> struct owned_ptr {
  using element_type = T;
  using pointer = T *;
  using deleter_type = void (*)(T *);

  pointer data;
  deleter_type deleter;

  owned_ptr() noexcept : data(nullptr), deleter(nullptr) {}
  owned_ptr(pointer ptr, deleter_type del) noexcept : data(ptr), deleter(del) {}

  ~owned_ptr() { reset(); }

  owned_ptr(const owned_ptr &) = delete;
  owned_ptr &operator=(const owned_ptr &) = delete;

  owned_ptr(owned_ptr &&other) noexcept
      : data(other.data), deleter(other.deleter) {
    other.data = nullptr;
    other.deleter = nullptr;
  }

  owned_ptr &operator=(owned_ptr &&other) noexcept {
    if (this != &other) {
      reset();
      data = other.data;
      deleter = other.deleter;
      other.data = nullptr;
      other.deleter = nullptr;
    }
    return *this;
  }

  void reset(pointer ptr = nullptr, deleter_type del = nullptr) noexcept {
    if (data && deleter) {
      deleter(data);
    }
    data = ptr;
    deleter = del;
  }

  pointer release() noexcept {
    pointer result = data;
    data = nullptr;
    deleter = nullptr;
    return result;
  }

  void swap(owned_ptr &other) noexcept {
    std::swap(data, other.data);
    std::swap(deleter, other.deleter);
  }

  pointer get() const noexcept { return data; }
  deleter_type get_deleter() const noexcept { return deleter; }
  explicit operator bool() const noexcept { return data != nullptr; }
  T &operator*() const noexcept { return *data; }
  pointer operator->() const noexcept { return data; }

  std::unique_ptr<T, deleter_type> to_unique_ptr() && {
    auto result = std::unique_ptr<T, deleter_type>(data, deleter);
    data = nullptr;
    deleter = nullptr;
    return result;
  }

  std::shared_ptr<T> to_shared_ptr() && {
    auto result = std::shared_ptr<T>(data, deleter);
    data = nullptr;
    deleter = nullptr;
    return result;
  }

  owned_ptr(std::unique_ptr<T> ptr) noexcept
      : data(ptr.release()), deleter([](T *p) noexcept { delete p; }) {}

  template <typename D>
  owned_ptr(std::unique_ptr<T, D> ptr) noexcept
      : data(ptr.release()), deleter(ptr.get_deleter()) {
    static_assert(std::is_convertible_v<D, deleter_type>,
                  "Deleter must be convertible to void(*)(T*) noexcept");
  }
};

template <typename T> struct owned_slice {
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
  using deleter_type = void (*)(slice<T>);

  slice<T> slice_data;
  deleter_type deleter;

  owned_slice() noexcept : slice_data(), deleter(nullptr) {}

  owned_slice(slice<T> s, deleter_type del) noexcept
      : slice_data(s), deleter(del) {}

  ~owned_slice() { reset(); }

  owned_slice(const owned_slice &) = delete;
  owned_slice &operator=(const owned_slice &) = delete;

  owned_slice(owned_slice &&other) noexcept
      : slice_data(other.slice_data), deleter(other.deleter) {
    other.slice_data = slice<T>();
    other.deleter = nullptr;
  }

  owned_slice &operator=(owned_slice &&other) noexcept {
    if (this != &other) {
      reset();
      slice_data = other.slice_data;
      deleter = other.deleter;
      other.slice_data = slice<T>();
      other.deleter = nullptr;
    }
    return *this;
  }

  explicit owned_slice(size_type count, const T &value = T())
      : slice_data(new T[count], count),
        deleter([](slice<T> s) noexcept { delete[] s.data; }) {
    for (auto it = this->begin(); it != this->end(); ++it) {
      *it = value;
    }
  }

  void reset() noexcept {
    if (slice_data.data && deleter) {
      deleter(slice_data);
    }
    slice_data = slice<T>();
    deleter = nullptr;
  }

  slice<T> release() noexcept {
    slice<T> result = slice_data;
    slice_data = slice<T>();
    deleter = nullptr;
    return result;
  }

  void swap(owned_slice &other) noexcept {
    std::swap(slice_data, other.slice_data);
    std::swap(deleter, other.deleter);
  }

  // Element access (delegate to slice)
  reference operator[](size_type idx) const noexcept { return slice_data[idx]; }
  reference at(size_type idx) const { return slice_data.at(idx); }
  reference front() const noexcept { return slice_data.front(); }
  reference back() const noexcept { return slice_data.back(); }
  pointer data() const noexcept { return slice_data.data; }

  // Iterators (delegate to slice)
  iterator begin() const noexcept { return slice_data.begin(); }
  iterator end() const noexcept { return slice_data.end(); }
  const_iterator cbegin() const noexcept { return slice_data.cbegin(); }
  const_iterator cend() const noexcept { return slice_data.cend(); }
  reverse_iterator rbegin() const noexcept { return slice_data.rbegin(); }
  reverse_iterator rend() const noexcept { return slice_data.rend(); }
  const_reverse_iterator crbegin() const noexcept {
    return slice_data.crbegin();
  }
  const_reverse_iterator crend() const noexcept { return slice_data.crend(); }

  // Capacity (delegate to slice)
  bool empty() const noexcept { return slice_data.empty(); }
  size_type size() const noexcept { return slice_data.size(); }

  slice<T> get_slice() const noexcept { return slice_data; }
  deleter_type get_deleter() const noexcept { return deleter; }
  explicit operator bool() const noexcept { return slice_data.data != nullptr; }
  operator slice<T>() const noexcept { return slice_data; }

  // Conversions
  std::vector<T> to_vector() && {
    std::vector<T> result;
    if (slice_data.data && slice_data.len > 0) {
      result.assign(slice_data.begin(), slice_data.end());
    }
    reset();
    return result;
  }

  owned_slice(const std::vector<T> &vec)
      : slice_data(nullptr, vec.size()),
        deleter([](slice<T> s) noexcept { delete[] s.data; }) {
    if (!vec.empty()) {
      T *new_data = new T[vec.size()];
      std::copy(vec.begin(), vec.end(), new_data);
      slice_data = slice<T>(new_data, vec.size());
    }
  }

  owned_slice(std::vector<T> &&vec)
      : slice_data(nullptr, vec.size()),
        deleter([](slice<T> s) noexcept { delete[] s.data; }) {
    if (!vec.empty()) {
      T *new_data = new T[vec.size()];
      std::move(vec.begin(), vec.end(), new_data);
      slice_data = slice<T>(new_data, vec.size());
    }
  }
};

template <typename T>
typename slice<T>::iterator begin(const slice<T> &s) noexcept {
  return s.begin();
}
template <typename T>
typename slice<T>::iterator end(const slice<T> &s) noexcept {
  return s.end();
}
template <typename T>
typename slice<T>::pointer data(const slice<T> &s) noexcept {
  return s.data;
}
template <typename T>
typename slice<T>::size_type size(const slice<T> &s) noexcept {
  return s.size();
}
template <typename T> bool empty(const slice<T> &s) noexcept {
  return s.empty();
}

template <typename T>
typename owned_slice<T>::iterator begin(const owned_slice<T> &s) noexcept {
  return s.begin();
}
template <typename T>
typename owned_slice<T>::iterator end(const owned_slice<T> &s) noexcept {
  return s.end();
}
template <typename T>
typename owned_slice<T>::pointer data(const owned_slice<T> &s) noexcept {
  return s.data();
}
template <typename T>
typename owned_slice<T>::size_type size(const owned_slice<T> &s) noexcept {
  return s.size();
}
template <typename T> bool empty(const owned_slice<T> &s) noexcept {
  return s.empty();
}

template <typename T> void swap(owned_ptr<T> &a, owned_ptr<T> &b) noexcept {
  a.swap(b);
}
template <typename T> void swap(owned_slice<T> &a, owned_slice<T> &b) noexcept {
  a.swap(b);
}

namespace detail {
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
} // namespace detail

template <typename T> using owned = detail::owned_selector<T>;

// Implement conversions declared in abi.hpp
template <typename T>
inline detail::abi::owned_ptr<T>::owned_ptr(
    ::vellum::owned_ptr<T> &&other) noexcept
    : data(other.data), deleter(other.deleter) {
  other.data = nullptr;
  other.deleter = nullptr;
}

template <typename T>
inline detail::abi::owned_ptr<T>::operator ::vellum::owned_ptr<T>()
    && noexcept {
  ::vellum::owned_ptr<T> out(data, deleter);
  data = nullptr;
  deleter = nullptr;
  return out;
}

template <typename T>
inline detail::abi::owned_slice<T>::owned_slice(
    ::vellum::owned_slice<T> &&other) noexcept
    : slice_data(), deleter(nullptr) {
  auto del = other.get_deleter();
  auto s = other.release();
  slice_data = s;
  deleter = del;
}

template <typename T>
inline detail::abi::owned_slice<T>::operator ::vellum::owned_slice<T>()
    && noexcept {
  ::vellum::owned_slice<T> out(slice_data, deleter);
  slice_data = slice<T>();
  deleter = nullptr;
  return out;
}

} // namespace vellum
