# serde-xmlrpc

[![Build Status](https://github.com/belak/serde-xmlrpc/actions/workflows/rust.yml/badge.svg)](https://github.com/belak/serde-xmlrpc/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/serde_xmlrpc)](https://crates.io/crates/serde_xmlrpc)
[![Docs](https://img.shields.io/badge/docs-stable-blue)](https://docs.rs/serde_xmlrpc)

This library is meant to be a simple XMLRPC library with the minimal support
needed to build out applications using XMLRPC. No additional parsing, no
transports, etc.

## Breaking Changes

### v0.3.0

* `value_from_str` changed to return `T` where `T: serde::de::Deserialize<'a>`
* `value_to_string` changed to take `T` where `T: serde::ser::Serialize`
* `request_to_string` changed to take an `impl Iterator<Item = Value>`
* Structs changed to only allow string types as keys

### v0.2.0

* `response_from_str` changed to take an `impl Iterator<Item = Value>`
