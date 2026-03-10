use std::sync::{Arc, Barrier, Mutex};
use std::thread;

pub fn spawn_join() -> Result<(), String> {
    let handles: Vec<_> = (0..8)
        .map(|i| thread::spawn(move || i * i))
        .collect();

    let results: Vec<u64> = handles
        .into_iter()
        .map(|h| h.join().map_err(|_| "thread panicked".to_string()))
        .collect::<Result<_, _>>()?;

    let expected: Vec<u64> = (0..8).map(|i: u64| i * i).collect();
    if results != expected {
        return Err(format!("unexpected results: {:?}", results));
    }
    Ok(())
}

pub fn tls_isolation() -> Result<(), String> {
    // Each thread writes a unique value into TLS then reads it back after
    // a barrier to confirm no cross-thread bleed.
    const N: usize = 8;
    let barrier = Arc::new(Barrier::new(N + 1));
    let errors  = Arc::new(Mutex::new(Vec::<String>::new()));

    let handles: Vec<_> = (0..N).map(|i| {
        let b  = Arc::clone(&barrier);
        let e  = Arc::clone(&errors);
        thread::spawn(move || {
            thread_local!(static V: std::cell::RefCell<usize> =
                          std::cell::RefCell::new(0));
            V.with(|v| *v.borrow_mut() = i * 0xDEAD);
            b.wait(); // all threads write before any reads
            V.with(|v| {
                if *v.borrow() != i * 0xDEAD {
                    e.lock().unwrap().push(
                        format!("thread {} TLS corrupted: got {}", i, *v.borrow())
                    );
                }
            });
        })
    }).collect();

    barrier.wait();
    for h in handles { h.join().map_err(|_| "thread panicked".to_string())?; }

    let errs = errors.lock().unwrap();
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}

pub fn barrier() -> Result<(), String> {
    const N: usize = 4;
    let barrier = Arc::new(Barrier::new(N));
    let counter = Arc::new(Mutex::new(0u32));

    let handles: Vec<_> = (0..N).map(|_| {
        let b = Arc::clone(&barrier);
        let c = Arc::clone(&counter);
        thread::spawn(move || {
            *c.lock().unwrap() += 1;
            b.wait(); // nobody proceeds until all N have incremented
        })
    }).collect();

    for h in handles { h.join().map_err(|_| "thread panicked".to_string())?; }
    let val = *counter.lock().unwrap();
    if val != N as u32 {
        return Err(format!("counter is {}, expected {}", val, N));
    }
    Ok(())
}
