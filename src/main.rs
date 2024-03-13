use std::rc::Rc;

use arguments::CommandLineArguments;
use clap::Parser;
use graph::{to_graph, ResolutionNode};
use input::parse_input_csv;
use plotting::plot_branch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parses command line arguments.
    let cl_args = CommandLineArguments::parse();
    let input_file = cl_args.csv_file();
    let output_dir = cl_args.output_directory();

    // Builds the cluster stability graph.
    let resolution_data = parse_input_csv(input_file)?;
    let result_graph = to_graph(resolution_data);
    let top_branch: Vec<Rc<ResolutionNode>> = result_graph
        .iter()
        .max_by(|a, b| {
            a.total_stability()
                .partial_cmp(&b.total_stability())
                .expect("There must only be valid stabilities.")
        })
        .map(ResolutionNode::branch)
        .unwrap_or(Vec::new());

    // Plots the top branch
    let output_graph_name = if let Some(file_name) = input_file.file_name() {
        format!("stability_graph_{}.svg", file_name.to_string_lossy())
    } else {
        "stability_graph_unknown_sample.svg".to_string()
    };
    let output_graph_path = output_dir.join(output_graph_name);
    plot_branch(&top_branch, output_graph_path)?;

    Ok(())
}

mod arguments;
mod data;
mod graph;
mod input;
mod optimisation;
mod plotting;
