#![feature(rustc_private)]

use std::time::Instant;
use rustc_hash::FxHashMap;

extern crate libc;

const NUM_SAMPLES: u128 = 1000;

// const SLEEP_NANOS: u128 = 40_000;
const SLEEP_NANOS: u128 = 0;

fn instant() -> Vec<u128> {
    eprintln!("instant");

    let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
    let mut i = 0;

    while i < NUM_SAMPLES {
        let inst = Instant::now();

        thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

        let d = inst.elapsed();
        let dif = d.as_nanos() - SLEEP_NANOS;
        durations.push(dif);

        i += 1;
    }

    durations
}

use std::mem::MaybeUninit;

#[cfg(target_vendor = "apple")]
pub mod plat_apple {
    use crate::{NUM_SAMPLES, SLEEP_NANOS, thread, Duration, libc_gettime_clock, MaybeUninit};
    use libc::clockid_t;
    unsafe extern "C" {
        fn clock_gettime_nsec_np(clk_id: clockid_t) -> u64;
    }

    pub fn measure_clock_gettime_nsec_np() -> Vec<u128> {
        eprintln!("clock_gettime_nsec_np");

        let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        while i < NUM_SAMPLES {
            let prev = unsafe { clock_gettime_nsec_np(libc::CLOCK_UPTIME_RAW) };

            thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

            let now = unsafe { clock_gettime_nsec_np(libc::CLOCK_UPTIME_RAW) };

            let dur = now - prev;
            let dif = dur - SLEEP_NANOS as u64;
            
            durations.push(dif as u128);

            i += 1;
        }

        durations
    }

    pub fn libc_gettime_uptime_raw() -> Vec<u128> {
        eprintln!("libc_gettime_uptime_raw");
        libc_gettime_clock(libc::CLOCK_UPTIME_RAW)
    }
    
    use mach_sys::mach_time::{mach_absolute_time, mach_timebase_info};
    use mach_sys::kern_return::KERN_SUCCESS;
    pub fn mat() -> Vec<u128> {
        eprintln!("mach_absolute_time");

        let mut mtt1: MaybeUninit<mach_timebase_info> = MaybeUninit::uninit();
        let retval = unsafe { mach_timebase_info(mtt1.as_mut_ptr()) };
        assert_eq!(retval, KERN_SUCCESS);
        let mtt2 = unsafe { mtt1.assume_init() };

        eprintln!("mach_timebase_info: {mtt2:?}");

        let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        while i < NUM_SAMPLES {
            let t1 = unsafe { mach_absolute_time() };

            thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

            let t2 = unsafe { mach_absolute_time() };

            let nanos = ((t2 - t1) * mtt2.numer as u64) / mtt2.denom as u64;
            let dif = nanos - SLEEP_NANOS as u64;
            durations.push(dif as u128);

            i += 1;
        }

        durations
    }
}
    
#[cfg(target_os = "macos")]
type ClockType = u32;

#[cfg(target_os = "linux")]
type ClockType = i32;

use std::thread;
use std::time::Duration;

fn libc_gettime_clock(clock: ClockType) -> Vec<u128> {
    let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
    let mut i = 0;
    
    while i < NUM_SAMPLES {
        let mut tp1: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
        let mut tp2: MaybeUninit<libc::timespec> = MaybeUninit::uninit();

        let retval1 = unsafe { libc::clock_gettime(clock as ClockType, tp1.as_mut_ptr()) };

        thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

        let retval2 = unsafe { libc::clock_gettime(clock as ClockType, tp2.as_mut_ptr()) };

        assert_eq!(retval1, 0);
        let instsec = unsafe { (*tp1.as_ptr()).tv_sec };

        thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

        let instnsec = unsafe { (*tp1.as_ptr()).tv_nsec };
        assert_eq!(retval2, 0);
        let newinstsec = unsafe { (*tp2.as_ptr()).tv_sec };
        let newinstnsec = unsafe { (*tp2.as_ptr()).tv_nsec };

        let durnanos = (newinstsec - instsec) * 1_000_000_000 + newinstnsec - instnsec;
        let dif = durnanos - SLEEP_NANOS as i64;

        durations.push(dif as u128);

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
    
#[cfg(target_arch = "x86_64")]
pub mod plat_x86_64 {
    use crate::{NUM_SAMPLES, SLEEP_NANOS, thread, Duration};
    use core::arch::x86_64;

    use cpuid;

    pub fn measure_rdtscp() -> Vec<u128> {
        eprintln!("rdtscp");

        let mut durations = Vec::with_capacity(NUM_SAMPLES as usize);
        let mut i = 0;
    
        let ofreq = cpuid::clock_frequency();
        assert!(ofreq.is_some());
        let mut prev_freq_mhz = ofreq.unwrap();
        eprintln!("freq {} MHz", prev_freq_mhz);

        while i < NUM_SAMPLES {
            let mut aux1 = 0;
            let mut aux2 = 0;
            let now1 = unsafe { x86_64::__rdtscp(&mut aux1) };

            thread::sleep(Duration::from_nanos(SLEEP_NANOS as u64));

            let now2 = unsafe { x86_64::__rdtscp(&mut aux2) };

            assert_eq!(aux1, aux2);

            let durcycles = now2 - now1;

            let ofreq = cpuid::clock_frequency();
            assert!(ofreq.is_some());
            let freq_mhz = ofreq.unwrap();
            assert_eq!(prev_freq_mhz, freq_mhz); // frequency changed
            prev_freq_mhz = freq_mhz;

            let durnanos = durcycles * 1000 / freq_mhz as u64;
            let dif = durnanos - SLEEP_NANOS as u64;
            
            durations.push(dif as u128);

            i += 1;
        }

        durations
    }
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
        println!("{:>10} {:>10} {:>10} {:>10}", key.separate_with_commas(), value.separate_with_commas(), (value + sum).separate_with_commas(), (NUM_SAMPLES - value - sum).separate_with_commas());
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
#[cfg(target_vendor = "apple")]
    {
        stats(plat_apple::libc_gettime_uptime_raw);
        eprintln!();
        stats(plat_apple::mat);
        eprintln!();
        stats(plat_apple::measure_clock_gettime_nsec_np);
    }
#[cfg(target_arch = "x86_64")]
    stats(plat_x86_64::measure_rdtscp);
}
