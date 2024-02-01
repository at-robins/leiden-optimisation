use std::path::PathBuf;

use clap::Parser;
use getset::Getters;

/// A tool for optimising the resolution parameter of the Leiden clustering algorithm.
#[derive(Parser, Getters, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The path to the CSV file (UTF-8 encoded, comma delimeted) containing the clustering information.
    #[getset(get = "pub")]
    csv_file: PathBuf,
}
