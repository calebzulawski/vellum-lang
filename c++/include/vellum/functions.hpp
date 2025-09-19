#include <cstdlib>
#include <functional>
#include <stdexcept>
#include <type_traits>
#include <utility>

#include "vellum/abi.hpp"

namespace vellum {

template <typename> struct closure;

namespace detail {

template <typename> struct function_ptr;
template <typename R, typename... Args> struct function_ptr<R(Args...)> {
  using type = R (*)(Args...);
};

template <typename R, typename... Args>
R call_std_function(void *state, Args... args) noexcept try {
  if (!state) {
    std::abort();
  }

  auto *f = static_cast<std::function<R(Args...)> *>(state);
  if constexpr (std::is_void_v<R>) {
    (*f)(std::forward<Args>(args)...);
  } else {
    return (*f)(std::forward<Args>(args)...);
  }
} catch (...) {
  std::abort();
}

template <typename R, typename... Args>
void delete_std_function(void *state) noexcept {
  if (state) {
    auto *f = static_cast<std::function<R(Args...)> *>(state);
    delete f;
  }
}

template <typename R, typename... Args>
R call_fn_pointer(void *state, Args... args) noexcept try {
  if (!state) {
    std::abort();
  }

  auto fp = reinterpret_cast<R (*)(Args...)>(state);
  if constexpr (std::is_void_v<R>) {
    fp(std::forward<Args>(args)...);
  } else {
    return fp(std::forward<Args>(args)...);
  }
} catch (...) {
  std::abort();
}
} // namespace detail

template <typename T> using function = typename detail::function_ptr<T>::type;

template <typename> struct closure;

template <typename R, typename... Args> struct closure<R(Args...)> {
  using result_type = R;
  using function_type = R(Args...);
  using caller_type = R (*)(void *, Args...);
  using deleter_type = void (*)(void *);

  caller_type caller;
  void *state;
  deleter_type deleter;

  closure(std::function<R(Args...)> f) {
    if (!f) {
      caller = nullptr;
      state = nullptr;
      deleter = nullptr;
      return;
    }

    if constexpr (sizeof(function<R(Args...)>) == sizeof(void *)) {
      auto *target = f.template target<function<R(Args...)>>();
      if (target != nullptr) {
        caller = &detail::call_fn_pointer<R, Args...>;
        state = reinterpret_cast<void *>(target);
        deleter = nullptr;
        return;
      }
    }

    caller = &detail::call_std_function<R, Args...>;
    state = new std::function<R(Args...)>(std::move(f));
    deleter = &detail::delete_std_function<R, Args...>;
  }

  template <typename F,
            typename = std::enable_if_t<
                !std::is_same_v<std::decay_t<F>, closure> &&
                !std::is_same_v<std::decay_t<F>, std::function<R(Args...)>>>>
  closure(F &&f) : closure(std::function<R(Args...)>(std::forward<F>(f))) {}

  closure(caller_type caller, void *state,
          deleter_type deleter = nullptr) noexcept
      : caller(caller), state(state), deleter(deleter) {}

  ~closure() noexcept {
    if (deleter) {
      deleter(state);
    }
  }

  closure(closure &&other) noexcept
      : caller(other.caller), state(other.state), deleter(other.deleter) {
    other.caller = nullptr;
    other.state = nullptr;
    other.deleter = nullptr;
  }

  closure &operator=(closure &&other) noexcept {
    if (this != &other) {
      if (deleter) {
        deleter(state);
      }

      caller = other.caller;
      state = other.state;
      deleter = other.deleter;

      other.caller = nullptr;
      other.state = nullptr;
      other.deleter = nullptr;
    }
    return *this;
  }

  closure() = delete;
  closure(const closure &) = delete;
  closure &operator=(const closure &) = delete;

  R operator()(Args... args) const noexcept {
    if (!caller) {
      std::abort();
    }

    if constexpr (std::is_void_v<R>) {
      caller(state, std::forward<Args>(args)...);
    } else {
      return caller(state, std::forward<Args>(args)...);
    }
  }

  explicit operator bool() const noexcept { return caller != nullptr; }

  void swap(closure &other) noexcept {
    std::swap(caller, other.caller);
    std::swap(state, other.state);
    std::swap(deleter, other.deleter);
  }
};

template <typename R, typename... Args>
void swap(closure<R(Args...)> &lhs, closure<R(Args...)> &rhs) noexcept {
  lhs.swap(rhs);
}

template <typename R, typename... Args>
bool operator==(const closure<R(Args...)> &c, std::nullptr_t) noexcept {
  return !c;
}

template <typename R, typename... Args>
bool operator==(std::nullptr_t, const closure<R(Args...)> &c) noexcept {
  return !c;
}

template <typename R, typename... Args>
bool operator!=(const closure<R(Args...)> &c, std::nullptr_t) noexcept {
  return static_cast<bool>(c);
}

template <typename R, typename... Args>
bool operator!=(std::nullptr_t, const closure<R(Args...)> &c) noexcept {
  return static_cast<bool>(c);
}

static_assert(std::is_standard_layout_v<closure<int(int)>>);
static_assert(!std::is_trivially_destructible_v<closure<int(int)>>);
static_assert(std::is_nothrow_move_constructible_v<closure<int(int)>>);
static_assert(std::is_nothrow_move_assignable_v<closure<int(int)>>);
static_assert(std::is_nothrow_swappable_v<closure<int(int)>>);

// ABI invariants for POD closure
static_assert(std::is_standard_layout_v<detail::abi::closure<int(int)>>);
static_assert(std::is_trivially_copyable_v<detail::abi::closure<int(int)>>);

// detail::abi::closure <-> RAII closure conversions
template <typename R, typename... Args>
inline detail::abi::closure<R(Args...)>::closure(
    ::vellum::closure<R(Args...)> &&other) noexcept
    : caller(other.caller), state(other.state), deleter(other.deleter) {
  other.caller = nullptr;
  other.state = nullptr;
  other.deleter = nullptr;
}

template <typename R, typename... Args>
inline detail::abi::closure<R(Args...)>::operator ::vellum::closure<
    R(Args...)>() && noexcept {
  ::vellum::closure<R(Args...)> out(caller, state, deleter);
  caller = nullptr;
  state = nullptr;
  deleter = nullptr;
  return out;
}

} // namespace vellum
