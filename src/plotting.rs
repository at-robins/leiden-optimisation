//! This module handles plotting of cluster stability data.

use std::{path::Path, rc::Rc};

use crate::{graph::ResolutionNode, optimisation::ClusterStabilityRegression};

use plotters::prelude::*;

/// The factor a plot axis is extended beyond the maximum value.
const AXIS_EXTENSION: f32 = 1.03;
/// The default y axis maximum.
const AXIS_Y_DEFAULT: f32 = 100.0;
/// How many individual points are used to draw the regression line.
const PLOTTING_RESOLUTION_STEPS_REGRESSION: usize = 1000;

/// Plots the specified branch of a stability graph as SVG.
///
/// # Parameters
///
/// * `branch` - the branch to plot
/// * `plot_path` - the file path to save the plot to
pub fn plot_branch<P: AsRef<Path>>(
    branch: &[Rc<ResolutionNode>],
    plot_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let regression = ClusterStabilityRegression::new(&branch);

    let max_x = branch
        .iter()
        // .flat_map(|branch| branch.iter())
        .map(|node| node.number_of_clusters())
        .max()
        .map(|clusters| clusters as f32 * AXIS_EXTENSION)
        .unwrap_or(AXIS_Y_DEFAULT * AXIS_EXTENSION);

    let root = SVGBackend::new(plot_path.as_ref(), (1800, 1200)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Test", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..(max_x as f32), 0f32..(1.0f32 * AXIS_EXTENSION))?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        branch
            .iter()
            .filter_map(|node| {
                node.optimal_stability()
                    .map(|s| (node.number_of_clusters(), s))
            })
            .map(|(n, s)| (n as f32, s as f32)),
        &BLACK,
    ))?;

    chart.draw_series(LineSeries::new(
        (0..=PLOTTING_RESOLUTION_STEPS_REGRESSION)
            .map(|x| (x as f64 / PLOTTING_RESOLUTION_STEPS_REGRESSION as f64) * max_x as f64)
            .map(|x| (x as f32, regression.predict(x) as f32)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}
