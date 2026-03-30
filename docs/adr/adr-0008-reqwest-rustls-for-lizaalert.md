# ADR-0008: reqwest + rustls for LizaAlert HTTP

- Status: accepted
- Date: 2026-03-28

## Context

The application fetches LizaAlert project listings and map packages from
`https://maps.lizaalert.ru`. This requires an HTTP client that:
- supports HTTPS (maps.lizaalert.ru does not serve HTTP)
- works in a blocking context (background thread, not async)
- compiles to a self-contained binary without native TLS system dependency

Initial integration used reqwest without TLS configuration, which produced connection
errors on the LizaAlert HTTPS endpoint.

## Decision

Use **reqwest 0.13 with `default-features = false` and `features = ["blocking", "rustls"]`**.

```toml
reqwest = { version = "0.13.2", default-features = false, features = ["blocking", "rustls"] }
```

- `blocking` — synchronous API used in background threads via `std::thread::spawn`
- `rustls` — pure-Rust TLS implementation; no dependency on OpenSSL or system TLS
- `default-features = false` — excludes the default `native-tls` backend which requires
  OpenSSL on Linux and is harder to cross-compile

## Consequences

### Positive

- No OpenSSL or system TLS library dependency; simpler builds on all platforms
- Pure Rust TLS stack is auditable and cross-platform
- Blocking API fits the background-thread model without an async runtime

### Negative

- `rustls` may refuse certificates that a platform's native TLS would accept
  (e.g. certain corporate or government CAs not in the webpki trust store)
- reqwest 0.13 blocking API is less flexible than async if concurrency is needed later

## Rejected Alternatives

### reqwest with native-tls

Rejected because it requires OpenSSL on Linux and complicates CI and cross-compilation.

### ureq

Considered but not chosen; reqwest was already a dependency and has better ecosystem
support for streaming downloads (needed for large map packages).
