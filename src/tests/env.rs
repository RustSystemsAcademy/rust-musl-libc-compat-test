use std::env;

pub fn round_trip() -> Result<(), String> {
    env::set_var("MUSL_TEST_KEY", "musl_test_value");
    let v = env::var("MUSL_TEST_KEY").map_err(|e| e.to_string())?;
    if v != "musl_test_value" {
        return Err(format!("expected 'musl_test_value', got '{}'", v));
    }
    env::remove_var("MUSL_TEST_KEY");
    Ok(())
}

pub fn iterate() -> Result<(), String> {
    let count = env::vars().count();
    if count == 0 {
        return Err("env::vars() returned nothing — musl environ may be broken".into());
    }
    Ok(())
}
