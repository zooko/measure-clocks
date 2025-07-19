#![feature(rustc_private)]

use std::time::Instant;
use rustc_hash::FxHashMap;
extern crate libc;

const NUM_SAMPLES: u128 = 10_000_000;

#[inline(never)]
pub fn dummy_func() -> i64 {
    // When I make this code a little faster/simpler then cputime on Macos starts telling me
    // that it took 0 nanoseconds. 🤔
    let mut a = Arc::new(0);
    for i in 0..30 {
        for j in 0..29 {
            *Arc::make_mut(&mut a) ^= black_box(i * j);
        }
    }

    *a
}

use std::sync::Arc;
use std::hint::black_box;
fn instant_secs() -> Vec<u128> {
    let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);

    let mut i = 0;
    while i < NUM_SAMPLES {
        let inst = Instant::now();

        black_box(dummy_func());

        let d = inst.elapsed();
        assert!(d.as_nanos() > 0);
        durations.push(d.as_nanos());

        i += 1;
    }

    durations
}

use std::mem::MaybeUninit;

#[cfg(target_vendor = "apple")]
pub mod plat_apple {
    use std::hint::black_box;
    use crate::{ClockType, NUM_SAMPLES, dummy_func, libc_gettime_clock, MaybeUninit};
    use libc::clockid_t;
    unsafe extern "C" {
        fn clock_gettime_nsec_np(clk_id: clockid_t) -> u64;
    }

    pub fn gettime_nsec_np_clock(clock: ClockType) -> Vec<u128> {
        let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        while i < NUM_SAMPLES {
            let prev = unsafe { clock_gettime_nsec_np(clock) };

            black_box(dummy_func());

            let now = unsafe { clock_gettime_nsec_np(clock) };

            assert!(now > prev);
            let dur: u128 = u128::from(now - prev);

            durations.push(dur);

            i += 1;
        }

        durations
    }

    pub fn uptime_raw_secs() -> Vec<u128> {
        libc_gettime_clock(libc::CLOCK_UPTIME_RAW)
    }
    
    pub fn nsec_np_uptime_raw_secs() -> Vec<u128> {
        gettime_nsec_np_clock(libc::CLOCK_UPTIME_RAW)
    }

    pub fn nsec_np_cputime_secs() -> Vec<u128> {
        gettime_nsec_np_clock(libc::CLOCK_PROCESS_CPUTIME_ID)
    }

    pub fn nsec_np_monotonic_secs() -> Vec<u128> {
        gettime_nsec_np_clock(libc::CLOCK_MONOTONIC)
    }

    pub fn nsec_np_monotonic_raw_secs() -> Vec<u128> {
        gettime_nsec_np_clock(libc::CLOCK_MONOTONIC_RAW)
    }

    use mach_sys::mach_time::{mach_absolute_time, mach_timebase_info};
    use mach_sys::kern_return::KERN_SUCCESS;
    pub fn mach_absolute_time_secs() -> Vec<u128> {
        let mut mtt1: MaybeUninit<mach_timebase_info> = MaybeUninit::uninit();
        let retval = unsafe { mach_timebase_info(mtt1.as_mut_ptr()) };
        assert_eq!(retval, KERN_SUCCESS);
        let mtt2 = unsafe { mtt1.assume_init() };

        //eprintln!("mach_timebase_info: {mtt2:?}");

        let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        while i < NUM_SAMPLES {
            let t1 = unsafe { mach_absolute_time() };

            black_box(dummy_func());

            let t2 = unsafe { mach_absolute_time() };
            assert!(t2 > t1);

            let nanos: u128 = u128::from(((t2 - t1) * mtt2.numer as u64) / mtt2.denom as u64);
            assert!(nanos > 0);
            durations.push(nanos);

            i += 1;
        }

        durations
    }
}
    
#[cfg(target_vendor = "apple")]
type ClockType = u32;

#[cfg(target_os = "linux")]
type ClockType = i32;

fn libc_gettime_clock(clock: ClockType) -> Vec<u128> {
    extern crate libc;

    let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
    let mut i = 0;
    
    while i < NUM_SAMPLES {
        let mut tp1: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
        let mut tp2: MaybeUninit<libc::timespec> = MaybeUninit::uninit();

        let retval1 = unsafe { libc::clock_gettime(clock, tp1.as_mut_ptr()) };

        black_box(dummy_func());

        let retval2 = unsafe { libc::clock_gettime(clock, tp2.as_mut_ptr()) };

        assert_eq!(retval1, 0);
        let instsec = unsafe { (*tp1.as_ptr()).tv_sec };
        let instnsec = unsafe { (*tp1.as_ptr()).tv_nsec };
        assert_eq!(retval2, 0);
        let newinstsec = unsafe { (*tp2.as_ptr()).tv_sec };
        let newinstnsec = unsafe { (*tp2.as_ptr()).tv_nsec };

        assert!(newinstsec * 1_000_000_000 + newinstnsec > instsec * 1_000_000_000 + instnsec, "newinstsec: {newinstsec}, newinstnsec: {newinstnsec}, instsec: {instsec}, instnsec: {instnsec}");
        let durnanosi64 = (newinstsec - instsec) * 1_000_000_000 + newinstnsec - instnsec;
        assert!(durnanosi64 > 0);
        let durnanos: u128 = durnanosi64.try_into().unwrap();
        assert!(durnanos > 0);

        durations.push(durnanos);

        i += 1;
    }

    durations
}
    
fn cputime_secs() -> Vec<u128> {
    libc_gettime_clock(libc::CLOCK_PROCESS_CPUTIME_ID)
}

fn monotonic_secs() -> Vec<u128> {
    libc_gettime_clock(libc::CLOCK_MONOTONIC)
}
    
fn monotonic_raw_secs() -> Vec<u128> {
    libc_gettime_clock(libc::CLOCK_MONOTONIC_RAW)
}
    
#[cfg(target_arch = "x86_64")]
pub mod plat_x86_64 {
    use crate::{dummy_func, NUM_SAMPLES};
    use core::arch::x86_64;
    use std::hint::black_box;

    pub fn rdtscp_ticks() -> Vec<u128> {
        let mut durticks = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        while i < NUM_SAMPLES {
            let mut aux1 = 0;
            let mut aux2 = 0;
            let now1 = unsafe { x86_64::__rdtscp(&mut aux1) };

            black_box(dummy_func());

            let now2 = unsafe { x86_64::__rdtscp(&mut aux2) };

            debug_assert!(now2 > now1);

            durticks.push((now2 - now1).into());

            i += 1;
        }

        durticks
    }
}

fn stats<F>(func: F, fnname: &str)
where
    F: Fn() -> Vec<u128>,
{
    let durations = func();

    let mut map: FxHashMap<u128, u128> = FxHashMap::default();

    for dur in durations {
        *map.entry(dur).or_insert(0) += 1;
    }

    let mut pairs: Vec<(&u128, &u128)> = map.iter().collect();

    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let mut perc50: u128 = 0;
    let mut perc95: u128 = 0;
    let min: u128 = *pairs[0].0;
    let max: u128 = *pairs[(*pairs).len()-1].0;

    let mut numsamples: u128 = 0;
    for (_nanos, num) in &pairs {
        numsamples += *num;
    }
    
    //println!("{:>10} {:>10} {:>10} {:>10}", "nanos", "#", "# < nanos", "# > nanos");
    //println!("{:>10} {:>10} {:>10} {:>10}", "-----", "-", "---------", "---------");
    let mut sumnanos = 0;
    let mut sumnums = 0;
    for (nanos, num) in &pairs {
        if (sumnums + *num >= numsamples * 95 / 100) && (sumnums < numsamples * 95 / 100) {
            //println!("95 percentile");
            perc95 = **nanos;
        }
        if (sumnums + *num >= numsamples * 50 / 100) && (sumnums < numsamples * 50 / 100) {
            //println!("50 percentile");
            perc50 = **nanos;
        }
        sumnanos += *nanos * *num;
        sumnums += *num;
    }
    assert!(sumnanos < i128::MAX as u128);
    let mean: i128 = (sumnanos / numsamples).try_into().unwrap();

    if perc50 == 0 {
        perc50 = *pairs[pairs.len()-1].0;
    }
    if perc95 == 0 {
        perc95 = *pairs[pairs.len()-1].0;
    }

    let mut sumsquares = 0;
    for (nanos, num) in pairs {
        let diff: i128 = *nanos as i128 - mean;
        let sqdiff: u128 = diff.pow(2).try_into().unwrap();
        sumsquares += sqdiff * *num;
    }
    let stddev = (sumsquares / (numsamples - 1)).isqrt();

    println!("{fnname:>38} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10}", numsamples.separate_with_commas(), min.separate_with_commas(), perc50.separate_with_commas(), mean.separate_with_commas(), perc95.separate_with_commas(), max.separate_with_commas(), stddev.separate_with_commas());
}

use thousands::Separable;

use std::thread;

macro_rules! add_wrapped_fn {
    ($vec:expr, $original:path) => {
        $vec.push(|| {
            stats($original, stringify!($original));
        });
    };
}

fn main() {
    let mut fns: Vec<fn()> = Vec::new();
    let mut handles = Vec::new();

    add_wrapped_fn!(fns, instant_secs);
    add_wrapped_fn!(fns, cputime_secs);
    add_wrapped_fn!(fns, monotonic_secs);
    add_wrapped_fn!(fns, monotonic_raw_secs);
#[cfg(target_vendor = "apple")]
    {
        add_wrapped_fn!(fns, plat_apple::m_mach_absolute_time_secs);
        add_wrapped_fn!(fns, plat_apple::uptime_raw_secs);
        add_wrapped_fn!(fns, plat_apple::nsec_np_uptime_raw_secs);
        add_wrapped_fn!(fns, plat_apple::nsec_np_cputime_secs);
        add_wrapped_fn!(fns, plat_apple::nsec_np_monotonic_secs);
        add_wrapped_fn!(fns, plat_apple::nsec_np_monotonic_raw_secs);
    }
#[cfg(target_arch = "x86_64")]
    add_wrapped_fn!(fns, plat_x86_64::rdtscp_ticks);

//    println!("NUM_SAMPLES: {}", NUM_SAMPLES.separate_with_commas());
    println!("{:>38} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10}", "fnname", "nsamples", "min", "perc50", "mean", "perc95", "max", "stddev");
    println!("{:>38} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10}", "------", "--------", "---", "------", "----", "------", "---", "------");

    for func in fns {
        let handle = thread::spawn(func);
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();  // Handle potential errors in production
    }
}
