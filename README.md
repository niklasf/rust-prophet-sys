# prophet-sys

[![crates.io](https://img.shields.io/crates/v/prophet-sys.svg)](https://crates.io/crates/prophet-sys)
[![docs.rs](https://docs.rs/prophet-sys/badge.svg)](https://docs.rs/prophet-sys)

Low-level Rust bindings for [libprophet](https://github.com/markus7800/prophet_tb_gen_and_probe),
a library to probe 6-piece "Prophet" chess endgame tablebases (depth to mate).

## Documentation

* [Generated API reference](https://docs.rs/prophet-sys)
* [`prophet.h`](https://github.com/markus7800/prophet_tb_gen_and_probe/blob/main/src/prophet.h)

## Safety

Verify tablebase checksums before using them.
Do not modify, move or delete tablebase files that are loaded.
Only probe with valid positions.

## License

These bindings and the original library are licensed under the GPL-3.0.
See the LICENSE file for the full license text.
