/*
Copyright 2024 Owain Davies
SPDX-License-Identifier: Apache-2.0 OR MIT
*/

use crate::{Bench, BenchFnArg, BenchFnNamed};
use std::sync::Arc;

/// Error type for `BenchBuilder`.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BenchBuilderError {
    /// Indicates that the number of repetitions is set to zero.
    #[error("Repetitions must be greater than 0.")]
    ZeroRepetitions,

    /// Indicates that the sizes vector is empty.
    #[error("The sizes vector must not be empty.")]
    NoSizes,

    /// Indicates that the functions vector is empty.
    #[error("The functions vector must not be empty.")]
    NoFunctions,
}

/// Builder for creating a `Bench` instance.
pub struct BenchBuilder<'a, T, R> {
    functions: Vec<BenchFnNamed<'a, T, R>>,
    argfunc: BenchFnArg<T>,
    sizes: Vec<usize>,
    repetitions: usize,
    parallel: bool,
    assert_equal: bool,
}

impl<'a, T, R> BenchBuilder<'a, T, R> {
    /// Creates a new `BenchBuilder` with required parameters.
    ///
    /// Mandatory parameters are required upfront and optional parameters are
    /// configured through method chaining.
    ///
    /// By default, `repetitions` is set to 1, `parallel` to false, and
    /// `assert_equal` to false.
    pub fn new(
        functions: Vec<BenchFnNamed<'a, T, R>>,
        argfunc: BenchFnArg<T>,
        sizes: Vec<usize>,
    ) -> Self {
        Self {
            functions,
            argfunc,
            sizes,
            repetitions: 1,
            parallel: false,
            assert_equal: false,
        }
    }

    /// Sets the number of times to time each (input size, function) pair.
    ///
    /// For each (input size, function) pair, the function is timed
    /// `repetitions` times and the average over the repetitions is used as the
    /// benchmark value.
    ///
    /// **Default**: `1`.
    pub fn repetitions(mut self, repetitions: usize) -> Self {
        self.repetitions = repetitions;
        self
    }

    /// Sets whether to run (input size, function) pair benchmarks in parallel.
    ///
    /// **Default**: `false`.
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Sets whether to assert that all function return values are equal.
    ///
    /// When set to `true`, if there exists an input size such that the function
    /// return values are not equal, then the program panics.
    ///
    /// If `repetitions` is greater than 1, then for each input size, only the
    /// function return values from the last repetition are compared.
    ///
    /// **Default**: `false`.
    pub fn assert_equal(mut self, assert_equal: bool) -> Self {
        self.assert_equal = assert_equal;
        self
    }

    /// Validates the configuration and builds a `Bench` instance.
    pub fn build(self) -> Result<Bench<'a, T, R>, BenchBuilderError> {
        if self.repetitions == 0 {
            return Err(BenchBuilderError::ZeroRepetitions);
        }
        if self.sizes.is_empty() {
            return Err(BenchBuilderError::NoSizes);
        }
        if self.functions.is_empty() {
            return Err(BenchBuilderError::NoFunctions);
        }
        Ok(Bench {
            functions: self
                .functions
                .into_iter()
                .map(|(func, name)| (Arc::new(func), name))
                .collect(),
            argfunc: Arc::new(self.argfunc),
            sizes: self.sizes,
            repetitions: self.repetitions,
            parallel: self.parallel,
            assert_equal: self.assert_equal,
            data: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_bench_fn(_: usize) -> usize {
        0
    }

    fn dummy_arg_fn(size: usize) -> usize {
        size
    }

    fn create_mandatory_args() -> (
        Vec<BenchFnNamed<'static, usize, usize>>,
        BenchFnArg<usize>,
        Vec<usize>,
    ) {
        let functions: Vec<BenchFnNamed<'static, usize, usize>> =
            vec![(Box::new(dummy_bench_fn), "Dummy Function")];
        let argfunc: BenchFnArg<usize> = Box::new(dummy_arg_fn);
        let sizes = vec![10, 20, 30];

        (functions, argfunc, sizes)
    }

    #[test]
    fn test_bench_builder_only_mandatory_args() {
        let (functions, argfunc, sizes) = create_mandatory_args();

        let builder = BenchBuilder::new(functions, argfunc, sizes);
        let result = builder.build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_setting_repetitions() {
        let (functions, argfunc, sizes) = create_mandatory_args();

        let builder =
            BenchBuilder::new(functions, argfunc, sizes).repetitions(8);
        let bench = builder.build().unwrap();

        assert_eq!(bench.repetitions, 8);
    }

    #[test]
    fn test_setting_parallel() {
        let (functions, argfunc, sizes) = create_mandatory_args();

        let builder =
            BenchBuilder::new(functions, argfunc, sizes).parallel(true);
        let bench = builder.build().unwrap();

        assert!(bench.parallel);
    }

    #[test]
    fn test_assert_equal() {
        let (functions, argfunc, sizes) = create_mandatory_args();

        let builder =
            BenchBuilder::new(functions, argfunc, sizes).assert_equal(true);
        let bench = builder.build().unwrap();

        assert!(bench.assert_equal);
    }

    #[test]
    fn test_zero_repetitions() {
        let (functions, argfunc, sizes) = create_mandatory_args();

        let builder =
            BenchBuilder::new(functions, argfunc, sizes).repetitions(0);
        let result = builder.build();

        assert!(matches!(result, Err(BenchBuilderError::ZeroRepetitions)));
    }

    #[test]
    fn test_no_sizes() {
        let functions: Vec<BenchFnNamed<'static, usize, usize>> =
            vec![(Box::new(dummy_bench_fn), "Dummy Function")];
        let argfunc: BenchFnArg<usize> = Box::new(dummy_arg_fn);

        let builder = BenchBuilder::new(functions, argfunc, Vec::new());
        let result = builder.build();

        assert!(matches!(result, Err(BenchBuilderError::NoSizes)));
    }

    #[test]
    fn test_no_functions() {
        let functions: Vec<BenchFnNamed<'static, usize, usize>> = Vec::new();
        let argfunc: BenchFnArg<usize> = Box::new(dummy_arg_fn);
        let sizes = vec![10, 20, 30];

        let builder = BenchBuilder::new(functions, argfunc, sizes);
        let result = builder.build();

        assert!(matches!(result, Err(BenchBuilderError::NoFunctions)));
    }
}
