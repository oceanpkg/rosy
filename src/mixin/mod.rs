//! Ruby mixins.

use crate::{
    prelude::*,
    ruby::{self, ID, VALUE},
    util::Sealed,
    vm::EvalArgs,
};

mod class;
mod method;
mod module;
pub use self::{class::*, method::*, module::*};

#[inline]
fn _get_const(m: impl Mixin, name: SymbolId) -> Option<AnyObject> {
    unsafe {
        if ruby::rb_const_defined(m.raw(), name.raw()) != 0 {
            Some(_get_const_unchecked(m, name))
        } else {
            None
        }
    }
}

#[inline]
unsafe fn _get_const_unchecked(m: impl Mixin, name: impl Into<SymbolId>) -> AnyObject {
    AnyObject::from_raw(ruby::rb_const_get(m.raw(), name.into().raw()))
}

// monomorphization
unsafe fn _set_attr(m: VALUE, name: ID, read: bool, write: bool) -> Result {
    crate::protected_no_panic(|| _set_attr_unchecked(m, name, read, write))
}

#[inline]
unsafe fn _set_attr_unchecked(m: VALUE, name: ID, read: bool, write: bool) {
    ruby::rb_attr(m, name, read as _, write as _, 0);
}

// monomorphization
unsafe fn _set_class_var(m: VALUE, key: ID, val: VALUE) -> Result {
    crate::protected_no_panic(|| _set_class_var_unchecked(m, key, val))
}

#[inline]
unsafe fn _set_class_var_unchecked(m: VALUE, key: ID, val: VALUE) {
    ruby::rb_cvar_set(m, key, val)
}

/// A type that supports mixins (see [`Class`](struct.Class.html) and
/// [`Module`](struct.Module.html)).
pub trait Mixin: Object + Sealed {
    /// Returns `self` as a `Class` if it is one or a `Module` otherwise.
    fn to_class(self) -> Result<Class, Module>;

    /// Returns `self` as a `Module` if it is one or a `Class` otherwise.
    fn to_module(self) -> Result<Module, Class>;

    /// Embeds the contents of `module` in `self`.
    #[inline]
    fn include(self, module: Module) {
        unsafe { ruby::rb_include_module(self.raw(), module.raw()) };
    }

    /// Returns whether `self` or one of its ancestors includes `module`.
    ///
    /// This is equivalent to the `include?` method.
    #[inline]
    #[must_use]
    fn includes(self, module: Module) -> bool {
        unsafe { ruby::rb_mod_include_p(self.raw(), module.raw()) != 0 }
    }

    /// Returns an array of the modules included in `self`.
    #[inline]
    fn included_modules(self) -> Array<Module> {
        unsafe { Array::from_raw(ruby::rb_mod_included_modules(self.raw())) }
    }

    /// Prepends `module` in `self`.
    #[inline]
    fn prepend(self, module: Module) {
        unsafe { ruby::rb_prepend_module(self.raw(), module.raw()) };
    }

    /// Defines a new class under `self` with `name`.
    #[inline]
    fn def_class(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        Class::_def_under(self, Class::object(), name.into())
    }

    /// Defines a new subclass of `superclass` under `self` with `name`.
    #[inline]
    fn def_subclass<S: Object>(
        self,
        superclass: Class<S>,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        Class::_def_under(self, superclass.into_any_class(), name.into())
    }

    /// Returns the existing `Class` with `name` in `self`.
    #[inline]
    fn get_class(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Class> {
        _get_const(self, name.into())?.to_class()
    }

    /// Returns the existing `Class` with `name` in `self`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Class`
    #[inline]
    unsafe fn get_class_unchecked(
        self,
        name: impl Into<SymbolId>,
    ) -> Class {
        Class::cast_unchecked(_get_const_unchecked(self, name))
    }

    /// Defines a new module under `self` with `name`.
    #[inline]
    fn def_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Module, DefMixinError> {
        Module::_def_under(self, name.into())
    }

    /// Returns the existing `Module` with `name` in `self`.
    #[inline]
    fn get_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Module> {
        _get_const(self, name.into())?.to_module()
    }

    /// Returns the existing `Module` with `name` in `self`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Module`
    #[inline]
    unsafe fn get_module_unchecked(
        self,
        name: impl Into<SymbolId>,
    ) -> Module {
        Module::cast_unchecked(_get_const_unchecked(self, name))
    }

    /// Returns whether a constant for `name` is defined in `self`, or in some
    /// parent class if not `self`.
    #[inline]
    fn has_const(self, name: impl Into<SymbolId>) -> bool {
        unsafe { ruby::rb_const_defined(self.raw(), name.into().raw()) != 0 }
    }

    /// Returns the constant value for `name` in `self`, or in some parent class
    /// if not `self`.
    ///
    /// # Exception Handling
    ///
    /// If `name` is an uninitialized variable, a `NameError` exception will be
    /// raised. If you're unsure whether `name` exists, either check
    /// [`has_const`](#method.has_const) or surround a call to this method in a
    /// `protected` closure.
    #[inline]
    fn get_const(self, name: impl Into<SymbolId>) -> AnyObject {
        let name = name.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_const_get(self.raw(), name)) }
    }

    /// Sets the value a constant for `name` in `self` to `val`.
    #[inline]
    fn set_const(self, name: impl Into<SymbolId>, val: impl Into<AnyObject>) {
        let val = val.into().raw();
        unsafe { ruby::rb_const_set(self.raw(), name.into().raw(), val) };
    }

    /// Removes the constant value for `name`, returning it.
    ///
    /// # Exception Handling
    ///
    /// If the constant for `name` cannot be removed, an exception is raised.
    #[inline]
    fn remove_const(self, name: impl Into<SymbolId>) -> AnyObject {
        let name = name.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_const_remove(self.raw(), name)) }
    }

    /// Returns whether the class-level `var` is defined in `self`.
    #[inline]
    fn has_class_var(self, var: impl Into<SymbolId>) -> bool {
        unsafe { ruby::rb_cvar_defined(self.raw(), var.into().raw()) != 0 }
    }

    /// Returns the class-level `var` in `self`.
    ///
    /// # Exception Handling
    ///
    /// If `var` is an uninitialized variable, a `NameError` exception will be
    /// raised. If you're unsure whether `var` exists, either check
    /// [`has_class_var`](#method.has_class_var) or surround a call to this
    /// method in a `protected` closure.
    ///
    /// ```
    /// use rosy::{Class, Object, Mixin, protected};
    /// # rosy::vm::init().unwrap();
    ///
    /// let class = Class::array();
    /// let error = protected(|| class.get_class_var("@@hello")).unwrap_err();
    ///
    /// assert!(error.is_name_error());
    /// ```
    #[inline]
    fn get_class_var(self, var: impl Into<SymbolId>) -> AnyObject {
        let var = var.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_cvar_get(self.raw(), var)) }
    }

    /// Sets the class-level `var` in `self` to `val`.
    #[inline]
    fn set_class_var<K, V>(self, key: K, val: V) -> Result
    where
        K: Into<SymbolId>,
        V: Into<AnyObject>,
    {
        let key = key.into().raw();
        let val = val.into().raw();
        unsafe { _set_class_var(self.raw(), key, val) }
    }

    /// Sets the class-level var for `key` in `self` to `val`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    unsafe fn set_class_var_unchecked<K, V>(self, key: K, val: V)
    where
        K: Into<SymbolId>,
        V: Into<AnyObject>,
    {
        _set_class_var_unchecked(self.raw(), key.into().raw(), val.into().raw())
    }

    /// Defines an read-only attribute on `self` with `name`.
    #[inline]
    fn def_attr_reader<N: Into<SymbolId>>(self, name: N) -> Result {
        unsafe { _set_attr(self.raw(), name.into().raw(), true, false) }
    }

    /// Defines an read-only attribute on `self` with `name`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    unsafe fn def_attr_reader_unchecked<N: Into<SymbolId>>(self, name: N) {
        _set_attr_unchecked(self.raw(), name.into().raw(), true, false);
    }

    /// Defines a write-only attribute on `self` with `name`.
    #[inline]
    fn def_attr_writer<N: Into<SymbolId>>(self, name: N) -> Result {
        unsafe { _set_attr(self.raw(), name.into().raw(), false, true) }
    }

    /// Defines a write-only attribute on `self` with `name`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    unsafe fn def_attr_writer_unchecked<N: Into<SymbolId>>(self, name: N) {
        _set_attr_unchecked(self.raw(), name.into().raw(), false, true);
    }

    /// Defines a read-write attribute on `self` with `name`.
    #[inline]
    fn def_attr_accessor<N: Into<SymbolId>>(self, name: N) -> Result {
        unsafe { _set_attr(self.raw(), name.into().raw(), true, true) }
    }

    /// Defines a read-write attribute on `self` with `name`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    unsafe fn def_attr_accessor_unchecked<N: Into<SymbolId>>(self, name: N) {
        _set_attr_unchecked(self.raw(), name.into().raw(), true, true);
    }

    /// Evaluates `args` in the context of `self`.
    ///
    /// See the docs for `EvalArgs` for more info.
    ///
    /// # Safety
    ///
    /// Code executed from `args` may void the type safety of objects accessible
    /// from Rust. For example, if one calls `push` on an `Array<A>` with an
    /// object of type `B`, then the inserted object will be treated as being of
    /// type `A`.
    ///
    /// An exception may be raised by the code or by `args` being invalid.
    #[inline]
    unsafe fn eval(self, args: impl EvalArgs) -> AnyObject {
        args.eval_in_mixin(self)
    }

    /// Evaluates `args` in the context of `self`, returning any raised
    /// exceptions.
    ///
    /// See the docs for `EvalArgs` for more info.
    ///
    /// # Safety
    ///
    /// Code executed from `args` may void the type safety of objects accessible
    /// from Rust. For example, if one calls `push` on an `Array<A>` with an
    /// object of type `B`, then the inserted object will be treated as being of
    /// type `A`.
    #[inline]
    unsafe fn eval_protected(self, args: impl EvalArgs) -> Result<AnyObject> {
        args.eval_in_mixin_protected(self)
    }
}

impl Mixin for Class {
    #[inline]
    fn to_class(self) -> Result<Class, Module> {
        Ok(self)
    }

    #[inline]
    fn to_module(self) -> Result<Module, Class> {
        Err(self)
    }
}

impl Mixin for Module {
    #[inline]
    fn to_class(self) -> Result<Class, Module> {
        Err(self)
    }

    #[inline]
    fn to_module(self) -> Result<Module, Class> {
        Ok(self)
    }
}

/// An error when attempting to define a [`Mixin`](trait.Mixin.html) type.
#[derive(Debug)]
pub enum DefMixinError {
    /// A class already exists with the same name in the same namespace.
    ExistingClass(Class),
    /// A module already exists with the same name in the same namespace.
    ExistingModule(Module),
    /// Some other constant already exists.
    ExistingConst(AnyObject),
    /// The given class is frozen and can't have items defined under it.
    FrozenClass(Class),
    /// The given module is frozen and can't have items defined under it.
    FrozenModule(Module),
}

impl DefMixinError {
    #[cold]
    #[inline]
    pub(crate) fn _frozen(m: impl Mixin) -> Self {
        match m.to_class() {
            Ok(class) => DefMixinError::FrozenClass(class),
            Err(module) => DefMixinError::FrozenModule(module),
        }
    }

    #[inline]
    fn _get(m: impl Mixin, name: SymbolId) -> Option<Self> {
        use crate::object::Ty;
        use DefMixinError::*;

        let existing = _get_const(m, name)?;
        let raw = existing.raw();
        let err = match crate::util::value_built_in_ty(raw) {
            Some(Ty::MODULE) => unsafe {
                ExistingModule(Module::from_raw(raw))
            },
            Some(Ty::CLASS) => unsafe {
                ExistingClass(Class::from_raw(raw))
            },
            Some(_) | None => ExistingConst(existing),
        };
        Some(err)
    }

    /// Returns the existing class that was found.
    #[inline]
    pub fn existing_class(&self) -> Option<Class> {
        match *self {
            DefMixinError::ExistingClass(c) => Some(c),
            _ => None,
        }
    }

    /// Returns the existing module that was found.
    #[inline]
    pub fn existing_module(&self) -> Option<Module> {
        match *self {
            DefMixinError::ExistingModule(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the existing constant that was found.
    #[inline]
    pub fn existing_const(&self) -> Option<AnyObject> {
        match *self {
            DefMixinError::ExistingConst(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the existing object that was found.
    #[inline]
    pub fn existing_object(&self) -> Option<AnyObject> {
        use DefMixinError::*;
        match *self {
            ExistingModule(m) => Some(m.into_any_object()),
            ExistingClass(c)  => Some(c.into_any_object()),
            ExistingConst(c)  => Some(c),
            _ => None,
        }
    }
}
