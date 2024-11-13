/*
Copyright 2024 Owain Davies
SPDX-License-Identifier: Apache-2.0 OR MIT
*/

mod builder;
mod plot;

pub use builder::{BenchBuilder, BenchBuilderError};
pub use plot::{PlotBuilder, PlotBuilderError};

use crate::util;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

/// Type alias for a function to benchmark that takes an argument of type `T`
/// and returns a result of type `R`.
pub type BenchFn<T, R> = Box<dyn Fn(T) -> R + Send + Sync>;

/// Type alias for a tuple containing a `BenchFn` and a name.
pub type BenchFnNamed<'a, T, R> = (BenchFn<T, R>, &'a str);

/// Type alias for a function accepting a positive integer size and returning
/// input for the benchmarking functions.
pub type BenchFnArg<T> = Box<dyn Fn(usize) -> T + Send + Sync>;

/// A structure for benchmarking functions over various input sizes and plotting
/// the results.
pub struct Bench<'a, T, R> {
    functions: Vec<(Arc<BenchFn<T, R>>, &'a str)>,
    argfunc: Arc<BenchFnArg<T>>,
    sizes: Vec<usize>,
    repetitions: usize,
    parallel: bool,
    assert_equal: bool,

    data: Vec<(usize, Vec<f64>)>,
}

type FunctionResult<R> = (R, f64);
type FunctionMultipleResult<R> = (R, Vec<f64>, f64);

impl<
        'a,
        T: Clone + Send + Sync + 'static,
        R: Clone + Send + Debug + PartialEq + 'static,
    > Bench<'a, T, R>
{
    #[allow(dead_code)]
    fn new(
        functions: Vec<(Arc<BenchFn<T, R>>, &'a str)>,
        argfunc: Arc<BenchFnArg<T>>,
        sizes: Vec<usize>,
        repetitions: usize,
        parallel: bool,
        assert_equal: bool,
    ) -> Self {
        Self {
            functions,
            argfunc,
            sizes,
            repetitions,
            parallel,
            assert_equal,
            data: Vec::new(),
        }
    }

    /// Executes all benchmarks.
    ///
    /// The function either runs benchmarks sequentially or in parallel based on
    /// the `parallel` flag.
    pub fn run(&mut self) -> &mut Self {
        if self.parallel {
            self.run_parallel();
        } else {
            self.run_sequential();
        }
        self
    }

    /// Times each `(input size, function)` pair sequentially.
    fn run_sequential(&mut self) {
        for &size in &self.sizes {
            let arg = (self.argfunc)(size);
            let results: Vec<FunctionMultipleResult<R>> =
                Self::time_functions(arg, &self.functions, self.repetitions);

            if self.assert_equal {
                assert!(util::all_items_equal(
                    results.iter().map(|(result, _, _)| result)
                ));
            }

            let execution_times: Vec<f64> =
                results.iter().map(|(_, _, avg)| *avg).collect();
            self.data.push((size, execution_times));
        }
    }

    /// Times `(input size, function)` pairs in parallel.
    fn run_parallel(&mut self) {
        use rayon::prelude::*;

        let size_args: Vec<_> = self
            .sizes
            .iter()
            .enumerate()
            .map(|(size_idx, &size)| {
                let arg = (self.argfunc)(size);
                (size_idx, size, arg)
            })
            .collect();

        let results_and_times: Vec<_> = size_args
            .par_iter()
            .flat_map(|&(size_idx, size, ref arg)| {
                let repetitions = self.repetitions;
                self.functions.par_iter().enumerate().map_with(
                    arg.clone(),
                    move |arg_clone, (func_idx, (func, _))| {
                        let (last_result, _times, avg_time) =
                            Self::time_function_multiple_times(
                                func,
                                arg_clone.clone(),
                                repetitions,
                            );

                        ((size_idx, func_idx), (size, (last_result, avg_time)))
                    },
                )
            })
            .collect();

        let mut results_by_size: HashMap<usize, Vec<R>> = HashMap::new();

        for ((_size_idx, func_idx), (size, (result, avg_time))) in
            results_and_times
        {
            results_by_size.entry(size).or_default().push(result);

            #[cfg(debug_assertions)]
            {
                println!(
                    "size index: {}, function index: {}",
                    _size_idx, func_idx
                );
            }

            if let Some((_, times)) =
                self.data.iter_mut().find(|(s, _)| *s == size)
            {
                times[func_idx] = avg_time;
            } else {
                let mut times = vec![0.0; self.functions.len()];
                times[func_idx] = avg_time;
                self.data.push((size, times));
            }
        }

        // Sort self.data by size_idx
        // TODO: not needed?
        self.data.sort_by(|a, b| a.0.cmp(&b.0));

        if self.assert_equal {
            for results in results_by_size.values() {
                assert!(util::all_items_equal(results));
            }
        }
    }

    /// Times the function once, returning a tuple containing the value returned
    /// by the function and the timing.
    fn time_function(func: &Arc<BenchFn<T, R>>, arg: T) -> FunctionResult<R> {
        let start = Instant::now();
        let result = func(arg);
        let duration = start.elapsed().as_secs_f64();
        (result, duration)
    }

    /// Times the function `n` times, returning a tuple containing the last
    /// return value of the function, the timings, and the average time.
    fn time_function_multiple_times(
        func: &Arc<BenchFn<T, R>>,
        arg: T,
        n: usize,
    ) -> FunctionMultipleResult<R> {
        let mut total_time = 0.0;
        let mut times = Vec::new();
        let mut last_result = None;

        for _ in 0..n {
            let (result, time) = Self::time_function(func, arg.clone());
            last_result = Some(result);

            total_time += time;
            times.push(time);
        }

        (last_result.unwrap(), times, total_time / n as f64)
    }

    /// Times each function `n` times, returning a vector of tuples containing
    /// the last return value of the function, the timings, and the average
    /// time.
    fn time_functions(
        arg: T,
        functions: &[(Arc<BenchFn<T, R>>, &str)],
        repetitions: usize,
    ) -> Vec<FunctionMultipleResult<R>> {
        functions
            .iter()
            .map(|(func, _name)| {
                Self::time_function_multiple_times(
                    func,
                    arg.clone(),
                    repetitions,
                )
            })
            .collect()
    }
}
