# XML-RPC for Rust

[![crates.io](https://img.shields.io/crates/v/simple-xmlrpc.svg)](https://crates.io/simple-xmlrpc)
[![docs.rs](https://docs.rs/simple-xmlrpc/badge.svg)](https://docs.rs/xmlrpc/)

This crate provides a simple implementation of the [XML-RPC specification](http://xmlrpc.scripting.com/spec.html) in stable Rust using `xml-rs`. It was originally based on the [xmlrpc crate](https://crates.io/xmlrpc), but with a large portion of the API removed.

Please refer to the [changelog](CHANGELOG.md) to see what changed in the last releases.

## Usage

Start by adding an entry to your `Cargo.toml`:

```toml
[dependencies]
simple-xmlrpc = "0.1.0"
```

Then import the crate into your Rust code:

```rust
use simple_xmlrpc
```
