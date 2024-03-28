#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cognitive_complexity
)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, allow(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used,))]

//!
//! Unofficial Rust SDK for [Knative](https://knative.dev)
//!
//! This is a placeholder. I'm currently implementing the same functionality closed source. As I
//! progress and get a better grip on the best approach I will start to build this crate in
//! parallel
//!
//! # Versioning
//! Please consider everything `0.0.*` as extremely unstable and absolutely not ready for production use.
//!
//! # Roadmap
//! - [ ] Eventing
//!   - [ ] [`Client`](eventing::Client)
//!     - [x] Sending Events
//!     - [x] Retry with backoff
//!     - [ ] circuit breaker
//!     - [ ] tracing logs
//!     - [ ] opentelemetry
//!   - [ ] Sink. Receiving Events.
//! - [ ] Serving

pub mod eventing;
pub mod serving;
