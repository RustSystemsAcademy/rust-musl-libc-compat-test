use std::env;

pub fn round_trip() -> Result<(), String> {
    // SAFETY: test binary is single-threaded at this point, no other thread
    // is reading the environment concurrently.
    unsafe { std::env::set_var("MUSL_TEST_KEY", "musl_test_value") };

    let v = std::env::var("MUSL_TEST_KEY").map_err(|e| e.to_string())?;
    if v != "musl_test_value" {
        return Err(format!("expected 'musl_test_value', got '{}'", v));
    }

    unsafe { std::env::remove_var("MUSL_TEST_KEY") };
    Ok(())
}

pub fn iterate() -> Result<(), String> {
    let count = env::vars().count();
    if count == 0 {
        return Err("env::vars() returned nothing — musl environ may be broken".into());
    }
    Ok(())
}
