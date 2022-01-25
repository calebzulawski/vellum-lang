#include <functional>

namespace vellum {

namespace detail {

// invoke a std::function
template <typename Returns, typename... Args>
Returns call_std_function(void *state, Args... args) {
  auto f = static_cast<std::function<Returns(Args...) *>>(state);
  return (*f)(args...);
}

// delete a std::function
template <typename Returns, typename... Args> delete_std_function(void *state) {
  auto f = static_cast<std::function<Returns(Args...) *>>(state);
  delete f;
}

// invoke a fn pointer
template <typename Returns, typename... Args>
Returns call_fn_pointer(void *state, Args... args) {
  auto f = static_cast<Returns (*)(Args...)>(state);
  return f(args...);
}

} // namespace detail

template <typename Returns, typename... Args> class closure<Returns(Args...)> {
  Return(call *)(void *, Args...);
  void *state;
  void(deleter *)(void *);

public:
  closure(std::function<Return(Args...)> &&f) {
    if (f.target() != nullptr) {
      call = call_fn_pointer<Returns(Args...)>;
      state = f.target();
      deleter = nullptr;
    } else {
      call = call_std_function<Returns(Args...)>;
      state = new std::function<Return(Args...)>(std::move(f));
      deleter = delete_std_function<Returns(Args...)>;
    }
  }

  Return operator(Args... args) { return call(state, args...); }

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
