# Rust Code Quality Audit

## Purpose

Audit the EVMAuth managed service platform Rust codebase for correctness,
idiom, and quality. The bar is production-grade systems Rust written by a
senior engineer. Flag anything that would cause a strong Rust engineer to
lose confidence in the codebase.

Work through every item in every section below. For each finding, provide:
- The file and line range
- A concise description of the problem
- A concrete corrected example

At the end, produce a prioritized summary: **Critical** (correctness or
security), **Major** (idiomatic or maintainability), **Minor** (style or
polish).

---

## Section 1: Error Handling

### 1.1 No `unwrap()` or `expect()` in non-test code

`unwrap()` and `expect()` panic on failure. In an async server context a
panic takes down the entire thread or task. Every call site must either
propagate the error with `?` or handle it explicitly.

- Flag every `unwrap()` and `expect()` outside of `#[cfg(test)]` blocks,
  `#[test]` functions, and `main.rs` startup assertions.
- Exception: `expect()` is acceptable in `main.rs` for configuration loading
  at startup where a panic is the correct behavior (fail fast before serving
  traffic). Flag it anyway with a note that it is intentional.

### 1.2 Error types are specific and carry context

- Each crate/service must define its own error enum using `thiserror::Error`.
- Variants must not be named `Error(String)` or `Other(String)` -- these
  destroy structural information. Flag any such variants.
- `#[from]` conversions must be used for wrapping foreign errors rather than
  manual `map_err(|e| MyError::Something(e.to_string()))`. The latter
  discards the original error type. Flag every `map_err` that converts to a
  string.
- Error messages must be lowercase, no trailing period, and not repeat the
  variant name. Flag violations.

### 1.3 `anyhow` is only used at the boundary

`anyhow::Error` is appropriate in `main.rs` and in handler functions where
multiple unrelated error types converge and the caller only needs to log or
return a 500. It must not appear in library crate public APIs (`crates/evm`,
`crates/postgres`, `crates/redis-client`, `crates/pagination`,
`crates/service-discovery`). Flag any `anyhow::Result` or `anyhow::Error` in
the public API surface of those crates.

### 1.4 The `?` operator is used consistently

Manual `match` blocks on `Result` that replicate what `?` would do are
unnecessary noise. Flag them.

---

## Section 2: Async Correctness

### 2.1 No blocking calls inside async functions

Any call that blocks the thread must not appear inside an `async fn` without
being wrapped in `tokio::task::spawn_blocking`. This includes:

- `std::fs` (use `tokio::fs`)
- `std::thread::sleep` (use `tokio::time::sleep`)
- CPU-bound loops without yield points
- Synchronous HTTP clients (must use `reqwest` async)
- Any `sqlx` query called without `.await`

Flag every `std::fs`, `std::thread::sleep`, and synchronous I/O call in async
context.

### 2.2 `Arc` is used correctly

- `Arc<T>` is for shared ownership across threads. If a value is only used
  within a single async task or passed by reference, `Arc` is unnecessary.
  Flag `Arc` wrapping types that are only ever cloned into a single call site.
- `Arc<Mutex<T>>` where `T` is only read (never mutated) should be
  `Arc<RwLock<T>>` or just `Arc<T>` if `T: Sync`. Flag these.
- `Arc<Mutex<T>>` held across `.await` points will deadlock or cause
  starvation. Flag any `MutexGuard` that is held across an `.await`.
- Prefer `tokio::sync::Mutex` over `std::sync::Mutex` when the lock must be
  held across `.await`. Flag uses of `std::sync::Mutex` in async code.

### 2.3 Tasks are spawned and awaited correctly

- `tokio::spawn` returns a `JoinHandle`. Dropping it without awaiting detaches
  the task silently. Any `tokio::spawn` whose return value is dropped without
  an explicit `drop()` or `let _ =` acknowledgment should be flagged. If
  fire-and-forget is intentional, it must be annotated with a comment.
- Errors from spawned tasks must not be silently discarded. Flag `let _ =
  handle.await` that discards a `Result`.

### 2.4 `select!` branches are cancel-safe

`tokio::select!` cancels all other branches when one completes. If a branch
contains an operation that is not cancel-safe (e.g. writing to a channel,
partially completing a database write), this is a bug. Flag any `select!`
where a branch contains a non-atomic operation that would be incorrect if
cancelled mid-flight.

---

## Section 3: Type Design and API Boundaries

### 3.1 Newtype wrappers for domain primitives

Raw primitives passed around as `String` or `Uuid` lose their semantic
meaning and make it possible to pass a `wallet_address` where an `org_id` is
expected. The following types must be newtype wrappers, not bare `String` or
`Uuid`:

- Wallet addresses: must be a `WalletAddress(String)` newtype that validates
  EIP-55 checksum on construction
- Contract addresses: same
- Transaction hashes: `TxHash(String)`
- Turnkey sub-org IDs: `TurnkeySubOrgId(String)`
- Client IDs: `ClientId(String)`

Flag every function signature in public crate APIs that accepts or returns
bare `String` or `Uuid` where a domain type should be used instead.

### 3.2 Impossible states are unrepresentable

- `Option<Option<T>>` is almost always a design mistake. Flag it.
- Struct fields that are logically always present together should be grouped
  into a nested struct, not left as separate `Option` fields that happen to
  be set at the same time.
- Enums with a `status: String` field that can be `"active"` or `"revoked"`
  should be typed enums. Flag stringly-typed state anywhere in domain types.

### 3.3 Crate boundaries are respected

The architecture is microservices with shared library crates:

```
services/auth      -> crates/evm, crates/postgres, crates/redis-client, crates/pagination
services/wallets   -> crates/evm, crates/postgres, crates/redis-client, turnkey_client SDK
services/registry  -> crates/evm, crates/postgres, crates/redis-client, crates/pagination
services/gateway   -> crates/service-discovery
services/docs      -> crates/service-discovery
services/assets    -> crates/postgres, crates/redis-client, crates/pagination
```

- Service handlers must not contain domain logic -- only routing, extraction,
  response mapping, and middleware.
- Library crates (`crates/*`) must have no knowledge of specific services.
- The `evm` crate must not make network calls that mutate state (it is
  read-only). It may use `alloy` types and read-only provider calls.
- Only the wallets service talks to the Turnkey SDK.

Flag any `use` statement that violates this graph.

### 3.4 Lifetimes are explicit where needed

- If a struct holds a reference, it must have an explicit lifetime annotation.
  Flag any struct with reference fields that lacks lifetime parameters.
- Avoid `'static` bounds unless the value genuinely lives for the program's
  lifetime. Flag `'static` on generic bounds in non-`spawn` contexts.

---

## Section 4: Security-Sensitive Code

### 4.1 Secrets are never logged

Any value that is a private key, API key, secret hash, authorization code
plaintext, or JWT must never appear in a `tracing` macro, `println!`,
`format!`, or `Debug` impl.

- Flag any `derive(Debug)` on structs that contain fields named `*key*`,
  `*secret*`, `*token*`, `*hash*`, `*code*`, or `*password*` without a
  custom `Debug` impl that redacts those fields.
- Flag any `tracing::debug!` or `tracing::info!` that formats a request or
  response body wholesale without field-level selection.

### 4.2 Constant-time comparison for secrets

Comparing secret values with `==` is vulnerable to timing attacks. Any
comparison of:
- HMAC values
- Hash digests (e.g. auth code hashes)
- API key hashes
- Authentication credential references

must use a constant-time comparison function (e.g. `subtle::ConstantTimeEq`
or `ring`'s comparison utilities). Flag every `==` comparison where either
operand is a hash, digest, or encoded secret.

### 4.3 EIP-55 normalization on input

All wallet addresses received from external input (request bodies, query
parameters, Turnkey SDK responses) must be normalized to EIP-55 checksum
format before storage or comparison. Flag any address stored or compared
without explicit checksum normalization.

### 4.4 SQL injection is impossible

`sqlx::query!` and `sqlx::query_as!` macros with `$N` parameter binding are
safe. Flag any use of `format!` to construct a SQL string, or any
`sqlx::query` (non-macro) call that interpolates a variable into the query
string.

### 4.5 The auth code is never stored in plaintext

The `auth_codes` Redis key stores `SHA-256(plaintext_code)` as the key. The
plaintext code must never appear in:
- A log statement
- A database or cache value
- A struct that derives `Debug` without redaction

Flag any code path where the plaintext auth code flows into a storage or
logging operation.

---

## Section 5: Performance and Resource Management

### 5.1 Database connections are pooled correctly

- `sqlx::PgPool` must be created once at startup and passed via `AppState`.
  Flag any code that creates a new pool per request or per connection.
- Long-running transactions must not hold a connection while awaiting an
  external I/O operation (Turnkey SDK call, chain RPC call). Flag any
  `sqlx::Transaction` that is held open across a `reqwest` or `alloy` await.

### 5.2 HTTP clients are not recreated per request

`reqwest::Client` is designed to be cloned and reused -- it maintains a
connection pool internally. Flag any code that calls `reqwest::Client::new()`
inside a request handler or in a loop.

### 5.3 Allocations in hot paths

The `/accounts` endpoint is the performance-critical path. Flag:
- Any `Vec` allocation inside the handler that could be replaced with a fixed-
  size array given that `relevant_token_ids` is bounded
- Any `String` cloning that could be a `&str` borrow
- Any `clone()` call on large structs in the handler path

### 5.4 Retry logic has backoff and a limit

The wallets service uses the official Turnkey Rust SDK (`turnkey_client`),
which handles retry logic internally via `RetryConfig` with exponential
backoff (default: 5 attempts, 500ms initial delay, 2x multiplier, 5s max
delay). Do not add custom retry wrappers around SDK calls.

For non-SDK external calls (e.g., inter-service HTTP via `reqwest`), flag
any retry loop that:
- Has no maximum attempt count
- Has no delay between attempts
- Does not distinguish retryable errors (network timeout, 503) from
  non-retryable errors (400 bad request, 401 unauthorized)

---

## Section 6: Code Clarity and Maintainability

### 6.1 Public items are documented

Every `pub` function, struct, and enum in `crates/evm`,
`crates/postgres`, `crates/redis-client`, `crates/pagination`, and
`crates/service-discovery` must have a `///` doc comment explaining what it
does, not merely restating its name. Flag undocumented public items in these
crates.

### 6.2 `todo!()` and `unimplemented!()` are absent

These panic at runtime. Flag every occurrence outside of test code.

### 6.3 Dead code is absent

Flag any function, struct, or module annotated with `#[allow(dead_code)]`
unless accompanied by a comment explaining why it is retained.

### 6.4 Magic values are named constants

Flag any numeric literal or string literal used directly in logic (not in
tests) that represents a domain concept without a named constant. Examples:
- `30` for the ERC-712 replay window in seconds -> must be
  `const ERC712_REPLAY_WINDOW_SECS: u64 = 30`
- `60` for auth code TTL -> `const AUTH_CODE_TTL_SECS: u64 = 60`
- Token ID values for platform capability tokens
- HD wallet derivation paths

### 6.5 Clippy passes cleanly

The codebase must compile with `cargo clippy -- -D warnings` producing zero
warnings. Flag any pattern that Clippy would warn on, including:
- `needless_return`
- `redundant_clone`
- `map_unwrap_or`
- `match_wildcard_for_single_variants`
- `unused_async` (async fn with no await inside)

---

## Section 7: Turnkey SDK Integration

The wallets service uses the official `turnkey_client` Rust SDK (v0.6) and
`turnkey_api_key_stamper` for P-256 request signing. There is no custom
Turnkey wrapper crate -- the SDK is used directly by the wallets service
handlers.

### 7.1 SDK typed intents and results are used

The SDK provides typed intent structs (e.g., `CreateSubOrganizationIntentV7`,
`CreateWalletIntent`, `SignRawPayloadIntentV2`) and typed result structs
(e.g., `ActivityResult<CreateWalletResult>`). Flag any code that constructs
raw JSON requests to the Turnkey API or deserializes responses via
`serde_json::Value` instead of using the SDK's typed methods.

### 7.2 SDK methods are used for all Turnkey operations

The SDK provides typed methods (e.g., `client.create_sub_organization(...)`,
`client.create_wallet(...)`, `client.sign_raw_payload(...)`) that handle
activity type strings, request stamping, and response parsing internally.
Flag any code that constructs raw Turnkey activity requests by manually
setting a `type` field as a string, or that bypasses the SDK to call the
Turnkey API directly via `reqwest`.

### 7.3 The delegated account policy is verified at provisioning

When creating an org sub-org and provisioning the delegated account, the code
must create a Turnkey policy restricting the delegated account to
`ACTIVITY_TYPE_SIGN_RAW_PAYLOAD` only, and verify the policy was
successfully applied before returning success. Flag any provisioning flow
that creates the delegated account user without creating and asserting the
signing-only policy is in place. Use the SDK's `create_policy` method.

### 7.4 SDK errors are handled, not swallowed

`TurnkeyClientError` has structured variants (`ActivityFailed`,
`ExceededRetries`, `StamperError`, `UnexpectedHttpStatus`, etc.). These must
be propagated or mapped to appropriate API errors. Flag any error path that
discards the SDK error or maps everything to a single opaque string.

---

## Section 8: EVM Crate Specifically

### 8.1 All addresses are validated before use

Any `Address` value received from external input must be validated as a valid
EVM address before being passed to Alloy. Flag any `Address::from_str` call
that does not handle the `Err` case explicitly.

### 8.2 RPC calls have timeouts

Every `alloy` provider call must have an explicit timeout. An RPC node that
stops responding must not hang the request indefinitely. Flag any provider
call without a configured timeout.

### 8.3 The EVM crate is read-only

The `crates/evm` crate provides read-only contract queries and ABI encoding.
It must not contain signing logic or mutable state operations. All signing
goes through the Turnkey SDK in the wallets service. Flag any signing key
types (`LocalSigner`, `PrivateKeySigner`) or transaction-sending logic in
the evm crate.

---

## How to Run This Audit

Apply this audit to the full backend workspace:

```
cd rs
cargo clippy -- -D warnings          # must produce zero warnings
cargo test --workspace               # all tests must pass
cargo audit                          # no known vulnerabilities
```

Then work through each section above file by file, starting with:
1. `services/wallets/src/` -- Turnkey trust boundary + signing
2. `services/auth/src/` -- authentication boundary
3. `services/registry/src/` -- contract management + accounts query
4. `crates/evm/src/` -- external I/O (chain RPC)
5. `crates/postgres/src/`, `crates/redis-client/src/` -- persistence
6. `crates/pagination/src/`, `crates/service-discovery/src/` -- shared libs
7. `services/gateway/src/`, `services/assets/src/`, `services/docs/src/`

Produce findings in this format:

```
[CRITICAL|MAJOR|MINOR] services/<service>/src/<file>.rs:<line>
Problem: <one sentence>
Current:
    <offending code>
Fixed:
    <corrected code>
```
