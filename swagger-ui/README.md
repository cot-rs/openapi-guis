# swagger-ui-redist

[![crates.io](https://img.shields.io/crates/v/swagger-ui-redist.svg)](https://crates.io/crates/swagger-ui-redist)

This crate implements necessary boilerplate code to serve [Swagger UI] via web
server. It provides a simple API to configure the [Swagger UI] and serve it
via a web server. The crate is deliberately kept simple and does not
implement any web server specific code. It is up to the user to
implement the web server specific code for the web framework of choice.

It does not download Swagger UI from the internet, but rather includes the necessary
static files in the crate. This reduces the number of build dependencies and
makes it easy to use the crate offline.

It was mainly created to be integrated inside the [Cot web framework](https://cot.rs/),
but does not depend on it. It can be used with any web framework.

## Swagger UI version

<!-- version -->The version of Swagger UI included in this crate is v5.20.8.

## Attribution

This crate is heavily based on [`utoipa-swagger-ui`](https://github.com/juhaku/utoipa),
licensed under Apache 2.0/MIT.

[Swagger UI] included in this crate is licensed under Apache 2.0.

[Swagger UI]: https://swagger.io/tools/swagger-ui/
