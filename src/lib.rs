#![allow(unused_features)]
#![allow(dead_code)]
#![feature(test)]

//! # lithium
//!
//! lithium is (probably) fast and (hopefully) convenient to use SQL builder.
//!
//! As for now, lithium provides interface to create a SQL, but it's not responsible
//! for executing it. We may provide an interface to execute generated SQL using rust-postgres.
//!
//! You can find examples in struct defined below.

pub mod common;
pub mod select;
pub mod where_cl;
pub mod update;
pub mod insert;

#[doc(inline)]
pub use common::ToSQL;
#[doc(inline)]
pub use select::{Select, Ordering};
#[doc(inline)]
pub use where_cl::Where;
