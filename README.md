# rust-elements-consensus

This library provides native bindings to elements' consensus module and exposes it via a simple API.

## Should I use this?

This is using the same code as elementsd to validate consensus rules.
_In theory_ you should therefore never see any mismatch (assuming we vendor the code regularly to get bugfixes etc).

Nevertheless, we urge you to make your own judgement.

This crate has primarily been developed to speed up testing of complex contracts and protocols.
By testing against the consensus rules directly, you don't need to spin up instances of elementsd to verify that your signed transaction would verify.

## Usage

The native code provided by this library depends on the C++ standard library.
It is the user's responsibility to specify, which standard library to link.

The easiest way of doing this through [`link_cplusplus`](https://docs.rs/link-cplusplus).
The default behaviour has been tested on Ubuntu and MacOS, so most likely you only need to:

1. Depend on `link_cplusplus`:

```toml
[dev-dependencies]
link-cplusplus = "1"
```

2. Add `extern crate link_cplusplus;`:

```rust
#[cfg(test)]
extern crate link_cplusplus;
```

See the documentation of `link_cplusplus` for details.
