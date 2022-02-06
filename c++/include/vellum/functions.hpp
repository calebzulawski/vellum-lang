#include <functional>

namespace vellum {

// 
// Regular function pointer
//
namespace detail {
template <typename> struct function_helper;
template <typename Returns, typename... Args>
struct function_helper<Returns(Args...)> {
  using type = Returns (*)(Args...);
};
} // namespace detail
template <typename T> using function = typename detail::function_helper<T>::type;

//
// Closures
//

namespace detail {

// invoke a std::function
template <typename Returns, typename... Args>
Returns call_std_function(void *state, Args... args) noexcept {
  auto f = static_cast<std::function<Returns(Args...)> *>(state);
  return (*f)(args...);
}

// delete a std::function
template <typename Returns, typename... Args>
void delete_std_function(void *state) noexcept {
  auto f = static_cast<std::function<Returns(Args...)> *>(state);
  delete f;
}

// invoke a fn pointer
template <typename Returns, typename... Args>
Returns call_fn_pointer(void *state, Args... args) noexcept {
  auto f = static_cast<Returns (*)(Args...)>(state);
  return f(args...);
}

} // namespace detail

template <typename> class closure;
template <typename Returns, typename... Args> class closure<Returns(Args...)> {
  Returns (*call)(void *, Args...);
  void *state;
  void (*deleter)(void *);

public:
  closure(std::function<Returns(Args...)> &&f) {
    if (f.target() != nullptr) {
      call = detail::call_fn_pointer<Returns(Args...)>;
      state = f.target();
      deleter = nullptr;
    } else {
      call = detail::call_std_function<Returns(Args...)>;
      state = new std::function<Returns(Args...)>(std::move(f));
      deleter = detail::delete_std_function<Returns(Args...)>;
    }
  }

  Returns operator()(Args... args) { return call(state, args...); }

  ~closure() {
    if (deleter)
      deleter(state);
  }

  closure() = delete;
  closure(const closure &) = delete;
  closure(closure &&) = default;

  closure<Returns(Args...)> &operator=(const closure &) = delete;
  closure<Returns(Args...)> &operator=(closure &&) = default;
};

} // namespace vellum
