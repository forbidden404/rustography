use anyhow::Result;
use clap::{Parser, Subcommand};
use image::io::Reader;
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
    }
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

    let cmd = std::process::Command::new("convert")
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

    let cmd = std::process::Command::new("convert")
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

    let cmd = std::process::Command::new("convert")
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
