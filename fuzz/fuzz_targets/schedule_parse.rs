#![no_main]
use cleansys_core::engine::schedule::{cron_with_block, parse_time, parse_weekday, strip_cron_block};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (&str, &str)| {
    let (a, b) = data;
    let _ = parse_time(a);
    let _ = parse_weekday(a);
    let _ = strip_cron_block(a);
    let _ = cron_with_block(a, b);
});
