/*
    dumpster, a cycle-tracking garbage collector for Rust.    Copyright (C) 2023 Clayton Ramsey.

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Contains macro to create optional garbage collection pointers.

/// Macro to create optional garbage collection pointers.
macro_rules! make_opt_gc {
    ($module:ident, $visit:ident; $($($bounds:tt)+)?) => {
        /// An alternative to <code>[Option]\<[Gc]\<T\>\></code> that takes up less space.
        ///
        /// Specifically `OptGc<T>` always has the same size as `Gc<T>`.
        ///
        /// # Interaction with `Drop`
        ///
        /// This is implemented by interpreting a dead `Gc` as none.
        /// So during a `Drop` implementation this type can turn into none.
        pub struct OptGc<T: Trace $(+ $($bounds)*)? + ?Sized + 'static>(Gc<T>);

        impl<T: Trace $(+ $($bounds)*)? + 'static> OptGc<T> {
            /// An `OptGc<T>` representing no value.
            ///
            /// This is only available for `Sized` values.
            /// To create an unsized none value you can use `NONE` with `coerce_opt_gc`.
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{OptGc, coerce_opt_gc};")]
            ///
            /// let gc: OptGc<[i32]> = coerce_opt_gc!(OptGc::<[i32; 0]>::NONE);
            /// assert!(gc.is_none());
            /// ```
            #[expect(clippy::declare_interior_mutable_const)]
            pub const NONE: Self = Self(Gc::<T>::DEAD);
        }

        impl<T: Trace $(+ $($bounds)*)? + ?Sized + 'static> OptGc<T> {
            /// Returns `true` if the option is some value of `T`.
            #[inline]
            pub fn is_some(&self) -> bool {
                !self.0.is_dead()
            }

            /// Returns `true` if the option is no value.
            #[inline]
            pub fn is_none(&self) -> bool {
                self.0.is_dead()
            }

            /// Create an `OptGc<T>` from a `Gc<T>`.
            ///
            /// # Examples
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
            ///
            /// let gc = Gc::new(7);
            /// let opt_gc = OptGc::some(gc);
            ///
            /// assert_eq!(**opt_gc.as_ref().unwrap(), 7);
            /// ```
            #[inline]
            #[must_use]
            pub fn some(gc: Gc<T>) -> Self {
                Self(gc)
            }

            /// Converts from `&OptGc<T>` to `Option<&Gc<T>>`.
            ///
            /// # Examples
            ///
            /// Calculates the length of an `OptGc<str>` without moving
            /// the `Gc<str>`.
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
            ///
            /// let text: OptGc<str> = OptGc::some(Gc::from("Hello, world!"));
            /// // First, convert `&OptGc<str>` to `Option<&Gc<str>>` with `as_ref`,
            /// // then consume *that* with `map`, leaving `text` on the stack.
            /// let text_length: Option<usize> = text.as_ref().map(|s| s.len());
            /// # assert_eq!(text_length, Some(13));
            /// println!("still can print text: {text:?}");
            /// ```
            #[inline]
            pub fn as_ref(&self) -> Option<&Gc<T>> {
                if self.0.is_dead() {
                    return None;
                }

                Some(&self.0)
            }

            /// Converts from `&mut OptGc<T>` to `Option<&mut Gc<T>>`.
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
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
            pub fn as_mut(&mut self) -> Option<&mut Gc<T>> {
                if self.0.is_dead() {
                    return None;
                }

                Some(&mut self.0)
            }

            /// Converts from `&OptGc<T>` to `Option<&T>`.
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
            ///
            /// let text: OptGc<str> = OptGc::some(Gc::from("Hello, world!"));
            /// let str: Option<&str> = text.as_deref();
            /// assert_eq!(str, Some("Hello, world!"));
            /// ```
            #[inline]
            pub fn as_deref(&self) -> Option<&T> {
                if self.0.is_dead() {
                    return None;
                }

                Some(&self.0)
            }

            /// Convert this `OptGc<T>` into an `Option<Gc<T>>`.
            #[inline]
            pub fn into_option(self) -> Option<Gc<T>> {
                if self.0.is_dead() {
                    return None;
                }

                Some(self.0)
            }

            /// Convert this `OptGc<T>` into a `Gc<T>`.
            ///
            /// If `self` is none then the returned `Gc<T>` will be dead.
            #[inline]
            pub fn into_maybe_dead_gc(self) -> Gc<T> {
                self.0
            }

            /// Convert a `Gc<T>` into an `OptGc<T>`.
            ///
            /// If `gc` is dead then the returned `OptGc<T>` will be none.
            #[inline]
            pub fn from_maybe_dead_gc(gc: Gc<T>) -> Self {
                Self(gc)
            }

            /// Determine whether two `OptGc`s are equivalent by reference.
            /// Returns `true` if both `this` and `other` point to the same value, in the same style as
            /// [`std::ptr::eq`].
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
            ///
            /// let gc1 = OptGc::some(Gc::new(0));
            /// let gc2 = gc1.clone(); // points to same spot as `gc1`
            /// let gc3 = OptGc::some(Gc::new(0)); // same value, but points to a different object than `gc1`
            ///
            /// assert!(gc1.ptr_eq(&gc2));
            /// assert!(!gc1.ptr_eq(&gc3));
            ///
            /// let gc4 = OptGc::<i32>::NONE; // no value
            /// let gc5 = OptGc::<i32>::NONE; // also no value
            /// assert!(gc4.ptr_eq(&gc5));
            /// ```
            #[inline]
            pub fn ptr_eq(&self, other: &Self) -> bool {
                Gc::ptr_eq(&self.0, &other.0)
            }

            /// Get the number of references to the value pointed to by this `OptGc`.
            ///
            /// This does not include internal references generated by the garbage collector.
            ///
            /// This function returns `0` if the `Gc` whose reference count we are loading is "dead" (i.e.
            /// generated through a `Drop` implementation). For further reference, take a look at
            /// [`Gc::is_dead`].
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use dumpster::", stringify!($module), "::{Gc, OptGc};")]
            ///
            /// let gc = OptGc::some(Gc::new(()));
            /// assert_eq!(gc.ref_count(), 1);
            /// let gc2 = gc.clone();
            /// assert_eq!(gc.ref_count(), 2);
            /// drop(gc);
            /// drop(gc2);
            ///
            /// let gc = OptGc::<i32>::NONE;
            /// assert_eq!(gc.ref_count(), 0);
            /// ```
            pub fn ref_count(&self) -> usize {
                match self.as_ref() {
                    Some(gc) => Gc::ref_count(gc).get(),
                    None => 0,
                }
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + ?Sized + 'static + fmt::Debug> fmt::Debug for OptGc<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.as_ref(), f)
            }
        }

        unsafe impl<V: Visitor, T: Trace $(+ $($bounds)*)? + ?Sized> TraceWith<V> for OptGc<T> {
            fn accept(&self, visitor: &mut V) -> Result<(), ()> {
                if let Some(gc) = self.as_ref() {
                    visitor.$visit(gc);
                }

                Ok(())
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static> Clone for OptGc<T> {
            fn clone(&self) -> Self {
                Gc::try_clone(&self.0).into()
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static> Default for OptGc<T> {
            fn default() -> Self {
                Self::NONE
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static> From<Option<Gc<T>>> for OptGc<T> {
            fn from(value: Option<Gc<T>>) -> Self {
                match value {
                    Some(gc) => Self(gc),
                    None => Self::NONE,
                }
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static> From<OptGc<T>> for Option<Gc<T>> {
            fn from(value: OptGc<T>) -> Self {
                value.into_option()
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static + PartialEq> PartialEq for OptGc<T> {
            fn eq(&self, other: &OptGc<T>) -> bool {
                self.as_ref() == other.as_ref()
            }
        }

        impl<T: Trace $(+ $($bounds)*)? + 'static + Eq> Eq for OptGc<T> {}
    };
}

pub(crate) use make_opt_gc;
