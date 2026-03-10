use std::fs;
use std::time::{Duration, SystemTime};

pub fn monotonic_clock() -> Result<(), String> {
    let before = SystemTime::now();
    std::thread::sleep(Duration::from_millis(20));
    let elapsed = before.elapsed().map_err(|e| e.to_string())?;
    if elapsed < Duration::from_millis(20) {
        return Err(format!("elapsed {:?} is less than 20ms sleep", elapsed));
    }
    Ok(())
}

pub fn clock_boottime() -> Result<(), String> {
    // CLOCK_BOOTTIME available since 2.6.39 — safe on 3.12
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    if ret != 0 {
        return Err(format!("clock_gettime(CLOCK_BOOTTIME) returned {}", ret));
    }
    if ts.tv_sec < 0 {
        return Err(format!("bogus uptime: {} seconds", ts.tv_sec));
    }
    Ok(())
}

pub fn proc_self() -> Result<(), String> {
    let pid = std::process::id();
    if pid == 0 {
        return Err("getpid() returned 0".into());
    }

    let exe = fs::read_link("/proc/self/exe").map_err(|e| {
        format!("/proc/self/exe unreadable: {}", e)
    })?;

    if !exe.exists() {
        return Err(format!("/proc/self/exe -> {:?} does not exist", exe));
    }

    Ok(())
}
