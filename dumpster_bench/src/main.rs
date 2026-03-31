/*
    dumpster, a cycle-tracking garbage collector for Rust.
    Copyright (C) 2023 Clayton Ramsey.

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Benchmarks for the `dumpster` garbage collection library.

use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use dumpster_bench::Multiref;

struct BenchmarkData {
    name: &'static str,
    test: &'static str,
    n_threads: usize,
    n_ops: usize,
    duration: Duration,
}

impl Display for BenchmarkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.name,
            self.test,
            self.n_threads,
            self.n_ops,
            self.duration.as_micros()
        )
    }
}

macro_rules! condition {
    (if true { $($then:tt)* } else { $($else:tt)* }) => { $($then)* };
    (if false { $($then:tt)* } else { $($else:tt)* }) => { $($else)* };
}

macro_rules! bench {
    ($name:expr, $krate:ident::$module:ident collect=$collect:tt) => {
        $krate::$module::set_collect_condition(condition! {
            if $collect {
                $krate::$module::default_collect_condition
            } else {
                |_| false
            }
        });

        println!(
            "{}",
            single_threaded::<$krate::$module::Gc<dumpster_bench::$krate::$module::MultirefImpl>>(
                $name, N_ITERS,
            )
        );
    };
}

macro_rules! benches {
    ($name:literal, $krate:ident) => {
        bench!(concat!("sync: ", $name), $krate::sync collect=true);
        bench!(concat!("sync/manual: ", $name), $krate::sync collect=false);
        bench!(concat!("unsync: ", $name), $krate::unsync collect=true);
        bench!(concat!("unsync/manual: ", $name), $krate::unsync collect=false);
    };
}

fn main() {
    const N_ITERS: usize = 1_000_000;

    for i in 0..200 {
        eprintln!("{i}");

        benches!("main", dumpster_main);
        benches!("fuel-64", dumpster_contains_gcs_trait);
        benches!("remove", dumpster_remove_contains_gcs);
    }
}

/// Run a benchmark of a multi-threaded garbage collector.
fn single_threaded<M: Multiref>(name: &'static str, n_iters: usize) -> BenchmarkData {
    fastrand::seed(12345);
    let mut gcs = (0..50).map(|_| M::new(Vec::new())).collect::<Vec<_>>();

    // println!("{name}: running...");
    let tic = Instant::now();
    for _n in 0..n_iters {
        // println!("iter {_n}");
        if gcs.is_empty() {
            gcs.push(M::new(Vec::new()));
        } else {
            match fastrand::u8(0..4) {
                0 => {
                    // println!("create allocation");
                    // create new allocation
                    gcs.push(M::new(Vec::new()));
                }
                1 => {
                    // println!("add reference");
                    // add a reference
                    if gcs.len() > 1 {
                        let from = fastrand::usize(0..gcs.len());
                        let to = fastrand::usize(0..gcs.len());
                        let new_gc = gcs[to].clone();
                        gcs[from].apply(|v| v.push(new_gc));
                    }
                }
                2 => {
                    // println!("remove gc");
                    // destroy a reference owned by the vector
                    gcs.swap_remove(fastrand::usize(0..gcs.len()));
                }
                3 => {
                    // println!("remove reference");
                    // destroy a reference owned by some gc
                    let from = fastrand::usize(0..gcs.len());
                    gcs[from].apply(|v| {
                        if !v.is_empty() {
                            let to = fastrand::usize(0..v.len());
                            v.swap_remove(to);
                        }
                    })
                }
                _ => unreachable!(),
            }
        }
    }
    drop(gcs);
    M::collect();
    let toc = Instant::now();
    // println!("finished {name} in {:?}", (toc - tic));
    BenchmarkData {
        name,
        test: "single_threaded",
        n_threads: 1,
        n_ops: n_iters,
        duration: toc.duration_since(tic),
    }
}
