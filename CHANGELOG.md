# Changelog

All notable changes to this project are documented in this file.

## v0.2.9 - 2026-02-10

### Highlights
- Added an end-to-end regression and performance test suite (`tests/test_e2e_regression_perf.ps1`).
- Added framed `dump-state` transport for persistent clients to reduce control-path overhead.
- Improved remote rendering performance by writing `rows_v2` directly into the frame buffer.
- Reduced layout payload allocation cost by compacting `CellJson` fields.

### Performance and Reliability
- Added layout geometry and border caching improvements.
- Reused hot-path buffers (`dirty_pane_ids`, `cmd_batch`, scratch vectors) to lower allocation churn.
- Added O(1) base64 decode path using a lookup table.
- Cached shell path lookup for split/new-window operations.
- Replaced busy-wait behavior with `recv_timeout` in the server loop.
- Hardened session handling with stricter session-name validation and safer control lookups.
- Added connection limiting to prevent thread exhaustion.
- Fixed respawn leaks by terminating old pane processes before replacement.

### Testing
- Added E2E regression and performance coverage for:
  - authentication rejection behavior,
  - framed protocol negotiation and payload validity,
  - persistent control-path performance baselines.
