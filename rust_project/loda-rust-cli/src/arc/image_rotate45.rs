//! Rotate an image by 45 degrees.
use super::{Image, ImageSymmetry};

pub trait ImageRotate45 {
    /// Rotate an image by 45 degrees. clockwise (CW)
    /// 
    /// Where rotate by 90 degrees is a simple operation, rotate by 45 degrees is a bit more complex.
    /// This yields gaps in the rotated image. Every pixel has 4 gaps surrounding it.
    fn rotate_cw_45(&self, fill_color: u8) -> anyhow::Result<Image>;

    /// Rotate an image by 45 degrees. counter clockwise (CCW)
    /// 
    /// Where rotate by 90 degrees is a simple operation, rotate by 45 degrees is a bit more complex.
    /// This yields gaps in the rotated image. Every pixel has 4 gaps surrounding it.
    fn rotate_ccw_45(&self, fill_color: u8) -> anyhow::Result<Image>;
}

impl ImageRotate45 for Image {
    fn rotate_cw_45(&self, fill_color: u8) -> anyhow::Result<Image> {
        rotate_45(&self, fill_color, true)
    }

    fn rotate_ccw_45(&self, fill_color: u8) -> anyhow::Result<Image> {
        rotate_45(&self, fill_color, false)
    }
}

fn rotate_45(original: &Image, fill_color: u8, is_clockwise: bool) -> anyhow::Result<Image> {
    if original.width() <= 1 && original.height() <= 1 {
        // No point in processing an empty image or a 1x1 image.
        return Ok(original.clone());
    }

    let combined_u16: u16 = original.width() as u16 + original.height() as u16 - 1;
    if combined_u16 > 255 {
        return Err(anyhow::anyhow!("Unable to rotate image. The combined width and height is too large: {}", combined_u16));
    }

    let mut image = Image::color(combined_u16 as u8, combined_u16 as u8, fill_color);

    // Copy the element from the original image to the rotated image
    for get_y in 0..original.height() {
        for get_x in 0..original.width() {
            let pixel_value: u8 = original.get(get_x as i32, get_y as i32).unwrap_or(255);
            let set_x: i32 = get_x as i32 + get_y as i32;
            let set_y: i32 = get_x as i32 - get_y as i32 + (original.height() - 1) as i32;
            match image.set(set_x, set_y, pixel_value) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_x, set_y));
                }
            }
        }
    }
    if is_clockwise {
        image = image.flip_diagonal_a()?;
    } else {
        image = image.flip_y()?;
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{Histogram, ImageHistogram, ImageRemoveRowColumn, ImageTrim, ImageTryCreate, Rectangle};
    use bit_set::BitSet;
    use num_integer::Integer;

    #[test]
    fn test_10000_rotate_ccw_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0,
            0, 2, 0, 6, 0,
            1, 0, 5, 0, 9,
            0, 4, 0, 8, 0,
            0, 0, 7, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_rotate_ccw_landscape_onerow() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3,
            0, 2, 0,
            1, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_ccw_landscape_tworows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0,
            0, 2, 0, 6,
            1, 0, 5, 0,
            0, 4, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_ccw_portrait_onecolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 
            2, 
            3,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 2, 0,
            0, 0, 3,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_rotate_ccw_portrait_twocolumns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 4, 0, 0,
            1, 0, 5, 0,
            0, 2, 0, 6,
            0, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_rotate_cw() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_cw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 2, 0, 4,
            3, 0, 5, 0,
            0, 6, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_reversable_rotate_remove_empty_lines() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 2, 0, 4,
            3, 0, 5, 0,
            0, 6, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(0).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 1, 0, 4, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 2, 0, 5, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 3, 0, 6, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
        ];
        let expected0: Image = Image::try_create(7, 7, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let histogram_columns: Vec<Histogram> = actual0.histogram_columns();
        let histogram_rows: Vec<Histogram> = actual0.histogram_rows();

        let space_color: u8 = 0;

        // Identify the rows and columns that can be removed
        let mut delete_row_indexes = BitSet::new();
        for (index, histogram) in histogram_rows.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                continue;
            }
            if histogram.most_popular_color_disallow_ambiguous() == Some(space_color) {
                delete_row_indexes.insert(index as usize);
            }
        }
        let mut delete_column_indexes = BitSet::new();
        for (index, histogram) in histogram_columns.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                continue;
            }
            if histogram.most_popular_color_disallow_ambiguous() == Some(space_color) {
                delete_column_indexes.insert(index as usize);
            }
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");

        // Assert
        let expected_pixels1: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let expected1: Image = Image::try_create(2, 3, expected_pixels1).expect("image");
        assert_eq!(actual1, expected1);
    }

    #[test]
    fn test_30001_reversable_rotate_keep_empty_lines_inside_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 0, 4,
            3, 0, 0, 0,
            0, 6, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        let space_color: u8 = 0;
        
        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 1, 0, 4, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 3, 0, 6, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
        ];
        let expected0: Image = Image::try_create(7, 7, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");
        assert_eq!(rect, Rectangle::new(2, 1, 3, 5));

        // Keep every second row and column
        let mut keep_ys = BitSet::new();
        for y in 0..rect.height() {
            if y.is_even() {
                keep_ys.insert((y as usize) + rect.y() as usize);
            }
        }
        let mut keep_xs = BitSet::new();
        for x in 0..rect.width() {
            if x.is_even() {
                keep_xs.insert((x as usize) + rect.x() as usize);
            }
        }

        // Identify the rows and columns that can be removed
        let mut delete_row_indexes = BitSet::new();
        let mut delete_column_indexes = BitSet::new();
        for x in 0..actual0.width() {
            if keep_xs.contains(x as usize) {
                continue;
            }
            delete_column_indexes.insert(x as usize);
        }
        for y in 0..actual0.height() {
            if keep_ys.contains(y as usize) {
                continue;
            }
            delete_row_indexes.insert(y as usize);
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");

        // Assert
        let expected_pixels1: Vec<u8> = vec![
            1, 4,
            0, 0,
            3, 6,
        ];
        let expected1: Image = Image::try_create(2, 3, expected_pixels1).expect("image");
        assert_eq!(actual1, expected1);

        // Rotating again, should yield the input image
        let actual2: Image = actual1.rotate_cw_45(space_color).expect("image");
        assert_eq!(actual2, input);
    }
}
