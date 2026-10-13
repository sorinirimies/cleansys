#![no_main]
use cleansys_core::engine::paths::expand;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|s: &str| {
    let env = |k: &str| Some(format!("/v/{k}"));
    let _ = expand(s, &env);
    let _ = expand(s, &|_: &str| None);
});
