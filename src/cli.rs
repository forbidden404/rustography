use clap::{Args, Parser, Subcommand};
use serde::Serialize;

#[derive(Debug, Parser)]
#[clap(name = "rustography", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Image manipulation submodule
    Image(ImageArgs),
    /// Social media caption submodule
    Caption(CaptionArgs),
}

#[derive(Debug, Args)]
pub struct ImageArgs {
    /// The input file
    #[clap(long, short)]
    pub input: std::path::PathBuf,

    /// The output file
    #[clap(long, short)]
    pub output: Option<std::path::PathBuf>,

    /// Add a border to the image
    #[clap(long, short = 'a')]
    pub add_border: Option<Option<usize>>,

    /// Fill to a certain aspect ratio (default 1.0 1.0)
    #[clap(long, short = 'f', num_args = 2)]
    pub fill_to_aspect_ratio: Option<Vec<f32>>,

    /// Resize so the longest side has a given value
    #[clap(long, short = 'l')]
    pub longest_side: Option<Option<usize>>,
}

#[derive(Debug, Args)]
pub struct CaptionArgs {
    /// Camera used
    #[arg(short = 'c', long)]
    pub camera: String,

    /// Title of the caption (optional)
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// film used (optional)
    #[arg(short = 'f', long)]
    pub film: Option<String>,

    /// Type of the film (e.g., color, black-and-white)
    #[arg(short = 'T', long)]
    pub film_type: Option<FilmType>,

    /// Lab used (optional)
    #[arg(short = 'l', long)]
    pub lab: Option<String>,

    /// Format used (optional)
    #[arg(short = 'F', long)]
    pub format: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilmType {
    /// Color film
    #[default]
    Color,
    /// Black & White film
    BlackAndWhite,
    /// Lomography Color film
    LomographyColor,
    /// Lomography Black & White film
    LomographyBlackAndWhite,
}
