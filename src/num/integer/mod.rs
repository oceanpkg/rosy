//! Ruby integers.

use std::{
    cmp::Ordering,
    ffi::c_void,
    fmt,
    mem,
    ops,
    os::raw::c_int,
    slice,
};
use crate::{
    prelude::*,
    object::{NonNullObject, Ty},
    ruby,
};

pub mod pack;
use pack::Word;

/// An instance of Ruby's `Integer` class.
///
/// This type supports conversions to/from _all_ of Rust's integer primitives,
/// from 8-bit through 128-bit. For working with bigger numbers, one can operate
/// over a buffer of `Word`s via [`pack`](#method.pack) and
/// [`unpack`](#method.unpack).
///
/// # Logical Binary Operations
///
/// The logical operations [AND], [OR], and [XOR] are all supported:
///
/// ```
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// use rosy::Integer;
///
/// let a_val = 0b1101;
/// let b_val = 0b0111;
/// let a_int = Integer::from(a_val);
/// let b_int = Integer::from(b_val);
///
/// assert_eq!(a_int & b_int, a_val & b_val);
/// assert_eq!(a_int | b_int, a_val | b_val);
/// assert_eq!(a_int ^ b_int, a_val ^ b_val);
/// # }).unwrap();
/// ```
///
/// [AND]: https://en.wikipedia.org/wiki/Logical_conjunction
/// [OR]:  https://en.wikipedia.org/wiki/Logical_disjunction
/// [XOR]: https://en.wikipedia.org/wiki/Exclusive_or
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Integer(NonNullObject);

impl AsRef<AnyObject> for Integer {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Integer> for AnyObject {
    #[inline]
    fn from(obj: Integer) -> Self { obj.0.into() }
}

impl<O: Object> PartialEq<O> for Integer {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        let other = other.as_any_object();
        let other_id = O::unique_id();

        let is_num =
            other_id == Self::unique_id() ||
            other_id == Float::unique_id() ||
            other.is_float() ||
            other.is_integer();

        if is_num {
            unsafe { ruby::rb_big_eq(self.raw(), other.raw()) != 0 }
        } else {
            self.as_any_object() == other
        }
    }
}

impl Eq for Integer {}

impl PartialOrd for Integer {
    #[inline]
    fn partial_cmp(&self, other: &Integer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Integer {
    #[inline]
    fn cmp(&self, other: &Integer) -> Ordering {
        let raw = unsafe { ruby::rb_big_cmp(self.raw(), other.raw()) };
        crate::util::value_to_fixnum(raw).cmp(&0)
    }
}

unsafe impl Object for Integer {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!((Ty::FIXNUM.id() as u128) | ((Ty::BIGNUM.id() as u128) << 8)))
    }

    #[inline]
    fn cast<A: Object>(object: A) -> Option<Self> {
        if object.into_any_object().is_integer() {
            unsafe { Some(Self::cast_unchecked(object)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty {
        if self.is_fixnum() {
            Ty::FIXNUM
        } else {
            Ty::BIGNUM
        }
    }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool {
        self.ty() == ty
    }
}

impl From<usize> for Integer {
    #[inline]
    fn from(int: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_uint2inum(int)) }
    }
}

impl From<isize> for Integer {
    #[inline]
    fn from(int: isize) -> Self {
        unsafe { Self::from_raw(ruby::rb_int2inum(int)) }
    }
}

impl From<u128> for Integer {
    #[inline]
    fn from(int: u128) -> Self {
        Self::unpack(slice::from_ref(&int))
    }
}

impl From<i128> for Integer {
    #[inline]
    fn from(int: i128) -> Self {
        Self::unpack(slice::from_ref(&int))
    }
}

impl From<u64> for Integer {
    #[inline]
    fn from(int: u64) -> Self {
        if mem::size_of::<u64>() == mem::size_of::<usize>() {
            (int as usize).into()
        } else {
            Self::unpack(slice::from_ref(&int))
        }
    }
}

impl From<i64> for Integer {
    #[inline]
    fn from(int: i64) -> Self {
        if mem::size_of::<i64>() == mem::size_of::<isize>() {
            (int as isize).into()
        } else {
            Self::unpack(slice::from_ref(&int))
        }
    }
}

impl From<u32> for Integer {
    #[inline]
    fn from(int: u32) -> Self {
        (int as usize).into()
    }
}

impl From<i32> for Integer {
    #[inline]
    fn from(int: i32) -> Self {
        (int as isize).into()
    }
}

impl From<u16> for Integer {
    #[inline]
    fn from(int: u16) -> Self {
        (int as usize).into()
    }
}

impl From<i16> for Integer {
    #[inline]
    fn from(int: i16) -> Self {
        (int as isize).into()
    }
}

impl From<u8> for Integer {
    #[inline]
    fn from(int: u8) -> Self {
        (int as usize).into()
    }
}

impl From<i8> for Integer {
    #[inline]
    fn from(int: i8) -> Self {
        (int as isize).into()
    }
}

macro_rules! forward_from {
    ($($t:ty)+) => { $(
        impl From<$t> for AnyObject {
            #[inline]
            fn from(int: $t) -> Self {
                Integer::from(int).into()
            }
        }
    )+ }
}

forward_from! {
    usize u128 u64 u32 u16 u8
    isize i128 i64 i32 i16 i8
}

macro_rules! forward_cmp {
    ($($t:ty)+) => { $(
        impl PartialEq<$t> for Integer {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                if let Some(this) = self.to_value::<$t>() {
                    this == *other
                } else {
                    false
                }
            }
        }

        impl PartialEq<Integer> for $t {
            #[inline]
            fn eq(&self, other: &Integer) -> bool {
                other == self
            }
        }

        impl PartialOrd<$t> for Integer {
            #[inline]
            fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                let (can_represent, is_negative) = self._can_represent::<$t>();

                if can_represent {
                    let mut this: $t = 0;
                    let sign = self.pack(slice::from_mut(&mut this));
                    debug_assert!(!sign.did_overflow(), "Overflow on {}", self);

                    Some(this.cmp(other))
                } else if is_negative {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }

        impl PartialOrd<Integer> for $t {
            #[inline]
            fn partial_cmp(&self, other: &Integer) -> Option<Ordering> {
                Some(other.partial_cmp(self)?.reverse())
            }
        }
    )+ }
}

forward_cmp! {
    usize u128 u64 u32 u16 u8
    isize i128 i64 i32 i16 i8
}

macro_rules! impl_bit_ops {
    ($($op:ident, $f:ident, $r:ident;)+) => { $(
        impl ops::$op for Integer {
            type Output = Self;

            #[inline]
            fn $f(self, rhs: Self) -> Self {
                let (a, b) = match (self.to_fixnum(), rhs.to_fixnum()) {
                    (Some(a), Some(b)) => {
                        return Self::from_fixnum_wrapping(a.$f(b));
                    },
                    (Some(_), None) => (rhs, self),
                    (None,    _)    => (self, rhs),
                };
                unsafe { Self::from_raw(ruby::$r(a.raw(), b.raw())) }
            }
        }
    )+ }
}

impl_bit_ops! {
    BitAnd, bitand, rb_big_and;
    BitOr,  bitor,  rb_big_or;
    BitXor, bitxor, rb_big_xor;
}

impl fmt::Display for Integer {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl Integer {
    #[inline]
    const unsafe fn _from_raw(raw: ruby::VALUE) -> Self {
        Self(NonNullObject::from_raw(raw))
    }

    /// Returns an instance with a value of 0.
    #[inline]
    pub const fn zero() -> Self {
        Self::from_fixnum_wrapping(0)
    }

    /// Returns the maximum value that may be used as a fixnum.
    ///
    /// # Examples
    ///
    /// This value is equal to
    /// [`isize::max_value()`](https://doc.rust-lang.org/std/primitive.isize.html#method.max_value)
    /// shifted right by 1.
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let fixnum = isize::max_value() >> 1;
    /// let integer = Integer::from(fixnum);
    ///
    /// assert_eq!(integer, Integer::max_fixnum());
    /// assert_eq!(integer.to_fixnum(), Some(fixnum));
    /// ```
    #[inline]
    pub const fn max_fixnum() -> Self {
        Self::from_fixnum_wrapping(isize::max_value() >> 1)
    }

    /// Returns the minimum value that may be used as a fixnum.
    ///
    /// # Examples
    ///
    /// This value is equal to
    /// [`isize::min_value()`](https://doc.rust-lang.org/std/primitive.isize.html#method.min_value)
    /// shifted right by 1.
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let fixnum = isize::min_value() >> 1;
    /// let integer = Integer::from(fixnum);
    ///
    /// assert_eq!(integer, Integer::min_fixnum());
    /// assert_eq!(integer.to_fixnum(), Some(fixnum));
    /// ```
    #[inline]
    pub const fn min_fixnum() -> Self {
        Self::from_fixnum_wrapping(isize::min_value() >> 1)
    }

    /// Returns an instance from the fixed-size number, wrapping at the most
    /// significant bit.
    #[inline]
    pub const fn from_fixnum_wrapping(n: isize) -> Self {
        unsafe { Self::_from_raw(crate::util::fixnum_to_value(n)) }
    }

    /// Unpacks the contents of `buf` into a new instance.
    #[inline]
    pub fn unpack<W: Word>(buf: &[W]) -> Self {
        Self::unpack_using(buf, Default::default())
    }

    /// Unpacks the contents of `buf` into a new instance using `options`.
    #[inline]
    pub fn unpack_using<W: Word>(buf: &[W], options: pack::Options) -> Self {
        use ruby::integer_flags::*;

        let ptr = buf.as_ptr() as *const c_void;
        let len = buf.len();
        let size = mem::size_of::<W>();

        let two = (W::IS_SIGNED as c_int) * PACK_2COMP;
        let neg = (options.is_negative as c_int) * PACK_NEGATIVE;
        let flags = options.flags() | two | neg;

        unsafe {
            Self::from_raw(ruby::rb_integer_unpack(ptr, len, size, 0, flags))
        }
    }

    /// Returns whether `self == 0`.
    #[inline]
    pub fn is_zero(self) -> bool {
        if let Some(fixnum) = self.to_fixnum() {
            fixnum == 0
        } else {
            unsafe { ruby::rb_bigzero_p(self.raw()) != 0 }
        }
    }

    /// Returns whether `self >= 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let big = Integer::from(u128::max_value());
    /// let fix = Integer::from(isize::max_value() / 2);
    /// # assert!(big.is_bignum());
    /// # assert!(fix.is_fixnum());
    ///
    /// assert!(big.is_positive());
    /// assert!(fix.is_positive());
    /// ```
    #[inline]
    pub fn is_positive(self) -> bool {
        !self.is_negative()
    }

    /// Returns whether `self < 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let big = Integer::from(i128::min_value());
    /// let fix = Integer::from(isize::min_value() / 2);
    /// # assert!(big.is_bignum());
    /// # assert!(fix.is_fixnum());
    ///
    /// assert!(big.is_negative());
    /// assert!(fix.is_negative());
    /// ```
    #[inline]
    pub fn is_negative(self) -> bool {
        if let Some(fixnum) = self.to_fixnum() {
            fixnum < 0
        } else {
            unsafe { ruby::rb_big_sign(self.raw()) == 0 }
        }
    }

    /// Returns whether `self` is a variable-width integer.
    #[inline]
    pub const fn is_bignum(self) -> bool {
        !self.is_fixnum()
    }

    /// Returns whether `self` is a fixed-width integer.
    #[inline]
    pub const fn is_fixnum(self) -> bool {
        crate::util::value_is_fixnum(self.0.raw())
    }

    /// Returns the value of the fixed-width integer stored in `self`, if it is
    /// not a bignum.
    #[inline]
    pub fn to_fixnum(self) -> Option<isize> {
        if self.is_fixnum() {
            Some(crate::util::value_to_fixnum(self.raw()))
        } else {
            None
        }
    }

    /// Returns the value of the fixed-width integer stored in `self`, assuming
    /// it is not a bignum.
    ///
    /// # Safety
    ///
    /// This method is not marked as `unsafe` because using it on a bignum is
    /// simply a programming error and will not result in memory or type
    /// unsafety.
    #[inline]
    pub const fn to_fixnum_unchecked(self) -> isize {
        crate::util::value_to_fixnum(self.0.raw())
    }

    /// Converts `self` to `W` if it can be represented as `W`.
    #[inline]
    pub fn to_value<W: Word>(self) -> Option<W> {
        if !self.can_represent::<W>() {
            return None;
        }
        let mut val = W::ZERO;
        let sign = self.pack(slice::from_mut(&mut val));
        debug_assert!(!sign.did_overflow());
        Some(val)
    }

    /// Converts `self` to its inner value as `W`, truncating on too large or
    /// small of a value.
    ///
    /// # Examples
    ///
    /// This has the same exact behavior as an
    /// [`as` cast](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions)
    /// between integer primitives in Rust:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// let val = u16::max_value();
    /// let int = rosy::Integer::from(val);
    ///
    /// assert_eq!(int.to_truncated::<u16>(), val);
    /// assert_eq!(int.to_truncated::<u8>(),  255);
    /// assert_eq!(int.to_truncated::<i8>(),   -1);
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn to_truncated<W: Word>(self) -> W {
        let mut val = W::ZERO;
        self.pack(slice::from_mut(&mut val));
        val
    }

    /// Returns `self` as a 64-bit floating point number.
    ///
    /// Note that this is very likely to be a lossy conversion.
    #[inline]
    pub fn to_f64(self) -> f64 {
        if let Some(fixnum) = self.to_fixnum() {
            fixnum as f64
        } else {
            unsafe { ruby::rb_big2dbl(self.raw()) }
        }
    }

    /// Returns a string for `self` in the given base, or an exception if one is
    /// raised.
    pub fn to_s_radix(self, radix: u32) -> Result<String> {
        unsafe {
            crate::protected_no_panic(|| self.to_s_radix_unchecked(radix))
        }
    }

    /// Returns a string for `self` in the given base.
    ///
    /// # Safety
    ///
    /// An exception will be raised if `self` is too large or if `radix > 36`.
    #[inline]
    pub unsafe fn to_s_radix_unchecked(self, radix: u32) -> String {
        String::from_raw(ruby::rb_big2str(self.raw(), radix as _))
    }

    /// Packs the contents of `self` into `buf` with the platform's native byte
    /// order.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use std::slice;
    /// use rosy::Integer;
    ///
    /// let value = u128::max_value() / 0xF00F;
    /// let integer = Integer::from(value);
    ///
    /// let mut buf = [0u128; 2];
    /// integer.pack(&mut buf);
    /// assert_eq!(buf[0], value);
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn pack<W: Word>(self, buf: &mut [W]) -> pack::Sign {
        self.pack_using(Default::default(), buf)
    }

    /// Packs the contents of `self` into `buf` using `options`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use std::slice;
    /// use rosy::num::{Integer, pack::Options};
    ///
    /// let value = u128::max_value() / 0xF00F;
    /// let integer = Integer::from(value);
    ///
    /// let mut be_buf = [0u128; 1];
    /// integer.pack_using(Options::big_endian(), &mut be_buf);
    /// assert_eq!(be_buf[0], value.to_be());
    ///
    /// let mut le_buf = [0u128; 1];
    /// integer.pack_using(Options::little_endian(), &mut le_buf);
    /// assert_eq!(le_buf[0], value.to_le());
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn pack_using<W: Word>(
        self,
        options: pack::Options,
        buf: &mut [W],
    ) -> pack::Sign {
        use ruby::integer_flags::*;
        use pack::Sign::*;

        let raw = self.raw();
        let ptr = buf.as_mut_ptr() as *mut c_void;
        let num = buf.len();
        let size = mem::size_of::<W>();

        let flags = options.flags() | ((W::IS_SIGNED as c_int) * PACK_2COMP);

        match unsafe { ruby::rb_integer_pack(raw, ptr, num, size, 0, flags) } {
            02 => Positive { did_overflow: true },
            01 => Positive { did_overflow: false },
            00 => Zero,
            -1 => Negative { did_overflow: false },
            _  => Negative { did_overflow: true },
        }
    }

    fn _can_represent_raw(self, signed: bool, word_size: usize) -> (bool, bool) {
        // Taken from documentation of `rb_absint_singlebit_p`
        let is_negative = self.is_negative();
        let raw = self.raw();

        let mut nlz_bits = 0;
        let mut size = unsafe { ruby::rb_absint_size(raw, &mut nlz_bits) };

        let can_represent = if signed {
            let single_bit = unsafe { ruby::rb_absint_singlebit_p(raw) != 0 };
            if nlz_bits == 0 && !(is_negative && single_bit) {
                size += 1
            }
            size <= word_size
        } else if is_negative {
            false
        } else {
            size <= word_size
        };
        (can_represent, is_negative)
    }

    #[inline]
    fn _can_represent<W: Word>(self) -> (bool, bool) {
        self._can_represent_raw(W::IS_SIGNED, mem::size_of::<W>())
    }

    /// Returns whether `self` can represent the word type `W`.
    #[inline]
    pub fn can_represent<W: Word>(self) -> bool {
        self._can_represent::<W>().0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        crate::vm::init().unwrap();

        macro_rules! test {
            ($($t:ty)+) => { $({
                let values = [
                    0,
                    <$t>::min_value(),
                    <$t>::max_value(),
                ];
                for &value in &values {
                    let int = Integer::from(value);
                    assert_eq!(int.to_s(), value.to_string());

                    let converted = int.to_value::<$t>()
                        .expect(&format!("{} cannot represent {}", int, value));
                    assert_eq!(converted, value);

                    let mut buf: [$t; 1] = [0];
                    let sign = int.pack(&mut buf);
                    assert!(
                        !sign.did_overflow(),
                        "Packing {} from {} overflowed as {:?}",
                        int,
                        value,
                        sign,
                    );
                    assert_eq!(buf[0], value);
                }
            })+ }
        }

        crate::protected(|| {
            test! {
                usize u128 u64 u32 u16 u8
                isize i128 i64 i32 i16 i8
            }
        }).unwrap();
    }

    #[test]
    fn bit_ops() {
        crate::vm::init().unwrap();

        macro_rules! test {
            ($($t:ty)+) => { $({
                let min = <$t>::min_value();
                let max = <$t>::max_value();

                let min_int = Integer::from(min);
                let max_int = Integer::from(max);

                assert_eq!(min_int & min_int, min & min);
                assert_eq!(min_int & max_int, min & max);
                assert_eq!(max_int & min_int, max & min);
                assert_eq!(max_int & max_int, max & max);

                assert_eq!(min_int | min_int, min | min);
                assert_eq!(min_int | max_int, min | max);
                assert_eq!(max_int | min_int, max | min);
                assert_eq!(max_int | max_int, max | max);

                assert_eq!(min_int ^ min_int, min ^ min);
                assert_eq!(min_int ^ max_int, min ^ max);
                assert_eq!(max_int ^ min_int, max ^ min);
                assert_eq!(max_int ^ max_int, max ^ max);
            })+ };
        }

        crate::protected(|| {
            test! {
                usize u128 u64 u32 u16 u8
                isize i128 i64 i32 i16 i8
            }
        }).unwrap();
    }
}
