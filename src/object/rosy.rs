use std::{
    ffi::c_void,
    fmt,
    marker::PhantomData,
    ptr,
};
use crate::{
    object::NonNullObject,
    prelude::*,
    ruby::{self, rb_data_type_t, rb_data_type_t_function},
};

/// An instance of a Ruby object that wraps around Rust data.
///
/// See the documentation for `Rosy` for more information.
#[repr(transparent)]
pub struct RosyObject<R> {
    inner: NonNullObject,
    _marker: PhantomData<R>,
}

#[cfg(test)]
mod assertions {
    use std::mem::align_of;
    use static_assertions::*;
    use super::*;

    #[repr(align(512))]
    struct AbsoluteUnit(u128, u128, u128, u128);

    type AbsoluteObject = RosyObject<AbsoluteUnit>;

    assert_eq_size!(size; AbsoluteObject, AnyObject);
    const_assert_eq!(align;
        align_of::<AbsoluteObject>(),
        align_of::<ruby::VALUE>(),
        align_of::<AnyObject>(),
    );
}

impl<R> Clone for RosyObject<R> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<R> Copy for RosyObject<R> {}

impl<R: Rosy> AsRef<AnyObject> for RosyObject<R> {
    #[inline]
    fn as_ref(&self) -> &AnyObject {
        self.inner.as_ref()
    }
}

impl<R: Rosy> From<RosyObject<R>> for AnyObject {
    #[inline]
    fn from(obj: RosyObject<R>) -> Self {
        obj.inner.into()
    }
}

impl<R: Rosy> PartialEq<AnyObject> for RosyObject<R> {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl<R: Rosy> Object for RosyObject<R> {
    #[inline]
    fn unique_id() -> Option<u128> {
        R::unique_object_id()
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        R::cast(obj)
    }

    #[inline]
    fn class(self) -> Class<Self> {
        unsafe { Class::from_raw((*self.r_typed_data()).basic.klass) }
    }
}

impl<R: Rosy> From<Box<R>> for RosyObject<R> {
    #[inline]
    fn from(rosy: Box<R>) -> Self {
        let rosy = Box::into_raw(rosy) as *mut c_void;
        let ty = RosyObject::<R>::data_type();
        let class = R::class().raw();
        unsafe {
            Self::from_raw(ruby::rb_data_typed_object_wrap(class, rosy, ty))
        }
    }
}

impl<R: Rosy> From<R> for RosyObject<R> {
    #[inline]
    fn from(rosy: R) -> Self {
        Box::new(rosy).into()
    }
}

impl<R: Rosy> fmt::Debug for RosyObject<R> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<R: Rosy> fmt::Display for RosyObject<R> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<R: Rosy> RosyObject<R> {
    #[inline]
    pub(crate) fn data_type() -> &'static rb_data_type_t {
        unsafe extern "C" fn dmark<R: Rosy>(rosy: *mut c_void) {
            (&mut *(rosy as *mut R)).mark();
        }
        unsafe extern "C" fn dfree<R: Rosy>(rosy: *mut c_void) {
            Box::from_raw(rosy as *mut R).free();
        }
        unsafe extern "C" fn dsize<R: Rosy>(rosy: *const c_void) -> usize {
            (&*(rosy as *const R)).size()
        }
        &rb_data_type_t {
            wrap_struct_name: R::ID,
            function: rb_data_type_t_function {
                dmark: Some(dmark::<R>),
                dfree: Some(dfree::<R>),
                dsize: Some(dsize::<R>),
                reserved: [ptr::null_mut(); 2],
            },
            parent: ptr::null(),
            data: ptr::null_mut(),
            flags: ruby::RUBY_TYPED_FREE_IMMEDIATELY,
        }
    }

    #[inline]
    fn r_typed_data(self) -> *mut ruby::RTypedData {
        self.raw() as *mut ruby::RTypedData
    }

    #[inline]
    fn data(self) -> *mut R {
        unsafe { (*self.r_typed_data()).data as *mut R }
    }

    /// Returns a reference to the inner `Rosy` value.
    #[inline]
    pub fn as_data(&self) -> &R {
        unsafe { &*self.data() }
    }
}
