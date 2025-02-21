/*
Copyright 2024-2025 Owain Davies
SPDX-License-Identifier: Apache-2.0 OR MIT
*/

use crate::Bench;
use plotters::prelude::full_palette::*;
use plotters::prelude::*;
use plotters::style::{Color, IntoFont, ShapeStyle};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// Colors for each function line. Wrap around if there are more functions.
const COLORS: &[RGBColor] = &[
    RGBColor(121, 192, 255),
    RGBColor(137, 87, 229),
    RGBColor(240, 136, 62),
    RGBColor(218, 54, 51),
    RGBColor(139, 148, 158),
    RGBColor(63, 185, 80),
    RGBColor(255, 215, 0),
    RGBColor(0, 255, 0),
    RGBColor(255, 20, 147),
    RGBColor(138, 43, 226),
    RGBColor(127, 255, 212),
];

/// Error type for `PlotBuilder`.
#[derive(Debug, thiserror::Error)]
pub enum PlotBuilderError {
    /// Represents errors originating from the [`plotters`] crate when
    /// attempting to create a plot.
    #[error("{0}")]
    DrawingError(#[from] DrawingAreaErrorKind<std::io::Error>),
}

impl<'a, T: Clone + Send + 'static, R: Send + 'static> Bench<'a, T, R> {
    /// Returns a builder for generating a plot of the benchmark results and
    /// saving it to a file.
    pub fn plot<P: AsRef<Path>>(
        &'a self,
        filename: P,
    ) -> PlotBuilder<'a, T, R> {
        PlotBuilder::new(self, filename)
    }
}

/// Builder for generating a plot of the benchmark results and saving it to a
/// file.
pub struct PlotBuilder<'a, T, R> {
    bench: &'a Bench<'a, T, R>,
    title: String,
    filename: PathBuf,
}

impl<'a, T: Clone + Send + 'static, R: Send + 'static> PlotBuilder<'a, T, R> {
    /// Creates a new `PlotBuilder` with required parameters.
    ///
    /// Mandatory parameters are required upfront and optional parameters are
    /// configured through method chaining.
    ///
    /// # Parameters
    /// - `bench`: Reference to an instance of `Bench`.
    /// - `filename`: Path of the file to save the plot to.
    pub fn new<P: AsRef<Path>>(
        bench: &'a Bench<'a, T, R>,
        filename: P,
    ) -> Self {
        Self {
            bench,
            title: String::new(),
            filename: filename.as_ref().to_path_buf(),
        }
    }

    /// Sets the title of the plot.
    ///
    /// By default, the `title` is empty.
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Creates a plot of the benchmark results and saves it to a file.
    pub fn build(self) -> Result<(), PlotBuilderError> {
        self.create_plot_and_save()
    }

    fn create_plot_and_save(self) -> Result<(), PlotBuilderError> {
        let root =
            SVGBackend::new(&self.filename, (800, 600)).into_drawing_area();
        root.fill(&RGBColor(255, 255, 255).mix(0.0))?;

        let (min_timing, max_timing) = self
            .bench
            .data
            .iter()
            .flat_map(|(_, timings)| timings.iter().cloned())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), timing| {
                (min.min(timing), max.max(timing))
            });

        let mut chart = ChartBuilder::on(&root)
            .caption(
                textwrap::fill(&self.title, 50),
                ("sans-serif", 24).into_font().color(&GREY.to_rgba()),
            )
            .margin(20)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(
                (self.bench.sizes[0] as f64
                    ..self.bench.sizes[self.bench.sizes.len() - 1] as f64)
                    .log_scale(),
                (min_timing..max_timing).log_scale(),
            )?;

        chart
            .configure_mesh()
            .light_line_style(TRANSPARENT)
            .x_desc("n")
            .y_desc("Time (s)")
            .x_labels(10)
            .y_labels(10)
            .x_label_formatter(&|v| {
                format!("10{}", superscript(v.log10().round() as i32))
            })
            .y_label_formatter(&|v| {
                format!("10{}", superscript(v.log10().round() as i32))
            })
            .axis_style(ShapeStyle {
                color: GREY.mix(0.3).to_rgba(),
                filled: true,
                stroke_width: 1,
            })
            .x_label_style(
                ("sans-serif", 24).into_font().color(&GREY.to_rgba()),
            )
            .y_label_style(
                ("sans-serif", 24).into_font().color(&GREY.to_rgba()),
            )
            .draw()?;

        for (i, &(_, name)) in self.bench.functions.iter().enumerate() {
            let data_series: Vec<(f64, f64)> = self
                .bench
                .data
                .iter()
                .map(|(size, timings)| (*size as f64, timings[i]))
                .collect();

            let style = ShapeStyle {
                color: COLORS[i % COLORS.len()].into(),
                filled: false,
                stroke_width: 2,
            };

            chart
                .draw_series(LineSeries::new(data_series, style))?
                .label(name.to_string())
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], style)
                });
        }

        chart
            .configure_series_labels()
            .background_style(RGBColor(255, 255, 255).mix(0.0))
            .border_style(GREY.to_rgba())
            .label_font(
                ("sans-serif", 18)
                    .into_font()
                    .color(&RGBColor(128, 128, 128)),
            )
            .position(SeriesLabelPosition::UpperLeft)
            .draw()?;

        root.present()?;
        Ok(())
    }
}

#[cfg(test)]
mod plot_tests {
    use super::*;
    use crate::{BenchBuilder, BenchFnArg, BenchFnNamed};
    use std::fs;
    use tempfile::{tempdir, TempDir};

    fn setup_bench_data() -> Bench<'static, usize, usize> {
        let functions: Vec<BenchFnNamed<'static, usize, usize>> = vec![
            (Box::new(|x| x * 2), "Double"),
            (Box::new(|x| x * x), "Square"),
        ];
        let argfunc: BenchFnArg<usize> = Box::new(|x| x);
        let sizes = vec![10, 100, 1000];
        let bench = BenchBuilder::new(functions, argfunc, sizes)
            .build()
            .unwrap();
        bench
    }

    fn get_temp_dir_and_file_path() -> (TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_plot.svg");
        assert!(!file_path.exists());
        (dir, file_path)
    }

    #[test]
    fn test_plot_file_creation() {
        let (_dir, file_path) = get_temp_dir_and_file_path();

        let mut bench = setup_bench_data();
        let plot_result =
            bench.run().plot(&file_path).title("Benchmark Plot").build();

        assert!(plot_result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_plot_with_title() {
        let (_dir, file_path) = get_temp_dir_and_file_path();

        let mut bench = setup_bench_data();
        let plot_result = bench
            .run()
            .plot(&file_path)
            .title("Custom Title for Plot")
            .build();

        assert!(plot_result.is_ok());

        let file_content =
            fs::read_to_string(file_path).expect("Failed to read plot file");

        assert!(file_content.contains("Custom Title for Plot"));
    }
}

pub fn superscript(n: i32) -> String {
    const DIGITS: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹";
    let mut result = String::new();

    if n < 0 {
        result.push('⁻');
    }

    let n_str = n.abs().to_string();
    for c in n_str.chars() {
        if let Some(digit) = c.to_digit(10) {
            result.push(DIGITS.chars().nth(digit as usize).unwrap());
        }
    }

    result
}

#[cfg(test)]
mod superscript_tests {
    use super::*;

    #[test]
    fn test_superscript_single_digit() {
        assert_eq!(superscript(-9), "⁻⁹");
        assert_eq!(superscript(-8), "⁻⁸");
        assert_eq!(superscript(-7), "⁻⁷");
        assert_eq!(superscript(-6), "⁻⁶");
        assert_eq!(superscript(-5), "⁻⁵");
        assert_eq!(superscript(-4), "⁻⁴");
        assert_eq!(superscript(-3), "⁻³");
        assert_eq!(superscript(-2), "⁻²");
        assert_eq!(superscript(-1), "⁻¹");

        assert_eq!(superscript(0), "⁰");

        assert_eq!(superscript(1), "¹");
        assert_eq!(superscript(2), "²");
        assert_eq!(superscript(3), "³");
        assert_eq!(superscript(4), "⁴");
        assert_eq!(superscript(5), "⁵");
        assert_eq!(superscript(6), "⁶");
        assert_eq!(superscript(7), "⁷");
        assert_eq!(superscript(8), "⁸");
        assert_eq!(superscript(9), "⁹");
    }

    #[test]
    fn test_superscript_multi_digit() {
        assert_eq!(superscript(10), "¹⁰");
        assert_eq!(superscript(23), "²³");
        assert_eq!(superscript(45), "⁴⁵");
        assert_eq!(superscript(678), "⁶⁷⁸");
        assert_eq!(superscript(980), "⁹⁸⁰");
        assert_eq!(superscript(1234567890), "¹²³⁴⁵⁶⁷⁸⁹⁰");

        assert_eq!(superscript(-10), "⁻¹⁰");
        assert_eq!(superscript(-234), "⁻²³⁴");
    }
}
