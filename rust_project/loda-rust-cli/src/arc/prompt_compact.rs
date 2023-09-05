use super::{Image, TaskGraph};
use super::prompt::{PromptSerialize, PromptDeserialize};
use super::arc_work_model::{Task, PairType};
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};

lazy_static! {
    /// Remove prefix and suffix, so only the image data remains.
    static ref EXTRACT_IMAGE_DATA: Regex = Regex::new(r"(width\d+,height\d+(?:,(:?\d+))+)").unwrap();

    /// Extract string, value from a string like: `width29`
    static ref EXTRACT_STRING_VALUE: Regex = Regex::new(r"(\w+)(\d+)").unwrap();

    /// Determine if it's all digits in the range 0..=9
    static ref ALL_DIGITS: Regex = Regex::new(r"^\d+$").unwrap();
}

struct TextToImage;

impl TextToImage {
    /// Decode a compact string representation into an ARC image.
    fn convert(input: &str) -> anyhow::Result<(Image, Option<String>)> {
        // Remove prefix and suffix
        let capture = match EXTRACT_IMAGE_DATA.captures(input) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("no image data found"));
            }
        };
        let input_trimmed: &str = capture.get(1).map_or("", |m| m.as_str());

        // Extract parameters for: `width`, `height`.
        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for capture in EXTRACT_STRING_VALUE.captures_iter(input_trimmed) {
            let capture1: &str = capture.get(1).map_or("", |m| m.as_str());
            let capture2: &str = capture.get(2).map_or("", |m| m.as_str());
            let value: u8 = capture2.parse::<u8>().context("value")?;
            match capture1 {
                "width" => {
                    found_width = Some(value);
                },
                "height" => {
                    found_height = Some(value);
                },
                _ => {}
            }
        }
        let field_width: u8 = found_width.context("width")?;
        let field_height: u8 = found_height.context("height")?;

        // Extract only strings with pixel values
        let mut rows = Vec::<String>::new();
        let mut width_max: usize = usize::MIN;
        let mut width_min: usize = usize::MAX;
        for item in input_trimmed.split(",") {
            if !ALL_DIGITS.is_match(item) {
                continue;
            }
            width_min = width_min.min(item.len());
            width_max = width_max.max(item.len());
            rows.push(item.to_string());
        }
        let pixeldata_height: usize = rows.len();

        // Checks if there is consensus about the width and the height and the pixeldata
        let same_width: bool = (width_max == width_min) && (width_max == field_width as usize);
        let same_height: bool = pixeldata_height == (field_height as usize);
        let same_size: bool = same_width && same_height;

        // Pick the biggest size of the size parameters, so no pixel data is outside the visible area.
        let width: u8 = (field_width as usize).max(width_max).min(40) as u8;
        let height: u8 = (field_height as usize).max(pixeldata_height).min(40) as u8;

        // Create empty image with 255 color to indicate that it has not been assigned a color yet.
        let fill_color: u8 = 255;
        let mut image: Image = Image::color(width, height, fill_color);

        // Assign pixel values
        for (row_index, row) in rows.iter().enumerate() {
            for (column_index, item) in row.chars().enumerate() {
                let x: i32 = column_index as i32;
                let y: i32 = row_index as i32;
                let color: u8 = item.to_digit(10).unwrap_or(255) as u8;
                _ = image.set(x, y, color);
            }
        }

        let mut problems = Vec::<String>::new();
        if width_min != width_max {
            let s: String = format!("Inconsistent width of pixeldata rows width_min: {} width_max: {}. They are supposed to be the same.", width_min, width_max);
            problems.push(s);
        }
        if !same_size {
            let s: String = format!("There is a mismatch between size of the image, and the pixel data. size: {}x{}, pixel data: {}x{}", field_width, field_height, width_max, pixeldata_height);
            problems.push(s);
        }
        let status: Option<String> = if problems.is_empty() {
            None
        } else {
            Some(problems.join(", "))
        };

        Ok((image, status))
    }
}

struct ImageToText;

impl ImageToText {
    /// Creates a compact string representation of an ARC image.
    /// 
    /// If `include_size` is false, then there is no width and height info in the dictionary.
    /// Returns a string like `008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    /// 
    /// If `include_size` is true, then it will include the width and height of the image, like this
    /// `width9,height9,008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    fn convert(image: &Image, include_size: bool) -> anyhow::Result<String> {
        let mut items = Vec::<String>::new();
        if include_size {
            items.push(format!("width{}", image.width()));
            items.push(format!("height{}", image.height()));
        }
        for y in 0..image.height() {
            let mut s = String::new();
            for x in 0..image.width() {
                let pixel = image.get(x as i32, y as i32).unwrap_or(255);
                s += &format!("{}", pixel);
            }
            items.push(s);
        }
        Ok(items.join(","))
    }
}

#[derive(Clone, Debug)]
pub struct PromptCompactSerializer;

impl PromptSerialize for PromptCompactSerializer {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let include_size: bool = true;

        let mut rows = Vec::<String>::new();

        rows.push("Hi, I'm doing Python experiments.\n\n".to_string());

        rows.push("These are images.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        rows.push("input = {}".to_string());
        rows.push("output = {}".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            rows.push(format!("# Group{}", pair_index));

            {
                let s0: String = ImageToText::convert(&pair.input.image, include_size)?;
                let s1: String = format!("input[{}] = '{}'", pair_index, s0);
                rows.push(s1);
            }

            match pair.pair_type {
                PairType::Train => {
                    let s0: String = ImageToText::convert(&pair.output.image, include_size)?;
                    let s1: String = format!("output[{}] = '{}'", pair_index, s0);
                    rows.push(s1);
                },
                PairType::Test => {
                    let s1: String = format!("output[{}] = 'PREDICT'", pair_index);
                    rows.push(s1);
                }
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        

        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImageSize};

    #[test]
    fn test_10000_image_to_text_without_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToText::convert(&input, false).expect("ok");

        // Assert
        let expected = "779,879";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_to_text_with_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            0, 1, 2,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToText::convert(&input, true).expect("ok");

        // Assert
        let expected = "width3,height2,012,012";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_text_to_image() {
        // Arrange
        let input: &str = "width2,height3,12,34,56";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20001_text_to_image_inconsistent_width() {
        // Arrange
        let input: &str = "width2,height3,12,3499,56";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 255, 255,
            3, 4, 9, 9,
            5, 6, 255, 255,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        let message: String = actual.1.expect("error message");
        assert_eq!(message.contains("width_min: 2 width_max: 4"), true);
    }

    #[test]
    fn test_20002_text_to_image_remove_prefix_and_suffix() {
        // Arrange
        let input: &str = "junk`output[8] = 'width2,height3,12,34,56'`junk";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }
}
