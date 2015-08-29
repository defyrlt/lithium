#![allow(unused_features)]
#![allow(dead_code)]
#![feature(test)]

//! # lithium
//!
//! lithium is (probably) fast and (hopefully) ergonomic SQL builder.  
//! **Attention**: stuff is not even close to ready and can break in a flash. Also, everything is
//! built on nightly and wasn't checked on stable/beta.
//!
//! As for now, lithium provides interface to create a SQL, but it's not responsible
//! for executing it.
//!
//! You can find examples in documentation for every struct.

pub mod common;
pub mod select;
pub mod where_cl;
pub mod update;
pub mod insert;

#[doc(inline)]
pub use common::{ToSQL, AsStr, Pusheable};
#[doc(inline)]
pub use select::Select;
#[doc(inline)]
pub use insert::Insert;
#[doc(inline)]
pub use update::Update;
#[doc(inline)]
pub use where_cl::Where;
