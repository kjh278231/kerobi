//! 04/05 LP engines. Ported stage by stage against the simplex-state oracle (docs/m1/m1_notes.md).
#![allow(clippy::needless_range_loop)] // numeric kernels index several parallel arrays by the same position

pub mod bfrt;
pub mod common;
pub mod fp;
pub mod lu;
pub mod pricing;
pub mod scaling;
pub mod simplex;
pub mod spx;
pub mod presolve;
pub mod start;
pub mod barrier;
pub mod pdhg;
pub mod crossover;
pub mod crossover_backend;
pub mod crossover_btran;
pub mod crossover_factor;
pub mod crossover_ftran;
pub mod crossover_init;
pub mod crossover_loop;
pub mod crossover_lu;
pub mod crossover_recompute;
pub mod crossover_refactor;
pub mod crossover_round;
pub mod crossover_supplier;
pub mod work;
