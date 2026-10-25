use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Strength {
    /// Minimal perturbation (fastest, smallest delta)
    Low,
    /// Balanced perturbation (recommended for general use)
    Medium,
    /// Aggressive perturbation (disrupts stubborn frequency-domain watermarks)
    High,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "ai-image-cleaner",
    author = "Antigravity",
    version = "0.1.0",
    about = "Sanitizes AI-generated images by stripping metadata, C2PA, and disrupting invisible watermarks"
)]
pub struct Cli {
    /// Input file or directory to process
    #[arg(required = true)]
    pub input: PathBuf,

    /// Output destination: file path (if input is file) or directory (if input is directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite source files in-place (mutually exclusive with --output)
    #[arg(short, long, default_value_t = false)]
    pub in_place: bool,

    /// Enable deep cleaning mode to disrupt invisible watermarks (e.g. SynthID)
    #[arg(short, long, default_value_t = false)]
    pub deep: bool,

    /// Intensity of deep watermark disruption
    #[arg(long, value_enum, default_value_t = Strength::Medium)]
    pub strength: Strength,

    /// Recursively search subdirectories when input is a directory
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    /// Number of worker threads (default: number of logical CPUs)
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Inspect and report AI metadata without writing any files
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Print detailed logs during processing
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
