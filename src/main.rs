use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use image::ImageError;

mod cli;
mod image_manipulator;

pub use crate::cli::*;
pub use crate::image_manipulator::*;

fn main() -> Result<(), ImageError> {
    let args = App::parse();

    match args.command {
        Command::Image(image) => {
            let output = image.output.unwrap_or_else(|| image.input.clone());
            let mut image_manipulator = ImageManipulator::new(image.input.clone(), output.clone())?;

            // Handle --add_border
            match image.add_border {
                Some(Some(value)) => image_manipulator = image_manipulator.add_border(value),
                Some(None) => image_manipulator = image_manipulator.add_border(20),
                _ => {}
            }

            // Handle --fill_to_aspect_ratio
            if let Some(values) = image.fill_to_aspect_ratio {
                if values.len() == 2 {
                    image_manipulator =
                        image_manipulator.fill_to_aspect_ratio(values[0], values[1]);
                } else if values.len() == 1 {
                    image_manipulator = image_manipulator.fill_to_aspect_ratio(values[0], 1.0);
                } else {
                    println!(
                        "--fill_to_aspect_ratio expects at most 2 values. Nothing will be done."
                    );
                }
            }

            // Handle --longest_side
            match image.longest_side {
                Some(Some(value)) => image_manipulator = image_manipulator.longest_side(value),
                Some(None) => image_manipulator = image_manipulator.longest_side(1350),
                _ => {}
            }

            image_manipulator.save()?;
        }
        Command::Caption(caption) => {
            let mut text = String::new();
            text.push_str(&format!(
                "{}\n\n",
                caption.title.unwrap_or_else(|| ".".to_string())
            ));

            text.push_str(&format!("📸 {}\n", caption.camera));

            if let Some(film) = caption.film.clone() {
                text.push_str(&format!("🎞️ {}\n", film));
            }

            if let Some(lab) = caption.lab {
                text.push_str(&format!("🧪 {}\n\n", lab));
            }

            if let (Some(film), Some(film_type), Some(format)) =
                (caption.film, caption.film_type, caption.format)
            {
                text.push_str(&hashtags_by_film(
                    &film,
                    &film_type,
                    &caption.camera,
                    &format,
                ));
            }

            let mut ctx = ClipboardContext::new().expect("Could not create a clipboard provider.");
            _ = ctx.set_contents(text.clone());
            println!("{}", text);
        }
    }

    Ok(())
}

fn hashtags_by_film(film: &str, film_type: &FilmType, camera: &str, format: &str) -> String {
    let mut hashtags = String::new();
    match film_type {
        FilmType::Color | FilmType::LomographyColor => {
            hashtags.push_str("#colorFilm #filmPhotography #analogPhotography #filmIsNotDead #iStillShootFilm #shootFilm #filmCommunity #filmLovers #colorFilmPhotography #filmShooter #analogLove #filmLife #analogVibes #analogLove");
        }
        FilmType::BlackAndWhite | FilmType::LomographyBlackAndWhite => {
            hashtags.push_str("#blackAndWhitePhotography #BWPhotography #analogPhotography #filmPhotography #classicBW #filmIsNotDead #shootFilm #iStillShootFilm #filmCommunity #BWFilm #BWFilmPhotography #filmLovers #monochromePhotography #filmShooter #BlackAndWhiteFilm #analogLove #filmLife");
        }
    };

    match film_type {
        FilmType::LomographyBlackAndWhite | FilmType::LomographyColor => {
            hashtags.push_str(" #HeyLomography")
        }
        _ => {}
    };

    hashtags.push_str(&format!(" #{} #{}film", format, format));

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
