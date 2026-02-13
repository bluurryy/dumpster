/*
    dumpster, a cycle-tracking garbage collector for Rust.
    Copyright (C) 2023 Clayton Ramsey.

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::sync::Mutex;

/// A garbage-collected structure which points to an arbitrary number of other garbage-collected
/// structures.
///
/// Cloning a `Multiref` yields a duplicated pointer, not a deep copy.
pub trait Multiref: Clone {
    /// Create a new multiref which points to some data.
    fn new(points_to: Vec<Self>) -> Self;
    /// Apply some function to the backing set of references owned by this structure.
    fn apply(&self, f: impl FnOnce(&mut Vec<Self>));
    /// Collect all the floating GCs out there.
    fn collect();
}

/// A trait for thread-safe synchronized multirefs.
pub trait SyncMultiref: Send + Sync + Multiref {}

impl<T> SyncMultiref for T where T: Send + Sync + Multiref {}

macro_rules! dumpster_multiref_impl {
    ($krate:ident) => {
        pub mod $krate {
            dumpster_multiref_impl!($krate::unsync);
            dumpster_multiref_impl!($krate::sync);
        }
    };
    ($krate:ident::$module:ident) => {
        pub mod $module {
            use crate::*;

            #[derive(::$krate::Trace)]
            #[dumpster(crate = ::$krate)]
            pub struct MultirefImpl {
                refs: Mutex<Vec<::$krate::$module::Gc<Self>>>,
            }

            impl Multiref for ::$krate::$module::Gc<MultirefImpl> {
                fn new(points_to: Vec<Self>) -> Self {
                    ::$krate::$module::Gc::new(MultirefImpl {
                        refs: Mutex::new(points_to),
                    })
                }

                fn apply(&self, f: impl FnOnce(&mut Vec<Self>)) {
                    f(self.refs.lock().unwrap().as_mut());
                }

                fn collect() {
                    ::$krate::$module::collect()
                }
            }
        }
    };
}

dumpster_multiref_impl!(dumpster);
dumpster_multiref_impl!(dumpster_foldhash_fast);
dumpster_multiref_impl!(dumpster_foldhash_fast_fixed);
dumpster_multiref_impl!(dumpster_foldhash_quality);
dumpster_multiref_impl!(dumpster_foldhash_quality_fixed);
