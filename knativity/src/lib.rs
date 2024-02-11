#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cognitive_complexity
)]
#![warn(missing_docs)]
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
//! # Roadmap
//! - [ ] Serving
//! - [ ] Eventing

pub mod eventing;
pub mod serving;
