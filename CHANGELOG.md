# Changelog [![crates.io][crate-badge]][crate] [![docs.rs][docs-badge]][docs]
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

## [Unreleased]
### Added
- `AnyObject::is_{bool|false_or_nil}`
- `Range::{contains|size|len}`
- Pre-interned `SymbolId` getters
- `SymbolId::from_raw`
- `Symbol::is_{static|dynamic}`
- Concrete `Exception` types
  - Allows for safely creating instances of specific exceptions via
    `Exception::new`

### Changed
- Functions to be `const`:
  - `Object::is_fixnum`
  - `SymbolId::raw`
- `AnyException::class` to be faster

## [0.0.9] - 2019-05-29
### Added
- `Range` object type
- Generic `Classify` implementation for `Array<O>` and `Hash<K, V>`
- `Symbol::{all|global_vars}`
- `Object::get_attr`
- `vm::backtrace`
- `Array::subseq`
- Ability to index with `Array::get` using a `Range<usize>`
- `Integer` methods:
  - `to_f64`
  - `is_zero`
  - `to_s_radix`
- `EvalArgs::eval_in_object` and `Object::eval`

### Changed
- `Object::call`, `eval`s, `vm::load`, `vm::require` to be `unsafe` due to
  ability to break assumptions made by types such as `Array<A>`, where `push` is
  called on a value of type `B`
- Functions to be `const`:
  - `AnyObject::is_{nil|undefined|true|false}`
  - `Integer::is_{big|fix}num`
  - `Integer::{{max|min}_fixnum|from_fixnum_wrapping|zero}`
- `Exception::raise` return type to `!`
- `Mixin::attr` and friends to `Mixin::def_attr`
- `Array::{contains|remove_all}` to take an arg that implements `Into<O>`
- `Array::remove_all` to return `Option<O>`
- `fixnum_value` in `Integer` to `to_fixnum`
- Fixnum type for `Integer` from `i64` to `isize`
- `EvalArgs::eval_in` to `eval_in_mixin`

### Fixed
- `Integer::to_fixnum` value for negative numbers

## [0.0.8] - 2019-05-23
### Added
- `Array`-based `PartialEq<[A]>` and `PartialEq<Vec<A>>` implementations on
  `AnyObject`
- `vm::DestroyError` type with the same [`NonZeroI32`] memory optimization as
  `vm::InitError`
- Ability to specify a Ruby version via `ROSY_RUBY=client:version`; e.g.
  `ROSY_RUBY=rvm:2.6.0`

### Changed
- Internal representation of `vm::InitError` to use [`NonZeroI32`]

### Removed
- Falling back to `RUBY` environment variable if `ROSY_RUBY` is not set
- `String`-based `PartialEq<[u8]>` and `PartialEq<Vec<u8>>` implementations on
  `AnyObject`

## [0.0.7] - 2019-05-22
### Added
- `duplicate` methods to `Array` and `Hash`
- More `PartialEq` and `PartialOrd` implementations to `Float` and `Integer`
- `meta` module with Ruby's metadata
- `From<()>` implementation for `AnyObject`
  - Allows for not returning anything in `def_method[_unchecked]!`
- `InstrSeq::eval_unchecked`
- `vm::[set_]safe_level` for managing Ruby's level of paranoia
- `vm::require` and friends that allow for importing files
- `vm::load[_unchecked]` that allows for simply executing files

### Changed
- Where `Float` and `Integer` are located; both now reside in a `num` module
  - Also moved types related to `Integer::[un_]pack` into `num::pack`.
- `Ty` into a `struct` with associated constants instead of an `enum`
  - This prevents the possibility of having an instance of `Ty` from Ruby that
    isn't a valid `enum` variant

### Fixed
- `vm::eval` error checking

## [0.0.6] - 2019-05-21
### Added
- Explicit typing to `Class`
  - `Object::class` and `Object::singleton_class` now return `Class<Self>`
- `Classify` trait for getting a typed `Class` instance for `Self`
- Defining methods on a typed class takes the class's wrapped type as a receiver
  - `MethodFn` now looks similar type-wise to [`FnOnce`] in that it now has an
    associated `Output` type and has `Receiver` instead of `Args`
- Ability to specify the receiver type in `def_method[_unchecked]!`
- `Float` object type
  - Supports arithmetic operations
- Functions for evaluating Ruby scripts to the `vm` module

## [0.0.5] - 2019-05-20
### Added
- Variants of `Class::new_instance` that are `unsafe` or take arguments
- `PartialEq<[A]>` implementation for `Array<O>` where `O: PartialEq<A>`
- `Partial{Eq|Cmp}` implementation over integers for `AnyObject`
- `def_method[_unchecked]!` macros for convenience over
  `Class::def_method[_unchecked]`

### Fixed
- Safety of `Class::new_instance`
- Indexing into a heap-allocated `Array`

### Removed
- `PartialEq + PartialOrd` from `Word`

## [0.0.4] - 2019-05-19
### Added
- `Integer` object type
  - Features `From` conversions for every native Rust integer type, including
    `u128` and `i128`
  - Supports logical bitwise operations
  - Methods:
    - `pack` and `unpack` for converting to and from words respectively
    - `to_truncated` for converting similarly to `as` with primitives
    - `to_value` for converting similarly to `TryFrom` on primitives
      - Has `can_represent` helper method
- [`Debug`] requirement for `Object` trait

## [0.0.3] - 2019-05-18
### Added
- Typed keys and values for `Hash`
- Fast encoding-checking methods to `String` that give `String::to_str` a ~7.5x
  performance improvement when the internal Ruby encoding is UTF-8
  - `encoding_is_ascii_8bit`
  - `encoding_is_utf8`
  - `encoding_is_us_ascii`
- Made some methods on `Encoding` a bit faster:
  - `is_ascii_8bit`
  - `is_utf8`
  - `is_us_ascii`
- Unsafe `protected_no_panic` variant for when the argument is guaranteed by the
  caller to not panic

### Fixed
- `Array::cast` would pass for any objects for `Array<AnyObject>`
- `protected` is now panic-safe via [`std::panic::catch_unwind`]

### Removed
- Fallback call to `is_ascii_whitespace` in `is_whitespace` on `String`

## [0.0.2] - 2019-05-17
### Added
- `_skip_linking` feature flag to hopefully get https://docs.rs/rosy up

## 0.0.1 - 2019-05-17
Initial release

[crate]:       https://crates.io/crates/rosy
[crate-badge]: https://img.shields.io/crates/v/rosy.svg
[docs]:        https://docs.rs/rosy
[docs-badge]:  https://docs.rs/rosy/badge.svg

[Keep a Changelog]:    http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

[Unreleased]: https://github.com/oceanpkg/rosy/compare/v0.0.9...HEAD
[0.0.9]: https://github.com/oceanpkg/rosy/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/oceanpkg/rosy/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/oceanpkg/rosy/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/oceanpkg/rosy/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/oceanpkg/rosy/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/oceanpkg/rosy/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/oceanpkg/rosy/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/oceanpkg/rosy/compare/v0.0.1...v0.0.2

[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[`FnOnce`]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[`std::panic::catch_unwind`]: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
[`NonZeroI32`]: https://doc.rust-lang.org/std/num/struct.NonZeroI32.html
