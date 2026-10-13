#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|s: &str| {
    let _ = cleansys_core::engine::parse_spec_file(s);
});
