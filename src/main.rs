#![feature(rustc_private)]

use std::time::Instant;
use rustc_hash::FxHashMap;

extern crate libc;

const NUM_ARGS: u128 = 1_000_000;

fn instant() -> Vec<u128> {
    eprintln!("instant");

    let mut durations = Vec::with_capacity(NUM_ARGS as usize);
    let mut i = 0;

    while i < NUM_ARGS {
        let inst = Instant::now();
        durations.push(inst.elapsed().as_nanos());

        i += 1;
    }

    durations
}

use std::mem::MaybeUninit;

fn libc_gettime_clock(clock: u32) -> Vec<u128> {
    let mut durations = Vec::with_capacity(NUM_ARGS as usize);
    let mut i = 0;
    
    while i < NUM_ARGS {
        let mut tp1: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
        let mut tp2: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
        let retval1 = unsafe { libc::clock_gettime(clock, tp1.as_mut_ptr()) };
        let retval2 = unsafe { libc::clock_gettime(clock, tp2.as_mut_ptr()) };
        assert_eq!(retval1, 0);
        let instsec = unsafe { (*tp1.as_ptr()).tv_sec };
        let instnsec = unsafe { (*tp1.as_ptr()).tv_nsec };
        assert_eq!(retval2, 0);
        let newinstsec = unsafe { (*tp2.as_ptr()).tv_sec };
        let newinstnsec = unsafe { (*tp2.as_ptr()).tv_nsec };

        let dur = (newinstsec - instsec) * 1_000_000_000 + newinstnsec - instnsec;
            
        durations.push(dur as u128);

        i += 1;
    }

    durations
}
    
fn libc_gettime_cputime() -> Vec<u128> {
    eprintln!("libc_gettime_cputime");
    libc_gettime_clock(libc::CLOCK_PROCESS_CPUTIME_ID)
}

fn libc_gettime_monotonic() -> Vec<u128> {
    eprintln!("libc_gettime_monotonic");
    libc_gettime_clock(libc::CLOCK_MONOTONIC)
}
    
fn libc_gettime_monotonic_raw() -> Vec<u128> {
    eprintln!("libc_gettime_monotonic_raw");
    libc_gettime_clock(libc::CLOCK_MONOTONIC_RAW)
}
    
fn libc_gettime_uptime_raw() -> Vec<u128> {
    eprintln!("libc_gettime_uptime_raw");
    libc_gettime_clock(libc::CLOCK_UPTIME_RAW)
}
    
use mach_sys::mach_time::{mach_absolute_time, mach_timebase_info};
use mach_sys::kern_return::KERN_SUCCESS;
fn mat() -> Vec<u128> {
    eprintln!("mach_absolute_time");

    let mut mtt1: MaybeUninit<mach_timebase_info> = MaybeUninit::uninit();
    let retval = unsafe { mach_timebase_info(mtt1.as_mut_ptr()) };
    assert_eq!(retval, KERN_SUCCESS);
    let mtt2 = unsafe { mtt1.assume_init() };

    eprintln!("mach_timebase_info: {mtt2:?}");

    let mut durations = Vec::with_capacity(NUM_ARGS as usize);
    let mut i = 0;
    
    while i < NUM_ARGS {
        let t1 = unsafe { mach_absolute_time() };
        let t2 = unsafe { mach_absolute_time() };

        let nanos = ((t2 - t1) * mtt2.numer as u64) / mtt2.denom as u64;
        durations.push(nanos as u128);

        i += 1;
    }

    durations
}

use thousands::Separable;
fn stats<F>(func: F)
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

    println!("{:>10} {:>10} {:>10} {:>10}", "nanos", "#", "# < nanos", "# > nanos");
    println!("{:>10} {:>10} {:>10} {:>10}", "-----", "-", "---------", "---------");
    let mut sum = 0;
    for (key, value) in pairs {
        println!("{:>10} {:>10} {:>10} {:>10}", key.separate_with_commas(), value.separate_with_commas(), (value + sum).separate_with_commas(), (NUM_ARGS - value - sum).separate_with_commas());
        sum += value;
    }
}

fn main() {
    println!("Hello, world!");

    stats(instant);
    eprintln!();
    stats(libc_gettime_cputime);
    eprintln!();
    stats(libc_gettime_monotonic);
    eprintln!();
    stats(libc_gettime_monotonic_raw);
    eprintln!();
    stats(libc_gettime_uptime_raw);
    eprintln!();
    stats(mat);
}
