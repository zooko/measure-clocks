#![feature(rustc_private)]

use std::time::Instant;
use rustc_hash::FxHashMap;

const DEFAULT_ITERS: u64 = 100_000;

const CALTIME_NANOS: u64 = 1_000_000;
const D: Duration = Duration::from_nanos(CALTIME_NANOS);

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
fn instant(_clock: Option<ClockType>, iters: u64) -> Vec<u64> {
    let mut durations = Vec::with_capacity(iters as usize);

    let mut i = 0;
    while i < iters {
        let inst = Instant::now();

        black_box(dummy_func());

        let d = inst.elapsed();
        assert!(d.as_nanos() > 0);
        durations.push(d.as_nanos() as u64);

        i += 1;
    }

    durations
}

#[cfg(target_os = "windows")]
pub mod plat_windows {
    use windows_sys::Win32::System::Performance::QueryPerformanceCounter;
    use crate::{black_box, dummy_func, Instant, sleep, ClockType, DEFAULT_ITERS, D};
    
    pub fn qpc(_clock: Option<ClockType>) -> Vec<u64> {
        let iters = get_iters();
        let mut res = Vec::with_capacity(iters);
        let mut i = 0;

        while i < iters {
	    let mut start: i64 = 0;
	    let mut stop: i64 = 0;
	    let start_result = unsafe { QueryPerformanceCounter(&mut start) };

            black_box(dummy_func());

	    let stop_result = unsafe { QueryPerformanceCounter(&mut stop) };

	    assert!(start_result != 0);
	    assert!(stop_result != 0);
            assert!(stop > start);
            let ticks = stop as u64 - start as u64;

            res.push(ticks);

            i += 1;
        }

	res
    }

    /// Returns the number of qpc ticks per nanosecond, in (numer, denomer) format.
    /// Sleeps for about a millisecond in order to calibrate.
    pub fn qpc_calibrate(_clock: Option<ClockType>) -> (u64, u64) {
	let mut start: i64 = 0;
	let mut stop: i64 = 0;

	// let mut frequency: i64 = 0;
	// let freq_result = unsafe { QueryPerformanceFrequency(&mut frequency) };
	// assert!(freq_result != 0);

	let start_instant = Instant::now();
	let start_result = unsafe { QueryPerformanceCounter(&mut start) };

        sleep(D);

	let stop_result = unsafe { QueryPerformanceCounter(&mut stop) };
	let elap = start_instant.elapsed();

	assert!(elap.as_nanos() > 0);
	assert!(start_result != 0);
	assert!(stop_result != 0);
	assert!(start > 0);
	assert!(stop > 0);
        assert!(stop > start);

	let ticks = stop as u64 - start as u64;
	let nanos = elap.as_nanos() as u64;

	// println!("QueryPerformanceFrequency said the ticks per second is {}. Our calibration says the ticks per nanosecond is {}/{}", freq_result, ticks, nanos);

	(ticks, nanos)
    }
}

#[cfg(target_vendor = "apple")]
pub mod plat_apple {
    use std::hint::black_box;
    use crate::{ClockType, dummy_func, D};
    extern crate libc;
    use libc::clockid_t;
    unsafe extern "C" {
        fn clock_gettime_nsec_np(clk_id: clockid_t) -> u64;
    }

    /// Returns the number of this clock's nanoseconds per Instant::now() nanoseconds, in (numer,
    /// denomer) format. Sleeps for about a millisecond in order to calibrate.
    pub fn gettime_nsec_np_clock_calibrate(clock: Option<ClockType>) -> (u64, u64) {
        let ct = clock.unwrap();

        let start_instant = Instant::now();
        let prev = unsafe { clock_gettime_nsec_np(ct) };
        sleep(D);
        let now = unsafe { clock_gettime_nsec_np(ct) };
        let elap = start_instant.elapsed().as_nanos() as u64;

        assert!(elap > 0);
        assert!(now > prev);

        let dur: u64 = now - prev;

        (dur, elap)
    }

    pub fn gettime_nsec_np_clock(clock: Option<ClockType>, iters: u64) -> Vec<u64> {
        let mut durations = Vec::with_capacity(iters as usize);
        let mut i = 0;
        let ct = clock.unwrap();
    
        while i < iters {
            let prev = unsafe { clock_gettime_nsec_np(ct) };

            black_box(dummy_func());

            let now = unsafe { clock_gettime_nsec_np(ct) };

            assert!(now > prev);
            let dur: u64 = now - prev;

            durations.push(dur);

            i += 1;
        }

        durations
    }

    use mach_sys::mach_time::{mach_absolute_time};
    use std::time::Instant;
    use std::thread::sleep;

    /// Returns the number of this clock's ticks per Instant::now()'s nanoseconds, in (numer,
    /// denomer) format. Sleeps for about a millisecond in order to calibrate.
    pub fn mach_absolute_time_ticks_calibrate(_clock: Option<ClockType>) -> (u64, u64) {
        //let mut mtt1: MaybeUninit<mach_timebase_info> = MaybeUninit::uninit();
        //let retval = unsafe { mach_timebase_info(mtt1.as_mut_ptr()) };
        //assert_eq!(retval, KERN_SUCCESS);
        //let mtt2 = unsafe { mtt1.assume_init() };

        let start_instant = Instant::now();
        let t1 = unsafe { mach_absolute_time() };
        sleep(D);
        let t2 = unsafe { mach_absolute_time() };
        let elap = start_instant.elapsed().as_nanos() as u64;

        assert!(elap > 0);
        assert!(t2 > t1);
        let ticks = t2 - t1;

        //eprintln!("Mach kernel says that the ratio of mach ticks to nanoseconds is {}/{}. Our calibratiom says that the ratio is {ticks}/{elap}.", mtt2.denom, mtt2.numer);
        (ticks, elap)
    }

    pub fn mach_absolute_time_ticks(_clock: Option<ClockType>, iters: u64) -> Vec<u64> {
        //let mut mtt1: MaybeUninit<mach_timebase_info> = MaybeUninit::uninit();
        //let retval = unsafe { mach_timebase_info(mtt1.as_mut_ptr()) };
        //assert_eq!(retval, KERN_SUCCESS);
        //let mtt2 = unsafe { mtt1.assume_init() };

        //eprintln!("mach_timebase_info: {mtt2:?}");

        let mut durations = Vec::with_capacity(iters as usize);
        let mut i = 0;
    
        while i < iters {
            let t1 = unsafe { mach_absolute_time() };

            black_box(dummy_func());

            let t2 = unsafe { mach_absolute_time() };
            assert!(t2 > t1);

            let ticks = t2 - t1;
            assert!(ticks > 0);
            durations.push(ticks);

            i += 1;
        }

        durations
    }
}
    
#[cfg(target_vendor = "apple")]
type ClockType = u32;

#[cfg(any(target_os = "linux", target_os = "windows"))]
type ClockType = i32;

use std::time::Duration;
use std::thread::sleep;

/// Returns the number of this clock's nanoseconds per Instant::now() nanoseconds, in (numer,
/// denomer) format. Sleeps for about a millisecond in order to calibrate. Note that it is using the
/// same clock for both of the measurements, so this is actually measuring nothing but the error in
/// our calibration tecnnique. :-} Most of the reason we are doing this at all is the insert a 1sec
/// delay before beginning the measurements of the clock, so that all of the measurements of clocks,
/// which are done on separate threads, will be running simultaneously.
fn instant_calibrate(_clock: Option<ClockType>) -> (u64, u64) {
    let start_instant1 = Instant::now();
    let start_instant2 = Instant::now();
    sleep(D);
    let elap2 = start_instant2.elapsed().as_nanos() as u64;
    let elap1 = start_instant1.elapsed().as_nanos() as u64;
    assert!(elap1 > 0);
    assert!(elap2 > 0);

    (elap2, elap1)
}

#[cfg(not(target_os = "windows"))]
pub mod plat_unixes {
    pub extern crate libc;
    use std::mem::MaybeUninit;
    use crate::{ClockType, D, Instant, sleep, black_box, dummy_func};

    /// Returns the number of this clock's nanoseconds per Instant::now() nanoseconds, in (numer,
    /// denomer) format. Sleeps for about a millisecond in order to calibrate.
    pub fn libc_gettime_clock_calibrate(clock: Option<ClockType>) -> (u64, u64) {
	let ct = clock.unwrap();

	let mut tp1: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
	let mut tp2: MaybeUninit<libc::timespec> = MaybeUninit::uninit();

	let start_instant = Instant::now();
	let retval1 = unsafe { libc::clock_gettime(ct, tp1.as_mut_ptr()) };
	sleep(D);
	let retval2 = unsafe { libc::clock_gettime(ct, tp2.as_mut_ptr()) };
	let elap = start_instant.elapsed();
	assert!(elap.as_nanos() > 0);

	assert_eq!(retval1, 0);
	let instsec = unsafe { (*tp1.as_ptr()).tv_sec };
	let instnsec = unsafe { (*tp1.as_ptr()).tv_nsec };
	assert_eq!(retval2, 0);
	let newinstsec = unsafe { (*tp2.as_ptr()).tv_sec };
	let newinstnsec = unsafe { (*tp2.as_ptr()).tv_nsec };

	assert!(newinstsec * 1_000_000_000 + newinstnsec > instsec * 1_000_000_000 + instnsec, "newinstsec: {newinstsec}, newinstnsec: {newinstnsec}, instsec: {instsec}, instnsec: {instnsec}");
	let durnanosi64 = (newinstsec - instsec) * 1_000_000_000 + newinstnsec - instnsec;
	assert!(durnanosi64 > 0);
	let durnanos: u64 = durnanosi64.try_into().unwrap();
	(durnanos, elap.as_nanos() as u64)
    }

    pub fn libc_gettime_clock(clock: Option<ClockType>, iters: u64) -> Vec<u64> {
	let mut durations = Vec::with_capacity(iters as usize);
	let mut i = 0;
	let ct = clock.unwrap();
	
	while i < iters {
            let mut tp1: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
            let mut tp2: MaybeUninit<libc::timespec> = MaybeUninit::uninit();

            let retval1 = unsafe { libc::clock_gettime(ct, tp1.as_mut_ptr()) };

            black_box(dummy_func());

            let retval2 = unsafe { libc::clock_gettime(ct, tp2.as_mut_ptr()) };

            assert_eq!(retval1, 0);
            let instsec = unsafe { (*tp1.as_ptr()).tv_sec };
            let instnsec = unsafe { (*tp1.as_ptr()).tv_nsec };
            assert_eq!(retval2, 0);
            let newinstsec = unsafe { (*tp2.as_ptr()).tv_sec };
            let newinstnsec = unsafe { (*tp2.as_ptr()).tv_nsec };

            assert!(newinstsec * 1_000_000_000 + newinstnsec > instsec * 1_000_000_000 + instnsec, "newinstsec: {newinstsec}, newinstnsec: {newinstnsec}, instsec: {instsec}, instnsec: {instnsec}");
            let durnanosi64 = (newinstsec - instsec) * 1_000_000_000 + newinstnsec - instnsec;
            assert!(durnanosi64 > 0);
            let durnanos: u64 = durnanosi64.try_into().unwrap();
            assert!(durnanos > 0);

            durations.push(durnanos);

            i += 1;
	}

	durations
    }
}

#[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
pub mod plat_x86_64 {
    use crate::{ClockType, dummy_func, DEFAULT_ITERS, D};
    use core::arch::x86_64;
    use std::hint::black_box;
    use std::thread::sleep;
    use std::time::Instant;

    pub fn rdtscp(_clock: Option<ClockType>, iters: u64) -> Vec<u64> {
        let mut aux = 0;

        let mut res = Vec::with_capacity(iters);
        let mut i = 0;
        
        while i < iters {
            let now1 = unsafe { x86_64::__rdtscp(&mut aux) };

            black_box(dummy_func());

            let now2 = unsafe { x86_64::__rdtscp(&mut aux) };

            debug_assert!(now2 > now1);
            let ticks = now2 as u64 - now1 as u64;

            res.push(ticks);

            i += 1;
        }

        res
    }

    /// Returns the number of tsc ticks per nanosecond, in (numer, denomer) format.
    /// Sleeps for about a millisecond in order to calibrate.
    pub fn rdtscp_calibrate(_clock: Option<ClockType>) -> (u64, u64) {
        let mut aux = 0;
        let start_instant = Instant::now();
        let start_tsc = unsafe { x86_64::__rdtscp(&mut aux) };
        sleep(D);
        let end_tsc = unsafe { x86_64::__rdtscp(&mut aux) };
        let elap = start_instant.elapsed();
        assert!(end_tsc > start_tsc);
        assert!(elap.as_nanos() > 0);

        ((end_tsc - start_tsc).into(), elap.as_nanos() as u64)
    }
}

fn stats<F, G>(func: F, calibrate: G, clock: Option<ClockType>, fnname: &str, clockname: &str, scale: bool)
where
    F: Fn(Option<ClockType>, u64) -> Vec<u64>,
    G: Fn(Option<ClockType>) -> (u64, u64),
{
    let iters = get_iters();

    let (numer, denomer) = calibrate(clock);
    let durations = func(clock, iters);

    let mut map: FxHashMap<u64, u64> = FxHashMap::default();

    if scale {
        for dur in durations {
            let nanos = dur * denomer / numer;
            *map.entry(nanos).or_insert(0) += 1;
        }
    } else {
        for dur in durations {
            *map.entry(dur).or_insert(0) += 1;
        }
    }
    
    let mut pairs: Vec<(&u64, &u64)> = map.iter().collect();

    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let mut perc50: u64 = 0;
    let mut perc95: u64 = 0;
    let min: u64 = *pairs[0].0;
    let max: u64 = *pairs[(*pairs).len()-1].0;

    let mut numsamples: u64 = 0;
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
    assert!(sumnanos < i64::MAX as u64);
    let mean: i64 = (sumnanos / numsamples).try_into().unwrap();

    if perc50 == 0 {
        perc50 = *pairs[pairs.len()-1].0;
    }
    if perc95 == 0 {
        perc95 = *pairs[pairs.len()-1].0;
    }

    let mut sumsquares: f64 = 0f64;
    for (nanos, num) in pairs {
        let f64ns: f64 = *nanos as f64;
        let diff: f64 = f64ns - mean as f64;
        let sqdiff: f64 = diff.powf(2f64);
        let n: f64 = (*num) as f64;
        sumsquares += sqdiff * n;
//        eprintln!("xyz, sumsquares: {}, sqdiff: {}, n: {}", sumsquares as u128, sqdiff as u128, n as u128);
    }
    let stddev = (sumsquares / (numsamples - 1) as f64).sqrt();

    if scale {
        println!("{fnname:>38} {clockname:>15} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10} {:>9}", numsamples.separate_with_commas(), min.separate_with_commas(), perc50.separate_with_commas(), mean.separate_with_commas(), perc95.separate_with_commas(), max.separate_with_commas(), (stddev as u128).separate_with_commas(), "---");
    } else {
        let drift = numer as f64 / denomer as f64;
        println!("{fnname:>38} {clockname:>15} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10} {:>9.6}", numsamples.separate_with_commas(), min.separate_with_commas(), perc50.separate_with_commas(), mean.separate_with_commas(), perc95.separate_with_commas(), max.separate_with_commas(), (stddev as u128).separate_with_commas(), drift);
    }

}

use thousands::Separable;

use std::thread;

macro_rules! add_wrapped_fn {
    ($vec:expr, $func:path, $calibrate:path, $clock:expr, $scale:expr) => {
        $vec.push(move || {
            // Full stringified clock (e.g., "Some(libc::CLOCK_THREAD_CPUTIME_ID)")
            let clock_str = stringify!($clock);

            let mut pruned_clockname = clock_str;
            if let Some(s) = pruned_clockname.strip_prefix("Some(") { pruned_clockname = s; }
            if let Some(s) = pruned_clockname.strip_prefix("libc::") { pruned_clockname = s; }
            if let Some(s) = pruned_clockname.strip_prefix("CLOCK_") { pruned_clockname = s; }
            if let Some(s) = pruned_clockname.strip_suffix(")") { pruned_clockname = s; }
            if let Some(s) = pruned_clockname.strip_suffix("_ID") { pruned_clockname = s; }
            
            // Call stats with the pruned clock name
            stats($func, $calibrate, $clock, stringify!($func), pruned_clockname, $scale);
        });
    };
}

fn get_iters() -> u64 {
    let args: Vec<String> = env::args().collect();

    // Iterate over the arguments to find the "--iters" argument
    for arg in &args {
        if let Some(iters_str) = arg.strip_prefix("--iters=") {
            if let Ok(argiters) = iters_str.parse::<u64>() {
                return argiters;
            } else {
                panic!();
            }
        }
    }

    DEFAULT_ITERS
}

use std::env;
fn main() {
    let mut fns: Vec<fn()> = Vec::new();
    let mut handles = Vec::new();

    add_wrapped_fn!(fns, instant, instant_calibrate, None, false);

#[cfg(not(target_os = "windows"))]
    {
        use crate::plat_unixes::{libc, libc_gettime_clock, libc_gettime_clock_calibrate};
    add_wrapped_fn!(fns, libc_gettime_clock, libc_gettime_clock_calibrate, Some(libc::CLOCK_THREAD_CPUTIME_ID), false);
    add_wrapped_fn!(fns, libc_gettime_clock, libc_gettime_clock_calibrate, Some(libc::CLOCK_MONOTONIC), false);
    add_wrapped_fn!(fns, libc_gettime_clock, libc_gettime_clock_calibrate, Some(libc::CLOCK_MONOTONIC_RAW), false);
    }
#[cfg(target_vendor = "apple")]
    {
        use crate::plat_unixes::{libc_gettime_clock, libc_gettime_clock_calibrate, libc};
        add_wrapped_fn!(fns, plat_apple::mach_absolute_time_ticks, plat_apple::mach_absolute_time_ticks_calibrate, None, true);
        add_wrapped_fn!(fns, libc_gettime_clock, libc_gettime_clock_calibrate, Some(libc::CLOCK_UPTIME_RAW), false);
        add_wrapped_fn!(fns, plat_apple::gettime_nsec_np_clock, plat_apple::gettime_nsec_np_clock_calibrate, Some(libc::CLOCK_UPTIME_RAW), false);
        add_wrapped_fn!(fns, plat_apple::gettime_nsec_np_clock, plat_apple::gettime_nsec_np_clock_calibrate, Some(libc::CLOCK_THREAD_CPUTIME_ID), false);
        add_wrapped_fn!(fns, plat_apple::gettime_nsec_np_clock, plat_apple::gettime_nsec_np_clock_calibrate, Some(libc::CLOCK_MONOTONIC), false);
        add_wrapped_fn!(fns, plat_apple::gettime_nsec_np_clock, plat_apple::gettime_nsec_np_clock_calibrate, Some(libc::CLOCK_MONOTONIC_RAW), false);
    }
#[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
    add_wrapped_fn!(fns, plat_x86_64::rdtscp, plat_x86_64::rdtscp_calibrate, None, true);
#[cfg(target_os = "windows")]
    add_wrapped_fn!(fns, plat_windows::qpc, plat_windows::qpc_calibrate, None, true);


//    println!("iters: {}", iters.separate_with_commas());
    println!("{:>38} {:>15} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10} {:>9}", "fnname", "clock", "nsamples", "min", "perc50", "mean", "perc95", "max", "stddev", "drift");
    println!("{:>38} {:>15} {:>11} {:>7} {:>7} {:>8} {:>9} {:>14} {:>10} {:>9}", "------", "-----", "--------", "---", "------", "----", "------", "---", "------", "-----");

    let args: Vec<String> = env::args().collect();

    let numthreadsperfunc = if args.contains(&"--overthread".to_string()) {
        let count = thread::available_parallelism().unwrap().get();
        assert!(count >= 1_usize);
        count * 2
    } else {
        1
    };

    for func in fns {
        for _i in 0..numthreadsperfunc {
            let handle = thread::spawn(func);
            handles.push(handle);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
