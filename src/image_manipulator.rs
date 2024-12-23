use image::{DynamicImage, GenericImage, GenericImageView, ImageError, Rgba};

#[derive(Default)]
pub struct ImageManipulator {
    image: DynamicImage,
    output: std::path::PathBuf,

    expected_ratio: Option<(f32, f32)>,
    expected_longest_side: Option<usize>,
    expected_border_spacing: Option<usize>,
}

impl ImageManipulator {
    pub fn new(
        input: std::path::PathBuf,
        output: std::path::PathBuf,
    ) -> Result<ImageManipulator, ImageError> {
        let image = image::open(input)?;

        Ok(ImageManipulator {
            image,
            output: output.clone(),
            ..Default::default()
        })
    }

    pub fn add_border(mut self, spacing: usize) -> ImageManipulator {
        self.expected_border_spacing = Some(spacing);
        self
    }

    pub fn fill_to_aspect_ratio(mut self, width: f32, height: f32) -> ImageManipulator {
        self.expected_ratio = Some((width, height));
        self
    }

    pub fn longest_side(mut self, size: usize) -> ImageManipulator {
        self.expected_longest_side = Some(size);
        self
    }

    fn try_add_border(&mut self) {
        if let Some(border_spacing) = self.expected_border_spacing {
            let (width, height) = self.image.dimensions();
            let spacing = border_spacing as u32;
            let new_width = width + (spacing * 2);
            let new_height = height + (spacing * 2);

            let mut bordered_image = self.image.clone().resize_exact(
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
            for y in 0..new_height {
                for x in 0..new_width {
                    bordered_image.put_pixel(x, y, Rgba([255; 4]));
                }
            }

            for y in 0..height {
                for x in 0..width {
                    let pixel = self.image.get_pixel(x, y);
                    bordered_image.put_pixel(x + spacing, y + spacing, pixel);
                }
            }

            self.image = bordered_image;
        }
    }

    fn fill_to_given_dimensions(
        &mut self,
        width: u32,
        height: u32,
        orig_width: u32,
        orig_height: u32,
        x_offset: u32,
        y_offset: u32,
    ) {
        let mut bordered_image =
            self.image
                .clone()
                .resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        for y in 0..height {
            for x in 0..width {
                bordered_image.put_pixel(x, y, Rgba([255; 4]));
            }
        }

        for y in 0..orig_height {
            for x in 0..orig_width {
                let pixel = self.image.get_pixel(x, y);
                bordered_image.put_pixel(x + x_offset, y + y_offset, pixel);
            }
        }

        self.image = bordered_image;
    }

    /// A tuple of (new_width, new_height).
    fn calculate_dimensions(
        &self,
        original_width: u32,
        original_height: u32,
        aspect_ratio: (f32, f32),
    ) -> (u32, u32) {
        let (aspect_width, aspect_height) = aspect_ratio;
        let aspect_ratio = aspect_width / aspect_height;

        let width_based_height = (original_width as f32 / aspect_ratio).ceil() as u32;
        let height_based_width = (original_height as f32 * aspect_ratio).ceil() as u32;

        if width_based_height >= original_height {
            (original_width, width_based_height)
        } else {
            (height_based_width, original_height)
        }
    }

    fn try_fill_aspect_ratio(&mut self) {
        if let Some(aspect_ratio) = self.expected_ratio {
            let (width, height) = self.image.dimensions();
            let mut x_offset = 0;
            let mut y_offset = 0;

            let (expected_width, expected_height) =
                self.calculate_dimensions(width, height, aspect_ratio);

            if width < expected_width {
                x_offset = (expected_width as i64 - width as i64) / 2;
            } else {
                y_offset = (expected_height as i64 - height as i64) / 2;
            }

            if x_offset > 0 || y_offset > 0 {
                self.fill_to_given_dimensions(
                    expected_width,
                    expected_height,
                    width,
                    height,
                    x_offset as u32,
                    y_offset as u32,
                );
            }
        }
    }

    fn try_longest_side(&mut self) {
        if let Some(longest_side) = self.expected_longest_side {
            self.image = self.image.resize(
                longest_side as u32,
                longest_side as u32,
                image::imageops::FilterType::Lanczos3,
            );
        }
    }

    pub fn save(&mut self) -> Result<(), ImageError> {
        self.try_add_border();
        self.try_fill_aspect_ratio();
        self.try_longest_side();

        self.image.save(self.output.clone())
    }
}
