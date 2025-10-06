/*
    dumpster, acycle-tracking garbage collector for Rust.    Copyright (C) 2023 Clayton Ramsey.

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Optional garbage collection pointers.

use std::fmt;
#[cfg(feature = "coerce-unsized")]
use std::marker::Unsize;

use crate::{sync, unsync, Trace};

/// Contains the [`Sealed`](gc_ptr::Sealed) trait for [`GcPtr`].
pub(crate) mod gc_ptr {
    #[cfg(feature = "coerce-unsized")]
    use std::marker::Unsize;

    use crate::Trace;

    /// Contains all the functionality for [`GcPtr`](crate::GcPtr).
    pub trait Sealed {
        #[doc(hidden)]
        type T: Trace + ?Sized + 'static;

        #[doc(hidden)]
        fn dead() -> Self
        where
            Self::T: Sized;

        #[doc(hidden)]
        #[cfg(feature = "coerce-unsized")]
        fn dead_with_metadata_of<U>() -> Self
        where
            U: Unsize<Self::T>;

        #[doc(hidden)]
        fn is_dead(&self) -> bool;
    }
}

/// A common trait of [`sync::Gc`] and [`unsync::Gc`], used for [`Opt`].
pub trait GcPtr: gc_ptr::Sealed {}
impl<T: Trace + ?Sized + 'static> GcPtr for unsync::Gc<T> {}
impl<T: Trace + Send + Sync + ?Sized + 'static> GcPtr for sync::Gc<T> {}

/// An optional `Gc` as an alternative for `Option<Gc<T>>`.
///
/// It has the advantage that it does not take up more space than `Gc`.
///
/// Instead of importing this type directly, prefer the aliases
/// [`sync::OptGc`] and [`unsync::OptGc`].
#[derive(Clone)]
pub struct Opt<Gc: GcPtr>(Gc);

impl<Gc: GcPtr + fmt::Debug> fmt::Debug for Opt<Gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_ref(), f)
    }
}

impl<Gc: GcPtr> Opt<Gc> {
    /// No value.
    #[inline]
    #[must_use]
    pub fn none() -> Self
    where
        Gc::T: Sized,
    {
        Self(Gc::dead())
    }

    /// Create an `Opt<Gc>` from a `Gc`.
    ///
    /// # Examples
    /// ```
    /// # use dumpster::unsync::{ Gc, OptGc };
    /// let gc = Gc::new(7);
    /// let opt_gc = OptGc::some(gc);
    ///
    /// assert_eq!(**opt_gc.as_ref().unwrap(), 7);
    /// ```
    #[inline]
    #[must_use]
    pub fn some(gc: Gc) -> Self {
        Self(gc)
    }

    /// Create a `Opt<Gc>` representing no value with the metadata
    /// of the given type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dumpster::unsync::OptGc;
    /// let gc: OptGc<[i32]> = OptGc::none_with_metadata_of::<[i32; 0]>();
    /// assert!(gc.is_none());
    /// ```
    #[inline]
    #[must_use]
    #[cfg(feature = "coerce-unsized")]
    pub fn none_with_metadata_of<U>() -> Self
    where
        U: Unsize<Gc::T>,
    {
        Self(Gc::dead_with_metadata_of::<U>())
    }

    /// Converts from `&Opt<Gc>` to `Option<&Gc>`.
    ///
    /// # Examples
    ///
    /// Calculates the length of an `OptGc<str>` without moving
    /// the `Gc<str>`.
    ///
    /// ```
    /// use dumpster::unsync::{Gc, OptGc};
    ///
    /// let text: OptGc<str> = OptGc::some(Gc::from("Hello, world!"));
    /// // First, convert `&Opt<Gc<str>>` to `Option<&Gc<str>>` with `as_ref`,
    /// // then consume *that* with `map`, leaving `text` on the stack.
    /// let text_length: Option<usize> = text.as_ref().map(|s| s.len());
    /// # assert_eq!(text_length, Some(13));
    /// println!("still can print text: {text:?}");
    /// ```
    #[inline]
    pub fn as_ref(&self) -> Option<&Gc> {
        if self.0.is_dead() {
            return None;
        }

        Some(&self.0)
    }

    /// Converts from `&mut Opt<Gc>` to `Option<&mut Gc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dumpster::unsync::{Gc, OptGc};
    ///
    /// let mut x: OptGc<i32> = OptGc::some(Gc::new(2));
    ///
    /// match x.as_mut() {
    ///     Some(v) => *v = Gc::new(42),
    ///     None => {}
    /// }
    ///
    /// assert_eq!(**x.as_ref().unwrap(), 42);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut Gc> {
        if self.0.is_dead() {
            return None;
        }

        Some(&mut self.0)
    }

    /// Convert this `Opt<Gc>` into an `Option<Gc>`.
    #[inline]
    pub fn into_option(self) -> Option<Gc> {
        if self.0.is_dead() {
            return None;
        }

        Some(self.0)
    }
}
