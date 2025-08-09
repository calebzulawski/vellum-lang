#include <memory>
#include <type_traits>

namespace vellum {

namespace detail {
template <typename T> void default_deleter(T *ptr) noexcept { delete ptr; }
template <typename T> void default_deleter(T const *ptr) noexcept {
  delete ptr;
}

template <typename T> struct owned_storage {
  T value;
  void (*deleter)(T) noexcept;

  owned_storage(void (*deleter)(T) noexcept, T value)
      : value(value), deleter(deleter) {}

  owned_storage(const owned_storage &) = delete;
  owned_storage(owned_storage &&) noexcept = default;

  owned_storage &operator=(const owned_storage &) = delete;
  owned_storage &operator=(owned_storage &&) noexcept = default;

  ~owned_storage() {
    if (deleter)
      deleter(value);
  }
};

} // namespace detail

template <typename T> struct owned;
template <typename T> struct owned<T *> : private detail::owned_storage<T *> {
  owned(std::unique_ptr<T> ptr)
      : detail::owned_storage<T *>(detail::default_deleter<T>, ptr.release()) {}

  owned(std::unique_ptr<T, void (*)(T *) noexcept> ptr)
      : detail::owned_storage<T *>(ptr.get_deleter(), ptr.release()) {}

  owned() : detail::owned_storage<T *>(nullptr, nullptr) {}
  owned(const owned &) = delete;
  owned(owned &&) noexcept = default;

  owned &operator=(const owned &) = delete;
  owned &operator=(owned &&) = default;

  operator std::unique_ptr<T, void (*)(T *) noexcept>() && {
    return std::unique_ptr<T, void (*)(T *) noexcept>(this->value,
                                                      this->deleter);
  }

  operator std::shared_ptr<T>() && {
    return std::shared_ptr<T>(this->value, this->deleter);
  }

  T *get() const noexcept {
    return this->value;
  }
};

template <typename T> struct owned<slice<T>>: private detail::owned_storage<slice<T>> {
  owned() : detail::owned_storage<T *>({nullptr, 0}, nullptr) {}
  owned(const owned &) = delete;
  owned(owned &&) noexcept = default;

  owned &operator=(const owned &) = delete;
  owned &operator=(owned &&) = default;
};

// sanity check
static_assert(std::is_standard_layout<owned<int *>>::value);
static_assert(std::is_standard_layout<owned<int const *>>::value);

} // namespace vellum
