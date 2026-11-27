macOS run on the current head `a35d30b9802ef6bc2c294e53ab1bc8743069980e` (macOS 26.6.2 build 25G83, arm64; rustc 1.99.0, cargo 1.99.0). No code changes; clean checkout of `istefp/soundcraft@fix/automation-plugin-params`.

| Check | Result |
|---|---|
| `cargo test -p soundcraft-au-host non_real_time_params_stay_editable_but_not_automatable` | 1 passed, 0 failed |
| `cargo test -p soundcraft-au-host` | 10 unit + 5 integration passed, 0 failed (0 doc-tests) |
| `cargo clippy -p soundcraft-au-host --all-targets -- -D warnings` | clean, exit 0 |
| `cargo fmt --check` | clean, exit 0 |

The only warnings are cargo's three "patch `egui_demo_lib`/`egui_inspection`/`egui_system_font_provider` was not used in the crate graph" notices, expected when building this crate alone.

Not run on macOS: the rest of the workspace (`soundcraft-ui-egui` etc. ran on Windows as recorded in the description) and `cargo xtask ci`.

<details><summary>Target test output</summary>

```
$ cargo test -p soundcraft-au-host non_real_time_params_stay_editable_but_not_automatable
    Finished `test` profile [optimized + debuginfo] target(s) in 35.47s
     Running unittests src/lib.rs (target/debug/deps/soundcraft_au_host-1482d001f453212a)

running 1 test
test registry::tests::non_real_time_params_stay_editable_but_not_automatable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/it/main.rs (target/debug/deps/it-e4d3931f4817b45c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```
</details>

<details><summary>Full crate output</summary>

```
$ cargo test -p soundcraft-au-host
     Running unittests src/lib.rs (target/debug/deps/soundcraft_au_host-1482d001f453212a)

running 10 tests
test ffi::tests::host_callbacks_report_the_transport ... ok
test ffi::tests::struct_layouts_match_the_c_headers ... ok
test registry::tests::non_real_time_params_stay_editable_but_not_automatable ... ok
test registry::tests::units_map ... ok
test registry::tests::params_map_and_hostile_ones_are_dropped ... ok
test tests::categories_and_names ... ok
test tests::foreign_ids_are_rejected_without_scanning ... ok
test tests::fourccs_round_trip ... ok
test tests::ids_parse_and_format ... ok
test tests::unknown_components_are_errors_not_crashes ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/it/main.rs (target/debug/deps/it-e4d3931f4817b45c)

running 5 tests
test system_units::scan_lists_apple_units ... ok
test system_units::lowpass_attenuates_highs ... ok
test system_units::state_reader_reads_while_another_thread_renders ... ok
test system_units::delay_runs_mono_stereo_and_odd_blocks ... ok
test system_units::dls_synth_plays_a_note ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

   Doc-tests soundcraft_au_host

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
</details>
