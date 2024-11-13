/*
Copyright 2024 Owain Davies
SPDX-License-Identifier: Apache-2.0 OR MIT
*/

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod bench;
mod util;

pub use bench::{
    Bench, BenchBuilder, BenchBuilderError, BenchFn, BenchFnArg, BenchFnNamed,
    PlotBuilder, PlotBuilderError,
};
