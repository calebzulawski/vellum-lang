# Specification

This document specifies the layouts of Vellum's types.

## Numbers
Unsigned integers: `u8`, `u16`, `u32`, `u64`, `usize`

Signed integers: `i8`, `i16`, `i32`, `i64`, `isize`

Floating point: `f32`, `f64`

## Structs
Structs have identical layout to C structs.
```
struct Foo {
  bar: Bar,
  baz: Baz,
}
```

is identical to the following C:

```
struct Foo {
  Bar bar;
  Baz baz;
};
```
### Abstract types

Structs may also be abstract, meaning their layout is unknown and they can only be referenced via pointers:
```
struct Foo;
```

## Pointers
### "Regular" pointers
* `const * T` is a pointer to immutable data
* `mut * T` is a pointer to mutable data
### Strings
`const string` and `mut string` are special pointers to a null-terminated string, equivalent to `char[]` in C.
### Slices
A slice `const [T]` or `mut [T]` is a pointer to a contiguous array of `T`, with the following layout in C:
```
struct {
  T *array;
  size_t length;
}
```
### Owned pointers
`owned * T` is a pointer to "owned" data, with the following layout in C:
```
struct {
  pointer data;
  void (*deleter)(pointer);
};
```
`data` is a mutable pointer to the data, and calling `deleter` on that frees the pointer.  The data or the deleter is permitted to be null.  `pointer` may be a regular pointer, a string, or a slice.

## Functions
Functions have the same calling convention as C functions.
```
function foo(bar: Bar, baz: Baz) -> Qux;
```
is identical to the following C:
```
Qux foo(Bar bar, Baz baz);
```

Functions are not permitted to unwind (e.g. C++ exceptions, Rust panics).

### Function pointers
Pointers to functions are permitted, for example:
```
struct Foo {
  foo: function (bar: Bar) -> Baz,
}
```
Function pointers are permitted to be null.

### Closures
```
closure (bar: Bar) -> Baz
```
is implemented in C as:
```
struct {
  Baz (*call)(void *, Bar);
  void *state;
  void (*deleter)(void *);
};
```
The closure is invoked by calling `call` with `state` as the first argument, followed by the closure arguments.  The closure is deallocated by calling `deleter` with `state` as its argument.
