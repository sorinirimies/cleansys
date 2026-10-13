#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|s: &str| {
    let _ = serde_json::from_str::<cleansys_core::settings::Settings>(s);
    let _ = cleansys_core::theme_index_by_name(s);
});
