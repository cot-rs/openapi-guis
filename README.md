# openapi-guis

[![Rust Build Status](https://github.com/cot-rs/openapi-guis/workflows/Rust%20CI/badge.svg)](https://github.com/cot-rs/openapi-guis/actions/workflows/rust.yml)

OpenAPI GUIs (Swagger, Scalar, etc.) exposed as Rust crates

## Scope

It is a deliberate decision **not** to support any web frameworks in these crates. This is to maximize usability and reduce the maintenance burden. The goal is to provide a simple way to serve OpenAPI GUIs in any Rust application, regardless of the web framework used.

If you want to use this in a web framework, it should be easy enough to do so. You are also encouraged to create a wrapper crate that provides support for a specific web framework. If you do, please let me know and I will link to it here.

## License

Code in this repository is licensed under either of the following, at your option:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Cot by you shall be
dual licensed under the MIT License and Apache License, Version 2.0, without any additional terms or conditions.
