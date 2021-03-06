//! Ruby floating-point numbers.

use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Sub, Mul, Div, Rem},
};
use crate::{
    prelude::*,
    object::{NonNullObject, Ty},
    ruby,
};

/// An instance of Ruby's `Float` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Float(NonNullObject);

impl AsRef<AnyObject> for Float {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Float> for AnyObject {
    #[inline]
    fn from(obj: Float) -> Self { obj.0.into() }
}

impl PartialEq for Float {
    #[inline]
    fn eq(&self, other: &Float) -> bool {
        self.to_f64() == other.to_f64()
    }
}

impl PartialEq<AnyObject> for Float {
    #[inline]
    fn eq(&self, other: &AnyObject) -> bool {
        if let Some(other) = other.to_float() {
            *self == other
        } else {
            false
        }
    }
}

impl PartialEq<f64> for Float {
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        self.to_f64() == *other
    }
}

impl PartialEq<f32> for Float {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.to_f64() == (*other as f64)
    }
}

impl PartialEq<Float> for f64 {
    #[inline]
    fn eq(&self, other: &Float) -> bool {
        *self == other.to_f64()
    }
}

impl PartialEq<Float> for f32 {
    #[inline]
    fn eq(&self, other: &Float) -> bool {
        (*self as f64) == other.to_f64()
    }
}

impl PartialOrd for Float {
    #[inline]
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        self.to_f64().partial_cmp(&other.to_f64())
    }
}

impl PartialOrd<f64> for Float {
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.to_f64().partial_cmp(other)
    }
}

impl PartialOrd<f32> for Float {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.partial_cmp(&(*other as f64))
    }
}

impl PartialOrd<Float> for f64 {
    #[inline]
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        self.partial_cmp(&other.to_f64())
    }
}

impl PartialOrd<Float> for f32 {
    #[inline]
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        Some(other.partial_cmp(self)?.reverse())
    }
}

unsafe impl Object for Float {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!(Ty::FLOAT.id() as u128))
    }

    #[inline]
    fn cast<A: Object>(object: A) -> Option<Self> {
        let object = object.into_any_object();
        if A::unique_id() == Self::unique_id() || object.is_float() {
            unsafe { Some(Self::cast_unchecked(object)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty {
        Ty::FLOAT
    }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool {
        self.ty() == ty
    }
}

impl From<f64> for Float {
    #[inline]
    fn from(f: f64) -> Self {
        unsafe { Self::from_raw(ruby::rb_float_new(f)) }
    }
}

impl From<f32> for Float {
    #[inline]
    fn from(f: f32) -> Self {
        (f as f64).into()
    }
}

impl From<f64> for AnyObject {
    #[inline]
    fn from(f: f64) -> Self {
        Float::from(f).into()
    }
}

impl From<f32> for AnyObject {
    #[inline]
    fn from(f: f32) -> Self {
        Float::from(f).into()
    }
}

impl From<Float> for f64 {
    #[inline]
    fn from(f: Float) -> Self {
        f.to_f64()
    }
}

macro_rules! forward_from_int {
    ($($i:ty)+) => { $(
        impl From<$i> for Float {
            #[inline]
            fn from(i: $i) -> Self {
                f64::from(i).into()
            }
        }
    )+ }
}

forward_from_int! { u32 i32 u16 i16 u8 i8 }

macro_rules! impl_ops {
    ($($op:ident, $op_f:ident;)+) => { $(
        impl $op for Float {
            type Output = Self;

            #[inline]
            fn $op_f(self, rhs: Float) -> Self {
                self.$op_f(rhs.to_f64())
            }
        }

        impl $op<f64> for Float {
            type Output = Self;

            #[inline]
            fn $op_f(self, rhs: f64) -> Self {
                self.to_f64().$op_f(rhs).into()
            }
        }

        impl $op<Float> for f64 {
            type Output = Float;

            #[inline]
            fn $op_f(self, rhs: Float) -> Float {
                self.$op_f(rhs.to_f64()).into()
            }
        }
    )+ }
}

impl_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
    Rem, rem;
}

impl fmt::Display for Float {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl Float {
    /// Performs a lossless conversion of `self` into an `f64`.
    #[inline]
    pub fn to_f64(self) -> f64 {
        unsafe { ruby::rb_float_value(self.raw()) }
    }

    /// Performs a lossy conversion of `self` into an `f32`.
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.to_f64() as f32
    }
}
