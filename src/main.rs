use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use image::io::Reader;
use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a white border to an image
    AddBorder,
    /// Fill the image with white to fit a given aspect ratio
    #[command(arg_required_else_help = true)]
    FillToAspectRatio { width: u32, height: u32 },
    /// Fill the image with white to fit a given aspect ratio and add white border
    #[command(arg_required_else_help = true)]
    FillToAspectRatioWithBorder { width: u32, height: u32 },
    /// Add text on the bottom with camera, focal length, aperture, shutter speed and ISO
    #[command(arg_required_else_help = true)]
    AddCaption {
        camera: String,
        focal_length: String,
        aperture: String,
        shutter_speed: String,
        iso: String,
    },
    /// Fill the image with white to fit a given aspect ratio, add white border and add
    /// text on the bottom with camera, focal length, aperture, shutter speed and ISO
    #[command(arg_required_else_help = true)]
    FillToAspectRatioWithBorderAndCaption {
        width: u32,
        height: u32,
        camera: String,
        focal_length: String,
        aperture: String,
        shutter_speed: String,
        iso: String,
    },
    /// Add instagram post content to clipboard
    #[command(arg_required_else_help = true)]
    InstagramCaption {
        camera: String,
        film: String,
        #[clap(default_value_t, value_enum)]
        film_type: FilmType,
        #[clap(default_value = "@nanni_lab")]
        lab: String,
        #[clap(default_value = ".")]
        title: String,
    },
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ImageMode {
    /// ImageMagick
    #[default]
    ImageMagick,
    /// image::io
    Native,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum FilmType {
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

/// Execute a command for a specific image
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The command to execute
    #[command(subcommand)]
    command: Commands,
    /// The path to the image
    #[arg(short, long)]
    path: std::path::PathBuf,
    /// The image mode used
    #[arg(short, long, default_value_t, value_enum)]
    image_mode: ImageMode,
}

fn get_image_dimensions(file_path: &std::path::PathBuf) -> Result<(u32, u32)> {
    let reader = Reader::open(file_path)?;
    let dimensions = reader.into_dimensions()?;
    Ok(dimensions)
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::AddBorder => match get_image_dimensions(&args.path) {
            Ok((width, height)) => {
                add_border(width, height, &args.path);
            }
            Err(e) => println!("error: {}", e),
        },
        Commands::FillToAspectRatio { width, height } => match get_image_dimensions(&args.path) {
            Ok((cur_width, cur_height)) => {
                fill_to_aspect_ratio(width, height, cur_width, cur_height, &args.path);
            }
            Err(e) => println!("error: {}", e),
        },
        Commands::FillToAspectRatioWithBorder { width, height } => {
            match get_image_dimensions(&args.path) {
                Ok((cur_width, cur_height)) => {
                    let old_file =
                        fill_to_aspect_ratio(width, height, cur_width, cur_height, &args.path);
                    add_border(width, height, &old_file);
                    if old_file != args.path {
                        std::fs::remove_file(old_file).expect("Unable to delete old file.");
                    }
                }
                Err(e) => println!("error: {}", e),
            }
        }
        Commands::AddCaption {
            camera,
            focal_length,
            aperture,
            shutter_speed,
            iso,
        } => match get_image_dimensions(&args.path) {
            Ok((width, _)) => {
                add_caption(
                    &camera,
                    &focal_length,
                    &aperture,
                    &shutter_speed,
                    &iso,
                    width,
                    &args.path,
                );
            }
            Err(e) => println!("error: {}", e),
        },
        Commands::FillToAspectRatioWithBorderAndCaption {
            width,
            height,
            camera,
            focal_length,
            aperture,
            shutter_speed,
            iso,
        } => {
            let bordered_file = match get_image_dimensions(&args.path) {
                Ok((cur_width, cur_height)) => add_border(cur_width, cur_height, &args.path),
                Err(e) => panic!("error: {}", e),
            };
            let captioned_file = match get_image_dimensions(&bordered_file) {
                Ok((cur_width, _)) => add_caption(
                    &camera,
                    &focal_length,
                    &aperture,
                    &shutter_speed,
                    &iso,
                    cur_width,
                    &bordered_file,
                ),
                Err(e) => panic!("error: {}", e),
            };
            match get_image_dimensions(&captioned_file) {
                Ok((cur_width, cur_height)) => {
                    let new_file =
                        fill_to_aspect_ratio(width, height, cur_width, cur_height, &captioned_file);
                    if bordered_file != args.path && bordered_file != new_file {
                        std::fs::remove_file(bordered_file).expect("Unable to delete old file.");
                    }
                    if captioned_file != args.path && captioned_file != new_file {
                        std::fs::remove_file(captioned_file).expect("Unable to delete old file.");
                    }
                }
                Err(e) => println!("error: {}", e),
            }
        }
        Commands::InstagramCaption {
            camera,
            film,
            film_type,
            lab,
            title,
        } => {
            let mut text = String::new();
            text.push_str(&format!("{}\n\n", title));
            text.push_str(&format!("📸 {}\n", camera));
            text.push_str(&format!("🎞️ {}\n", film));
            text.push_str(&format!("🧪 {}\n\n", lab));
            text.push_str(&hashtags_by_film(&film, &film_type, &camera));

            let mut ctx = ClipboardContext::new().expect("Could not create a clipboard provider.");
            _ = ctx.set_contents(text);
        }
    }
}

fn hashtags_by_film(film: &str, film_type: &FilmType, camera: &str) -> String {
    let mut hashtags = String::new();
    match film_type {
        FilmType::Color | FilmType::LomographyColor => {
            hashtags.push_str("#35mm #colorFilm #filmPhotography #analogPhotography #filmIsNotDead #iStillShootFilm #shootFilm #filmCommunity #filmLovers #colorFilmPhotography #35mmFilm #filmShooter #analogLove #filmLife #analogVibes #analogLove");
        }
        FilmType::BlackAndWhite | FilmType::LomographyBlackAndWhite => {
            hashtags.push_str("#35mm #blackAndWhitePhotography #BWPhotography #analogPhotography #filmPhotography #classicBW #filmIsNotDead #shootFilm #iStillShootFilm #filmCommunity #BWFilm #BWFilmPhotography #filmLovers #monochromePhotography #35mmFilm #filmShooter #BlackAndWhiteFilm #analogLove #filmLife");
        }
    };

    match film_type {
        FilmType::LomographyBlackAndWhite | FilmType::LomographyColor => {
            hashtags.push_str(" #HeyLomography")
        }
        _ => {}
    };

    accumulate_slices(film, ' ')
        .iter_mut()
        .for_each(|str| hashtags.push_str(&format!(" #{}", str)));

    accumulate_slices(camera, ' ')
        .iter_mut()
        .for_each(|str| hashtags.push_str(&format!(" #{}", str)));

    hashtags
}

fn accumulate_slices(input: &str, separator: char) -> Vec<String> {
    let slices: Vec<&str> = input.split(separator).collect();
    let mut result = Vec::new();
    let mut current = String::new();

    for slice in slices {
        if slice.starts_with('(') {
            continue;
        }
        let mut initial_index = 0;
        if slice.starts_with('@') {
            initial_index = 1;
        }
        current.push_str(&slice[initial_index..]);
        result.push(current.clone());
    }

    result
}

fn add_border(width: u32, height: u32, path: &std::path::PathBuf) -> std::path::PathBuf {
    let op = if width < height { "5x%" } else { "x5%" };
    let file_name = format!(
        "{}-border.{}",
        path.file_stem()
            .expect("Couldn't get file_stem")
            .to_str()
            .expect("Couldn't turn file_stem to string"),
        path.extension()
            .expect("Couldn't get the extension.")
            .to_str()
            .expect("Couldn't turn extension to string")
    );
    let mut new_path = path.to_owned();
    new_path.set_file_name(file_name);

    let cmd = std::process::Command::new("magick")
        .arg(path.to_str().expect("Couldn't turn path to string"))
        .args(["-bordercolor", "white"])
        .args(["-border", op])
        .arg(&new_path)
        .output()
        .expect("failed to execute process");
    println!("status: {}", cmd.status);
    io::stdout().write_all(&cmd.stdout).unwrap();
    io::stderr().write_all(&cmd.stderr).unwrap();

    new_path
}

fn fill_to_aspect_ratio(
    width: u32,
    height: u32,
    cur_width: u32,
    cur_height: u32,
    path: &std::path::PathBuf,
) -> std::path::PathBuf {
    let op = if cur_width < cur_height {
        let expected_width = (cur_height * width) / height;
        let fill_size: i64 = (expected_width as i64 - cur_width as i64) / 2;
        if fill_size <= 0 {
            return path.to_owned();
        }
        format!("{}x", fill_size)
    } else {
        let expected_height = (cur_width * height) / width;
        let fill_size: i64 = (expected_height as i64 - cur_height as i64) / 2;
        if fill_size <= 0 {
            return path.to_owned();
        }
        format!("x{}", fill_size)
    };

    let file_name = format!(
        "{}-filled.{}",
        path.file_stem()
            .expect("Couldn't get file_stem")
            .to_str()
            .expect("Couldn't turn file_stem to string"),
        path.extension()
            .expect("Couldn't get the extension.")
            .to_str()
            .expect("Couldn't turn extension to string")
    );
    let mut new_path = path.to_owned();
    new_path.set_file_name(file_name);

    let cmd = std::process::Command::new("magick")
        .arg(path.to_str().expect("Couldn't turn path to string"))
        .args(["-bordercolor", "white"])
        .args(["-border", &op])
        .arg(&new_path)
        .output()
        .expect("failed to execute process");

    println!("status: {}", cmd.status);
    io::stdout().write_all(&cmd.stdout).unwrap();
    io::stderr().write_all(&cmd.stderr).unwrap();

    new_path
}

fn add_caption(
    camera: &str,
    focal_length: &str,
    aperture: &str,
    shutter_speed: &str,
    iso: &str,
    width: u32,
    path: &std::path::PathBuf,
) -> std::path::PathBuf {
    let file_name = format!(
        "{}-captioned.{}",
        path.file_stem()
            .expect("Couldn't get file_stem")
            .to_str()
            .expect("Couldn't turn file_stem to string"),
        path.extension()
            .expect("Couldn't get the extension.")
            .to_str()
            .expect("Couldn't turn extension to string")
    );
    let mut new_path = path.to_owned();
    new_path.set_file_name(file_name);

    let label = format!(
        "{}\n{}mm  f{}  {}s  ISO{}",
        camera, focal_length, aperture, shutter_speed, iso
    );

    let cmd = std::process::Command::new("magick")
        .arg(path.to_str().expect("Couldn't turn path to string"))
        .args(["-background", "white"])
        .args(["-fill", "grey25"])
        .args(["-font", "SF-Compact-Rounded"])
        .args(["-kerning", "1"])
        .args(["-interline-spacing", "5"])
        .args(["-gravity", "Center"])
        .args(["-size", &format!("{}x{}", width / 3, width / 10)])
        .arg(format!("label:{}", label))
        .args(["-smush", "-100"])
        .arg(&new_path)
        .output()
        .expect("failed to execute process");

    println!("status: {}", cmd.status);
    io::stdout().write_all(&cmd.stdout).unwrap();
    io::stderr().write_all(&cmd.stderr).unwrap();

    new_path
}
