# openapi-guis

OpenAPI GUIs (Swagger, Scalar, etc.) exposed as Rust crates

## Scope

It is a deliberate decision **not** to support any web frameworks in these crates. This is to maximize usability and reduce the maintenance burden. The goal is to provide a simple way to serve OpenAPI GUIs in any Rust application, regardless of the web framework used.

If you want to use this in a web framework, it should be easy enough to do so. You are also encouraged to create a wrapper crate that provides support for a specific web framework. If you do, please let me know and I will link to it here.

## Crate versions

The crate versions correspond to the version of the OpenAPI GUI that is used. They still adhere to the [Semantic Versioning](https://semver.org/) rules and the Rust code should not break between minor versions.
