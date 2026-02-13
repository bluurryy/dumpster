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
    ($name:literal $krate:ident::$module:ident collect=$collect:tt) => {
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

fn main() {
    const N_ITERS: usize = 1_000_000;

    for _ in 0..100 {
        bench!("sync: std" dumpster::sync collect=true);
        bench!("sync: foldhash/quality" dumpster_foldhash_quality::sync collect=true);
        bench!("sync: foldhash/quality/fixed" dumpster_foldhash_quality_fixed::sync collect=true);
        bench!("sync: foldhash/fast" dumpster_foldhash_fast::sync collect=true);
        bench!("sync: foldhash/fast/fixed" dumpster_foldhash_fast_fixed::sync collect=true);
        bench!("sync/manual: std" dumpster::sync collect=false);
        bench!("sync/manual: foldhash/quality" dumpster_foldhash_quality::sync collect=false);
        bench!("sync/manual: foldhash/quality/fixed" dumpster_foldhash_quality_fixed::sync collect=false);
        bench!("sync/manual: foldhash/fast" dumpster_foldhash_fast::sync collect=false);
        bench!("sync/manual: foldhash/fast/fixed" dumpster_foldhash_fast_fixed::sync collect=false);
        bench!("unsync: std" dumpster::unsync collect=true);
        bench!("unsync: foldhash/quality" dumpster_foldhash_quality::unsync collect=true);
        bench!("unsync: foldhash/quality/fixed" dumpster_foldhash_quality_fixed::unsync collect=true);
        bench!("unsync: foldhash/fast" dumpster_foldhash_fast::unsync collect=true);
        bench!("unsync: foldhash/fast/fixed" dumpster_foldhash_fast_fixed::unsync collect=true);
        bench!("unsync/manual: std" dumpster::unsync collect=false);
        bench!("unsync/manual: foldhash/quality" dumpster_foldhash_quality::unsync collect=false);
        bench!("unsync/manual: foldhash/quality/fixed" dumpster_foldhash_quality_fixed::unsync collect=false);
        bench!("unsync/manual: foldhash/fast" dumpster_foldhash_fast::unsync collect=false);
        bench!("unsync/manual: foldhash/fast/fixed" dumpster_foldhash_fast_fixed::unsync collect=false);
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
