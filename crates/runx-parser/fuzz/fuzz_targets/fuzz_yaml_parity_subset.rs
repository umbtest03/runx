#![no_main]

// Differential property:
//   if `assert_yaml_parity_subset` returns Ok and serde_norway parses the
//   document into a mapping at the top level, then no key in that mapping
//   contains `": "` (colon-space). The parity validator's contract is "no
//   top-level ambiguous mapping construct"; serde_norway is the authoritative
//   reader; this asserts they agree on what got past the validator.
//
// Run with `cargo +nightly fuzz run fuzz_yaml_parity_subset -- -max_total_time=60`
// from inside `crates/runx-parser/fuzz`.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let parity = runx_parser::assert_yaml_parity_subset("fuzz", text);
    let parsed: Result<serde_norway::Value, _> = serde_norway::from_str(text);
    if let (Ok(()), Ok(serde_norway::Value::Mapping(map))) = (parity, parsed) {
        for (key, _) in map {
            if let serde_norway::Value::String(string_key) = key {
                assert!(
                    !string_key.contains(": "),
                    "validator accepted colon-space top-level key: {string_key:?}\ninput: {text:?}"
                );
            }
        }
    }
});
