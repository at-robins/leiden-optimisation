use std::path::PathBuf;

use clap::Parser;
use getset::{CopyGetters, Getters};

/// A tool for optimising the resolution parameter of the Leiden clustering algorithm.
#[derive(Parser, CopyGetters, Getters, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The path to the CSV file (UTF-8 encoded, comma delimeted) containing the clustering information.
    #[getset(get = "pub")]
    csv_file: PathBuf,
    /// The output directory [default: the parent directory of the input CSV]
    #[arg(short, long)]
    output_directory: Option<PathBuf>,
    /// The threashold used to compute the optimal clustering resolution.
    #[getset(get_copy = "pub")]
    #[arg(short, long, default_value_t = 0.95)]
    stability_threashold: f64,
}

impl CommandLineArguments {
    /// Returns the output directory.
    /// If no directory has been specified the parent directory of the input file is returned.
    pub fn output_directory(&self) -> PathBuf {
        self.output_directory.as_ref().map(|output_dir| output_dir.to_path_buf()).unwrap_or_else(|| self.csv_file_parent_directory())
    }

    /// Returns the directory that contains the input CSV file.
    fn csv_file_parent_directory(&self) -> PathBuf {
        self.csv_file.parent().map(|parent| parent.to_path_buf()).unwrap_or("/".into())
    }
}
