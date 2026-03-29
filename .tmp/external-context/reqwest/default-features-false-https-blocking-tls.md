---
source: docs.rs and upstream source
library: reqwest
package: reqwest
topic: HTTPS support with default-features = false for blocking client requests
fetched: 2026-03-28T09:33:19Z
official_docs: https://docs.rs/crate/reqwest/0.13.2
---

## What is required

- With `default-features = false`, reqwest does **not** keep its default TLS backend unless you re-enable one explicitly.
- For `reqwest::blocking` HTTPS requests, you need:
  - `blocking`, and
  - **one TLS backend feature**: `rustls`, `native-tls`, `native-tls-vendored`, `native-tls-no-alpn`, `native-tls-vendored-no-alpn`, or `default-tls`.

## Why HTTPS breaks without a TLS backend

- In reqwest 0.13.x, the no-TLS path builds a plain `HttpConnector`.
- reqwest only calls `http.enforce_http(false)` in TLS-enabled connector paths.
- Hyper's `HttpConnector` defaults to `enforce_http(true)`, so `https://...` is rejected.

## Common runtime symptom

- Building a blocking client can still succeed, but sending a request to an `https://` URL fails at runtime.
- The underlying connector error is commonly:
  - `invalid URL, scheme is not http`
- In reqwest this surfaces as a request/connect error from the blocking client request path.

## Minimal Cargo.toml fix

Prefer explicit rustls when you have `default-features = false`:

```toml
[dependencies]
reqwest = { version = "0.13", default-features = false, features = ["blocking", "rustls"] }
```

If you want reqwest's current default backend explicitly, this also works:

```toml
[dependencies]
reqwest = { version = "0.13", default-features = false, features = ["blocking", "default-tls"] }
```

## Actionable guidance

- If you use `reqwest::blocking` + `https://` + `default-features = false`, always add a TLS backend feature.
- Use `rustls` for the simplest cross-platform fix.
- Use `native-tls` or `native-tls-vendored` only if you specifically need platform/OpenSSL-backed TLS.

## Official docs used

- Reqwest 0.13.2 crate docs: https://docs.rs/crate/reqwest/0.13.2
- Reqwest 0.13.2 feature flags: https://docs.rs/crate/reqwest/0.13.2/features
- Reqwest 0.13.2 blocking module: https://docs.rs/reqwest/0.13.2/reqwest/blocking/
- Reqwest 0.13.2 blocking ClientBuilder: https://docs.rs/reqwest/0.13.2/reqwest/blocking/struct.ClientBuilder.html
- Reqwest 0.13.2 TLS docs/source: https://docs.rs/crate/reqwest/0.13.2/source/src/tls.rs
- Reqwest 0.13.2 Cargo features source: https://docs.rs/crate/reqwest/0.13.2/source/Cargo.toml.orig
- Hyper-util HttpConnector docs: https://docs.rs/hyper-util/latest/hyper_util/client/legacy/connect/struct.HttpConnector.html
