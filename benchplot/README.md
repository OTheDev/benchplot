[<img alt="github" src="https://img.shields.io/badge/github-othedev/benchplot-76e8b5?style=for-the-badge&labelColor=24292e&logo=github" height="20">](https://github.com/OTheDev/benchplot)
[![Multi-Platform Test](https://github.com/OTheDev/benchplot/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/OTheDev/benchplot/actions/workflows/test.yml)
[![Static Analysis](https://github.com/OTheDev/benchplot/actions/workflows/static.yml/badge.svg?branch=main)](https://github.com/OTheDev/benchplot/actions/workflows/static.yml)

# benchplot

`benchplot` is a utility for benchmarking functions over various input
sizes and plotting the results.

## Usage

<p align="center">
  <img src="https://github.com/OTheDev/benchplot/raw/main/examples/sorting/output.svg?raw=true" />
</p>

```rust
use benchplot::{BenchBuilder, BenchFnArg, BenchFnNamed};
use rand::Rng;

fn main() {
    // Functions to benchmark (with names)
    let functions: Vec<BenchFnNamed<Vec<i32>, Vec<i32>>> = vec![
        (Box::new(bubble_sort), "Bubble Sort"),
        (Box::new(insertion_sort), "Insertion Sort"),
        (Box::new(merge_sort), "Merge Sort"),
    ];

    // For each size, returns an argument to pass to the functions to benchmark
    let argfunc: BenchFnArg<Vec<i32>> = Box::new(|size: usize| {
        let mut rng = rand::thread_rng();
        (0..size).map(|_| rng.gen_range(1..=1000)).collect()
    });

    // Input sizes to test. NOTE: a wider range was used for the image above.
    let sizes: Vec<usize> = (0..15).map(|k| 1 << k).collect();

    // Build a `Bench` instance
    let mut bench = BenchBuilder::new(functions, argfunc, sizes)
        .repetitions(1)
        .parallel(true)
        .assert_equal(true)
        .build()
        .unwrap();

    // Run benchmarks and plot them
    bench
        .run()
        .plot("output.svg")
        .title("Sorting Algorithms")
        .build()
        .expect("Plotting failed");
}

// Bubble Sort: O(n²)
fn bubble_sort(mut a: Vec<i32>) -> Vec<i32> {
    let mut n = a.len();
    while n > 1 {
        let mut np = 0;
        for i in 1..n {
            if a[i - 1] > a[i] {
                a.swap(i - 1, i);
                np = i;
            }
        }
        n = np;
    }
    a
}

// Insertion Sort: O(n²)
fn insertion_sort(mut a: Vec<i32>) -> Vec<i32> {
    let n = a.len();
    for i in 1..n {
        let x = a[i];
        let mut j = i;
        while j > 0 && a[j - 1] > x {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = x;
    }
    a
}

// Merge Sort: O(n log n)
// Based on code from https://en.wikipedia.org/wiki/Merge_sort
fn merge_sort(mut a: Vec<i32>) -> Vec<i32> {
    let len = a.len();
    let mut b = a.clone();
    top_down_split_merge(&mut a, 0, len, &mut b);
    a
}

fn top_down_split_merge(
    b: &mut [i32],
    begin: usize,
    end: usize,
    a: &mut [i32],
) {
    if end - begin <= 1 {
        return;
    }
    let middle = (end + begin) / 2;
    top_down_split_merge(a, begin, middle, b);
    top_down_split_merge(a, middle, end, b);
    top_down_merge(b, begin, middle, end, a);
}

fn top_down_merge(
    b: &mut [i32],
    begin: usize,
    middle: usize,
    end: usize,
    a: &mut [i32],
) {
    let mut i = begin;
    let mut j = middle;
    for b_k in b.iter_mut().take(end).skip(begin) {
        if i < middle && (j >= end || a[i] <= a[j]) {
            *b_k = a[i];
            i += 1;
        } else {
            *b_k = a[j];
            j += 1;
        }
    }
}
```

## License

This project is dual-licensed under either the [Apache License, Version 2.0](https://github.com/OTheDev/benchplot/blob/main/LICENSE-APACHE)
or the [MIT License](https://github.com/OTheDev/benchplot/blob/main/LICENSE-MIT),
at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you shall be dual-licensed as above, without
any additional terms or conditions.
