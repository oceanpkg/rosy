//! Ruby strings.

use std::{
    borrow::Cow,
    cmp::Ordering,
    convert::TryFrom,
    ffi::{CStr, CString},
    fmt,
    iter::FromIterator,
    os::raw::{c_int, c_long},
    str::Utf8Error,
    string,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

mod encoding;
pub use encoding::*;

/// An instance of Ruby's `String` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct String(NonNullObject);

impl fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.to_str_lossy().fmt(f) }
    }
}

unsafe impl Object for String {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!(Ty::STRING.id() as u128))
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        Some(obj.to_s())
    }

    #[inline]
    fn ty(self) -> Ty { Ty::STRING }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::STRING }
}

impl AsRef<AnyObject> for String {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<String> for AnyObject {
    #[inline]
    fn from(object: String) -> AnyObject { object.0.into() }
}

macro_rules! forward_from {
    ($($t:ty,)+) => { $(
        impl From<$t> for AnyObject {
            #[inline]
            fn from(string: $t) -> Self {
                String::from(string).into()
            }
        }
    )+ }
}

forward_from! {
    &str, &std::string::String,
    &[u8], &Vec<u8>,
    &CStr, &CString,
}

impl From<&str> for String {
    #[inline]
    fn from(s: &str) -> String {
        unsafe { String::from_raw(ruby::rb_utf8_str_new(
            s.as_ptr() as *const _,
            s.len() as _,
        )) }
    }
}

impl From<&std::string::String> for String {
    #[inline]
    fn from(s: &std::string::String) -> String {
        unsafe { String::from_raw(ruby::rb_utf8_str_new(
            s.as_ptr() as *const _,
            s.len() as _,
        )) }
    }
}

impl From<&CStr> for String {
    #[inline]
    fn from(s: &CStr) -> String {
        s.to_bytes().into()
    }
}

impl From<&CString> for String {
    #[inline]
    fn from(s: &CString) -> String {
        s.as_c_str().into()
    }
}

impl From<&[u8]> for String {
    #[inline]
    fn from(bytes: &[u8]) -> String {
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        unsafe { String::from_raw(ruby::rb_str_new(ptr as *const _, len as _)) }
    }
}

impl From<&Vec<u8>> for String {
    #[inline]
    fn from(bytes: &Vec<u8>) -> String {
        bytes.as_slice().into()
    }
}

impl TryFrom<String> for std::string::String {
    type Error = Utf8Error;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.to_string()
    }
}

impl FromIterator<char> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        let string = String::with_capacity(lower_bound);
        unsafe {
            iter.into_iter().for_each(|c| string.push(c));
            string.force_encoding(Encoding::utf8());
        }
        string
    }
}

impl<'a> FromIterator<&'a char> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        iter.into_iter().map(|&c| c).collect()
    }
}

impl<'a> FromIterator<&'a str> for String {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let string = String::new();
        unsafe { iter.into_iter().for_each(|s| string.push_str(s)) };
        string
    }
}

impl<'a> FromIterator<&'a std::string::String> for String {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = &'a std::string::String>
    {
        iter.into_iter().map(|s| s.as_str()).collect()
    }
}

// Make fast byte comparison version of `PartialEq<Self>` when specialization is
// made stable
impl<O: Object> PartialEq<O> for String {
    // If `obj` is not an instance of `String` but responds to `to_str`, then
    // the two strings are compared using `obj.==`.
    #[inline]
    fn eq(&self, obj: &O) -> bool {
        let this = self.raw();
        let that = obj.raw();
        unsafe { ruby::rb_str_equal(this, that) != crate::util::FALSE_VALUE }
    }
}

// Implements `PartialEq` against all relevant string-related types
macro_rules! impl_eq {
    ($($t:ty, $bytes:ident;)+) => { $(
        impl PartialEq<$t> for String {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                // Safe because no other thread can access the bytes
                unsafe { self.as_bytes() == other.$bytes() }
            }
        }

        // Needed to prevent conflict with `PartialEq<impl Object>`
        impl PartialEq<&$t> for String {
            #[inline]
            fn eq(&self, other: &&$t) -> bool {
                *self == **other
            }
        }

        impl PartialEq<String> for $t {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                other == self
            }
        }

        impl PartialEq<String> for &$t {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                other == self
            }
        }
    )+ }
}

impl_eq! {
    [u8],           as_ref;
    Vec<u8>,        as_slice;
    str,            as_bytes;
    string::String, as_bytes;
    CStr,           to_bytes;
    CString,        to_bytes;
}

impl<S: ?Sized + Clone> PartialEq<Cow<'_, S>> for String
    where String: PartialEq<S>
{
    #[inline]
    fn eq(&self, other: &Cow<'_, S>) -> bool {
        self == AsRef::<S>::as_ref(other)
    }
}

impl Eq for String {}

impl PartialOrd for String {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for String {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { ruby::rb_str_cmp(self.raw(), other.raw()).cmp(&0) }
    }
}

impl String {
    #[inline]
    pub(crate) fn rstring(self) -> *mut ruby::RString {
        self.as_any_object()._ptr() as _
    }

    #[inline]
    pub(crate) fn _enc_index(self) -> c_int {
        unsafe { ruby::rb_enc_get_index(self.raw()) }
    }

    // Taken from `enc_get_index_str` in 'encoding.c':
    // `enc_get_index_str` checks for the "encoding" ivar on the string if
    // `ENCODING_GET_INLINED` returns `ENCODING_INLINE_MAX`
    #[inline]
    pub(crate) fn _enc_index_skip_ivar(self) -> c_int {
        unsafe { (*self.rstring()).basic.encoding_index() }
    }

    /// Creates a new empty string with a capacity of 0.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new string with `capacity`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_str_buf_new(capacity as _)) }
    }

    /// Returns a new instance from `s` encoded as `enc`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the bytes are actually encoded this
    /// way. Otherwise, Ruby may make incorrect assumptions about the underlying
    /// data.
    #[inline]
    pub unsafe fn with_encoding(s: impl AsRef<[u8]>, enc: Encoding) -> Self {
        let s = s.as_ref();
        String::from_raw(ruby::rb_external_str_new_with_enc(
            s.as_ptr() as *const _,
            s.len() as _,
            enc._enc(),
        ))
    }

    /// Duplicates the contents of `self` into a new instance.
    #[inline]
    pub fn duplicate(self) -> Self {
        unsafe { Self::from_raw(ruby::rb_str_dup(self.raw())) }
    }

    /// Returns how the bytes of `self` are encoded.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let string = rosy::String::from("¡Hola!");
    /// assert!(string.encoding().is_utf8());
    /// ```
    #[inline]
    pub fn encoding(self) -> Encoding {
        Encoding::_from_index(self._enc_index())
    }

    /// Associates the bytes of `self` with `encoding` without checking whether
    /// `self` is actually encoded that way.
    #[inline]
    pub unsafe fn force_encoding(self, encoding: Encoding) {
        ruby::rb_enc_associate_index(self.raw(), encoding._index());
    }

    /// A fast shortcut to `self.encoding().is_ascii_8bit()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let bytes: &[u8] = &[0, 1, 255, 42];
    /// let string = rosy::String::from(bytes);
    /// assert!(string.encoding_is_ascii_8bit());
    /// ```
    #[inline]
    pub fn encoding_is_ascii_8bit(self) -> bool {
        self._enc_index_skip_ivar() == ruby::rb_encoding::ascii_8bit_index()
    }

    /// A fast shortcut to `self.encoding().is_utf8()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let string = rosy::String::from("hellooo");
    /// assert!(string.encoding_is_utf8());
    /// ```
    #[inline]
    pub fn encoding_is_utf8(self) -> bool {
        self._enc_index_skip_ivar() == ruby::rb_encoding::utf8_index()
    }

    /// A fast shortcut to `self.encoding().is_us_ascii()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Object, String};
    ///
    /// let string = String::from("hellooo");
    /// unsafe {
    ///     string.call_with("encode!", &[String::from("US-ASCII")]);
    /// }
    ///
    /// assert!(string.encoding_is_us_ascii());
    /// ```
    #[inline]
    pub fn encoding_is_us_ascii(self) -> bool {
        self._enc_index_skip_ivar() == ruby::rb_encoding::us_ascii_index()
    }

    /// Returns a reference to the underlying bytes in `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` and the bytes
    /// pointed to by `self` are not changed through the VM or otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let rs = "Hey, I just met you, and this is crazy;
    ///           but here's my number, so call me maybe.";
    /// let rb = rosy::String::from(rs);
    ///
    /// unsafe { assert_eq!(rs.as_bytes(), rb.as_bytes()) };
    /// ```
    #[inline]
    pub unsafe fn as_bytes(&self) -> &[u8] {
        let ptr = (*self.rstring()).start() as *const u8;
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Returns a buffer of the underlying bytes in `self`.
    #[inline]
    pub fn to_bytes(self) -> Vec<u8> {
        unsafe { self.as_bytes().into() }
    }

    /// Returns whether all bytes in `self` satisfy `f`.
    #[inline]
    pub fn bytes_all<F>(self, f: F) -> bool
        where F: FnMut(u8) -> bool
    {
        unsafe { self.as_bytes().iter().cloned().all(f) }
    }

    /// Returns whether any bytes in `self` satisfy `f`.
    #[inline]
    pub fn bytes_any<F>(self, f: F) -> bool
        where F: FnMut(u8) -> bool
    {
        unsafe { self.as_bytes().iter().cloned().any(f) }
    }

    /// Returns a reference to the underlying UTF-8 encoded string in `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` and the
    /// characters pointed to by `self` are not changed through the VM or
    /// otherwise.
    ///
    /// If Ruby believes that the underlying encoding is indeed UTF-8, then we
    /// return the bytes directly without any further checking. However, if the
    /// method `force_encoding` has been called on `self`, then we are
    /// susceptible to getting invalid UTF-8 in a `str` instance, which is UB.
    /// To force a check, one should call
    /// [`str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)
    /// on the result of [`as_bytes`](#method.as_bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let rs = "Somebody once told me the world is gonna roll me...";
    /// let rb = rosy::String::from(rs);
    ///
    /// unsafe { assert_eq!(rb.to_str().unwrap(), rs) };
    /// ```
    pub unsafe fn to_str(&self) -> Result<&str, Utf8Error> {
        if self.encoding_is_utf8() {
            return Ok(self.to_str_unchecked());
        }
        std::str::from_utf8(self.as_bytes())
    }

    /// Returns the underlying string lossy-encoded as UTF-8. See
    /// [`String::from_utf8_lossy`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy)
    /// for more details.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that, if the returned value is a reference
    /// to `self`, the length of `self` and the characters pointed to by `self`
    /// are not changed through the VM or otherwise.
    ///
    /// If Ruby believes that the underlying encoding is indeed UTF-8, then we
    /// return the bytes directly without any further checking. However, if the
    /// method `force_encoding` has been called on `self`, then we are
    /// susceptible to getting invalid UTF-8 in a `str` instance, which is UB.
    /// To force a check, one should call
    /// [`str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)
    /// on the result of [`as_bytes`](#method.as_bytes).
    pub unsafe fn to_str_lossy(&self) -> Cow<'_, str> {
        if self.encoding_is_utf8() {
            return Cow::Borrowed(self.to_str_unchecked());
        }
        std::string::String::from_utf8_lossy(self.as_bytes())
    }

    /// Returns a reference to the underlying bytes of `self` as if they were
    /// UTF-8 encoded.
    ///
    /// # Safety
    ///
    /// Same reasons as [`to_str`](#method.to_str) as well as that no UTF-8
    /// checking is performed.
    #[inline]
    pub unsafe fn to_str_unchecked(&self) -> &str {
        std::str::from_utf8_unchecked(self.as_bytes())
    }

    /// Returns a buffer of the underlying UTF-8 encoded string of `self`.
    #[inline]
    pub fn to_string(self) -> Result<std::string::String, Utf8Error> {
        unsafe { Ok(self.to_str()?.into()) }
    }

    /// Returns the number of bytes in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = "Í'm in Rüby!";
    /// let s2 = "I'm in Ruby!";
    /// let s3 = rosy::String::from(s1);
    ///
    /// assert_eq!(s3.len(), s1.len());
    /// assert_ne!(s3.len(), s2.len());
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        unsafe { (*self.rstring()).len() }
    }

    /// Returns the number of characters in `self`.
    ///
    /// # Examples
    ///
    /// This is a [Unicode](https://en.wikipedia.org/wiki/Unicode)-aware method:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = "Í'm in Rüby!";
    /// let s2 = "I'm in Ruby!";
    /// let s3 = rosy::String::from(s1);
    ///
    /// assert_eq!(s3.char_len(), s1.chars().count());
    /// assert_eq!(s3.char_len(), s2.chars().count());
    /// ```
    #[inline]
    pub fn char_len(self) -> usize {
        unsafe { ruby::rb_str_strlen(self.raw()) as usize }
    }

    /// Returns whether `self` has no characters.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns whether `self` only contains whitespace.
    ///
    /// # Examples
    ///
    /// When `self` is encoded as UTF-8, it checks against all unicode
    /// characters with the property "WSpace=Y":
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::String;
    ///
    /// let space = String::from("\u{0009}\u{000A}\u{000B}\u{000C}\u{000D}\
    ///                           \u{0020}\u{0085}\u{00A0}\u{1680}\u{2000}\
    ///                           \u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\
    ///                           \u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\
    ///                           \u{2028}\u{202F}\u{2029}\u{205F}\u{3000}");
    ///
    /// assert!(space.is_whitespace());
    /// ```
    pub fn is_whitespace(self) -> bool {
        unsafe {
            if let Ok(s) = self.to_str() {
                s.chars().all(|ch| ch.is_whitespace())
            } else {
                // We only care about Unicode whitespace
                false
            }
        }
    }

    /// Returns whether `self` only contains ASCII whitespace.
    pub fn is_ascii_whitespace(self) -> bool {
        self.bytes_all(|b| b.is_ascii_whitespace())
    }

    /// Concatenates `c` to `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn push(self, c: char) {
        self.push_str(c.encode_utf8(&mut [0; 4]))
    }

    /// Concatenates `s` to `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn push_str(self, s: &str) {
        ruby::rb_str_cat(self.raw(), s.as_ptr() as *const _, s.len() as _);
    }

    /// Returns the contents of `self` with an ellipsis (three dots) if it's
    /// longer than `len` _characters_.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = rosy::String::from("Hello, there!");
    /// let s2 = s1.ellipsized(8);
    ///
    /// assert_eq!(s2, "Hello...");
    /// ```
    #[inline]
    pub fn ellipsized(self, len: usize) -> Self {
        if len > c_long::max_value() as usize {
            // Avoid an exception for going negative
            return self.duplicate();
        }
        let len = len as c_long;
        unsafe { Self::from_raw(ruby::rb_str_ellipsize(self.raw(), len)) }
    }

    /// Returns whether the string is locked by the VM.
    #[inline]
    pub fn is_locked(self) -> bool {
        unsafe { (*self.rstring()).is_locked() }
    }

    /// Attempts to call `f` if a lock on `self` can be acquired, returning its
    /// output on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s = rosy::String::from("Hello!");
    /// let result = s.with_lock(|s| s.is_locked());
    ///
    /// assert_eq!(result, Some(true));
    /// ```
    #[inline]
    #[must_use]
    pub fn with_lock<F, O>(self, f: F) -> Option<O>
        where F: FnOnce(Self) -> O
    {
        if self.is_locked() {
            return None;
        }
        unsafe { self.raw_lock() };
        let output = f(self);
        unsafe { self.raw_unlock() };
        Some(output)
    }

    /// Locks the string, preventing others from writing to it.
    ///
    /// # Safety
    ///
    /// The exception raised by the VM must be handled if the string is already
    /// locked.
    #[inline]
    pub unsafe fn raw_lock(self) {
        ruby::rb_str_locktmp(self.raw());
    }

    /// Unlocks the string, allowing others to write to it.
    ///
    /// # Safety
    ///
    /// The exception raised by the VM must be handled if the string is already
    /// unlocked.
    #[inline]
    pub unsafe fn raw_unlock(self) {
        ruby::rb_str_unlocktmp(self.raw());
    }
}

#[cfg(all(test, nightly))]
mod benches {
    use test::{Bencher, black_box};
    use super::*;

    const STRING_MULTIPLE: usize = 10;

    fn create_string() -> String {
        let mut string = std::string::String::new();
        for _ in 0..STRING_MULTIPLE {
            string.push('a');
            string.push('ñ');
            string.push('ß');
        }
        String::from(&*string)
    }

    #[bench]
    fn to_str(b: &mut Bencher) {
        crate::vm::init().unwrap();

        let string = create_string();

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(String::to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }

    #[bench]
    fn to_str_checked(b: &mut Bencher) {
        crate::vm::init().unwrap();

        let string = create_string();
        let enc = String::from("ASCII-8BIT");
        let sym = "force_encoding";
        unsafe { string.call_with_protected(sym, &[enc]).unwrap(); }

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(String::to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }

    #[bench]
    fn to_str_no_lookup(b: &mut Bencher) {
        crate::vm::init().unwrap();

        unsafe fn to_str(s: &String) -> Result<&str, Utf8Error> {
            std::str::from_utf8(s.as_bytes())
        }

        let string = create_string();

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }
}
