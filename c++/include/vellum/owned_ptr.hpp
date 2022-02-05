#include <memory>

namespace vellum {

namespace detail {
template <typename T> void default_deleter(T *ptr) noexcept { delete ptr; }
} // namespace detail

template <typename T> class owned_ptr {
  T *ptr;
  void (*deleter)(T *);

public:
  owned_ptr(std::unique_ptr<T> ptr)
      : ptr(ptr.release()), deleter(detail::default_deleter<T>) {}

  owned_ptr(std::unique_ptr<T, void (*)(T *)> ptr)
      : deleter(ptr.get_deleter()), ptr(ptr.release()) {}

  owned_ptr() : ptr(nullptr), deleter(nullptr) {}
  owned_ptr(const owned_ptr<T> &) = delete;
  owned_ptr(owned_ptr<T> &&) = default;

  owned_ptr<T> &operator=(const owned_ptr<T> &) = delete;
  owned_ptr<T> &operator=(owned_ptr<T> &&) = default;

  ~owned_ptr() {
    if (deleter)
      deleter(ptr);
  }

  operator std::unique_ptr<T, void (*)(T *)>() && {
    return std::unique_ptr<T, void (*)(T *)>(ptr, deleter);
  }

  operator std::shared_ptr<T>() && { return std::shared_ptr<T>(ptr, deleter); }
};

} // namespace vellum
