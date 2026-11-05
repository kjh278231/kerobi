//! 04/05 LP engines. Ported stage by stage against the simplex-state oracle (docs/m1_notes.md).
#![allow(clippy::needless_range_loop)] // numeric kernels index several parallel arrays by the same position

pub mod fp;
pub mod lu;
pub mod scaling;
pub mod simplex;
pub mod spx;
pub mod start;
