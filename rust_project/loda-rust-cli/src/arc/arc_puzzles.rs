#[cfg(test)]
mod tests {
    use crate::arc::{RunWithProgram, RunWithProgramResult, SolutionSimple, ImageResize};
    use crate::arc::{ImageOverlay, ImageNoiseColor, ImageRemoveGrid, ImageExtractRowColumn, ImageSegment, ImageSegmentAlgorithm, ImageMask, Histogram};
    use crate::arc::{Model, GridToImage, ImagePair, ImageFind, ImageOutline, ImageRotate, ImageBorder};
    use crate::arc::{Image, convolution2x2, PopularObjects, ImageNeighbour, ImageNeighbourDirection};
    use crate::arc::{ImageTrim, ImageRemoveDuplicates, ImageStack, ImageMaskCount, ImageSetPixelWhere};
    use crate::arc::{ImageReplaceColor, ImageSymmetry, ImageOffset, ImageColorProfile, ImageCreatePalette};
    use crate::arc::{ImageNgram, RecordTrigram, ImageHistogram, ImageDenoise, ImageDetectHole, ImageTile};
    use bit_set::BitSet;
    use std::collections::HashMap;

    #[test]
    fn test_10000_puzzle_4258a5f9() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_pixel_color: u8 = input.most_popular_color().expect("pixel");
            let result_image: Image = input.outline_type1(1, background_pixel_color).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("4258a5f9").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    const ADVANCED_PROGRAM_4258A5F9: &'static str = r#"
    mov $40,0 ; outline color

    ; process "train" vector
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
        mov $0,$$81 ; load train[x].input image
        mov $1,$$82 ; load train[x].output image

        ; analyze the output images
        f12 $1,101070 ; least popular colors
        mov $40,$2 ; get the outline color

        ; next iteration
        add $81,10 ; jump to address of next training input image
        add $82,10 ; jump to address of next training output image
    lpe
    
    ; process "train"+"test" vectors
    mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
    mov $81,100 ; address of vector[0].input
    mov $82,102 ; address of vector[0].computed_output
    lps $80
        mov $0,$$81 ; load vector[x].input image

        mov $5,$0
        f11 $5,101060 ; most popular color

        mov $1,$40 ; outline color
        mov $2,$5 ; background color
        f31 $0,101080 ; draw outline

        mov $$82,$0 ; save vector[x].computed_output image

        ; next iteration
        add $81,10 ; jump to address of next input image
        add $82,10 ; jump to address of next computed_output image
    lpe
    "#;

    #[test]
    fn test_10001_puzzle_4258a5f9_loda() {
        let model: Model = Model::load_testdata("4258a5f9").expect("model");
        let program = ADVANCED_PROGRAM_4258A5F9;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_advanced(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_20000_puzzle_5614dbcf() {
        let solution: SolutionSimple = |data| {
            let image_denoised: Image = data.image.denoise_type3(3).expect("image");
            let result_image: Image = image_denoised.remove_duplicates().expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("5614dbcf").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_5614DBCF: &'static str = "
    mov $1,3 ; number of noise colors to remove
    f21 $0,101092 ; denoise type 3
    f11 $0,101140 ; remove duplicates
    ";

    #[test]
    fn test_20001_puzzle_5614dbcf_loda() {
        let model: Model = Model::load_testdata("5614dbcf").expect("model");
        let program = PROGRAM_5614DBCF;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_30000_puzzle_2013d3e2() {
        let solution: SolutionSimple = |data| {
            let input_trimmed: Image = data.image.trim().expect("image");

            // Extract top/left corner
            let top_rows: Image = input_trimmed.top_rows(input_trimmed.height() / 2).expect("image");
            let result_image: Image = top_rows.left_columns(input_trimmed.height() / 2).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("2013d3e2").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_2013D3E2: &'static str = "
    f11 $0,101160 ; trim

    mov $4,$0
    mov $5,$0

    f11 $4,101000 ; get width
    f11 $5,101001 ; get height

    div $4,2
    div $5,2

    mov $1,$4
    f21 $0,101220 ; get top rows
    
    mov $1,$5
    f21 $0,101222 ; get left columns
    ";

    #[test]
    fn test_30001_puzzle_2013d3e2_loda() {
        let model: Model = Model::load_testdata("2013d3e2").expect("model");
        let program = PROGRAM_2013D3E2;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_40000_puzzle_90c28cc7_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let input_trimmed: Image = input.trim().expect("image");
            let result_image: Image = input_trimmed.remove_duplicates().expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("90c28cc7").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_90C28CC7: &'static str = "
    f11 $0,101160 ; trim
    f11 $0,101140 ; remove duplicates
    ";

    #[test]
    fn test_40001_puzzle_90c28cc7_loda() {
        let model: Model = Model::load_testdata("90c28cc7").expect("model");
        let program = PROGRAM_90C28CC7;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_50000_puzzle_7468f01a_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let input_trimmed: Image = input.trim().expect("image");
            let result_image: Image = input_trimmed.flip_x().expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("7468f01a").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_7468F01A: &'static str = "
    f11 $0,101160 ; trim
    f11 $0,101190 ; flip x
    ";

    #[test]
    fn test_50001_puzzle_7468f01a_loda() {
        let model: Model = Model::load_testdata("7468f01a").expect("model");
        let program = PROGRAM_7468F01A;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_60000_puzzle_63613498() -> anyhow::Result<()> {
        // TODO: port to LODA
        let model: Model = Model::load_testdata("63613498").expect("model");
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // Extract needle
        let mut needle: Image = Image::zero(3, 3);
        let center_pixel_color: u8 = input.get(1, 1).unwrap_or(255);
        for y in 0..3i32 {
            for x in 0..3i32 {
                let pixel_value: u8 = input.get(x, y).unwrap_or(255);
                let mut mask_value: u8 = 0;
                if pixel_value == center_pixel_color {
                    mask_value = 1;
                }
                match needle.set(x, y, mask_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) inside the needle bitmap", x, y));
                    }
                }
            }
        }

        // Clear the needle area from the search area
        let mut search_area: Image = input.clone();
        for y in 0..4i32 {
            for x in 0..4i32 {
                match search_area.set(x, y, 0) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) inside the search area", x, y));
                    }
                }
            }
        }
        // println!("needle: {:?}", needle);
        // println!("search area: {:?}", search_area);

        // Find the pattern
        let mut optional_position: Option<(u8, u8)> = None;
        for color in 1..=255u8 {
            let needle_with_color: Image = needle.replace_color(1, color)?;
            optional_position = search_area.find_exact(&needle_with_color).expect("some position");
            if optional_position == None {
                continue;
            }
            break;
        }
        let position: (u8, u8) = match optional_position {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Didn't find needle inside the search area"));
            }
        };
        // println!("position: {:?}", position);

        // Clear color of the found pattern
        let mut result_bitmap: Image = input.clone();
        for y in 0..3i32 {
            for x in 0..3i32 {
                let xx = x + (position.0 as i32);
                let yy = y + (position.1 as i32);
                let pixel_value: u8 = needle.get(x, y).unwrap_or(255);
                if pixel_value == 0 {
                    continue;
                }
                match result_bitmap.set(xx, yy, 5) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) in the result_bitmap", x, y));
                    }
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_70000_puzzle_cdecee7f() -> anyhow::Result<()> {
        // TODO: port to LODA
        let model: Model = Model::load_testdata("cdecee7f").expect("model");
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let background_pixel_color: u8 = input.most_popular_color().expect("pixel");

        // Traverse columns
        let mut stack: Vec<u8> = vec!();
        for x in 0..input.width() {
            // Take foreground pixels that is different than the background color, and append the foreground pixel to the stack
            for y in 0..input.height() {
                let pixel_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value != background_pixel_color {
                    stack.push(pixel_value);
                }
            }
        }
        // Padding to 9 items
        while stack.len() < 9 {
            stack.push(0);
        }

        // Transfer values from the 9 element stack to the 3x3 bitmap
        let mut result_bitmap: Image = Image::zero(3, 3);
        for (index, pixel_value) in stack.iter().enumerate() {
            let y: usize = index / 3;
            let mut x: usize = index % 3;
            if y == 1 {
                // The middle row is reversed
                x = 2 - x;
            }
            let set_x: i32 = x as i32;
            let set_y: i32 = y as i32;
            match result_bitmap.set(set_x, set_y, *pixel_value) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) in the result_bitmap", x, y));
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_80000_puzzle_007bbfb7() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let mut image: Image = Image::zero(9, 9);
            for y in 0..input.height() {
                for x in 0..input.width() {
                    let mask_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                    if mask_value == 0 {
                        continue;
                    }
                    image = image.overlay_with_position(&input, (x * 3) as i32, (y * 3) as i32)?;
                }
            }
            Ok(image)
        };
        let model: Model = Model::load_testdata("007bbfb7").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_007BBFB7: &'static str = "
    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$5 ; tile0
    mov $12,$0 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_80001_puzzle_007bbfb7_loda() {
        let model: Model = Model::load_testdata("007bbfb7").expect("model");
        let program = PROGRAM_007BBFB7;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_90000_puzzle_b9b7f026_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("color");

            // Detect corners / holes
            let corner_image: Image = input.detect_hole_type1(background_color).expect("image");
            // println!("input: {:?}", input);
            // println!("corner_image: {:?}", corner_image);
    
            // Extract color of the corner
            let corner_color: u8 = corner_image.least_popular_color().expect("color");
            let result_image: Image = Image::color(1, 1, corner_color);
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("b9b7f026").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_B9B7F026: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color
    ; $1 is background color
    f21 $0,101110 ; detect holes

    mov $2,$0
    f11 $2,101070 ; least popular color
    ; $2 is the corner color

    mov $0,1 ; width=1
    mov $1,1 ; height=1
    f31 $0,101010 ; create image with color
    ";

    #[test]
    fn test_90001_puzzle_b9b7f026_loda() {
        let model: Model = Model::load_testdata("b9b7f026").expect("model");
        let program = PROGRAM_B9B7F026;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_100000_puzzle_a79310a0_manual() {
        let solution: SolutionSimple = |data| {
            let image_with_offset: Image = data.image.offset_wrap(0, 1).expect("image");
            let result_image: Image = image_with_offset.replace_color(8, 2).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("a79310a0").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_A79310A0: &'static str = "
    mov $1,0
    mov $2,1
    f31 $0,101180 ; offset dx,dy
    mov $1,8
    mov $2,2
    f31 $0,101050 ; replace color with color
    ";

    #[test]
    fn test_100001_puzzle_a79310a0_loda() {
        let model: Model = Model::load_testdata("a79310a0").expect("model");
        let program = PROGRAM_A79310A0;

        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_100002_puzzle_a79310a0_manual_without_hardcoded_colors() -> anyhow::Result<()> {
        // Pseudo code for a LODA program:
        // Loop through training input/output images.
        // Extract color palette from image. Nx2 where N is the number of histogram entries. Top row is the color, bottom row the count.
        // Merge color palette images. hstack images.
        // Remove duplicates from palette images.
        // Use color palette images for replacement.

        let model: Model = Model::load_testdata("a79310a0").expect("model");

        // These images contain 2 colors. Build a mapping from source color to target color
        let train_pairs: Vec<ImagePair> = model.images_train().expect("pairs");
        let mut color_replacements = HashMap::<u8, u8>::new();
        for (index, pair) in train_pairs.iter().enumerate() {
            let input_histogram = pair.input.histogram_all();
            if input_histogram.number_of_counters_greater_than_zero() != 2 {
                return Err(anyhow::anyhow!("input[{}] Expected exactly 2 colors", index));
            }
            let output_histogram = pair.output.histogram_all();
            if output_histogram.number_of_counters_greater_than_zero() != 2 {
                return Err(anyhow::anyhow!("output[{}] Expected exactly 2 colors", index));
            }
            let in_color0: u8 = input_histogram.most_popular_color().expect("u8");
            let out_color0: u8 = output_histogram.most_popular_color().expect("u8");
            color_replacements.insert(in_color0, out_color0);

            let in_color1: u8 = input_histogram.least_popular_color().expect("u8");
            let out_color1: u8 = output_histogram.least_popular_color().expect("u8");
            color_replacements.insert(in_color1, out_color1);
        }

        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {

            let mut image: Image = pair.input.offset_wrap(0, 1).expect("image");
            image = image.replace_colors_with_hashmap(&color_replacements).expect("image");

            assert_eq!(image, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
        Ok(())
    }

    #[test]
    fn test_100003_puzzle_a79310a0_manual_without_hashmap() {
        let model: Model = Model::load_testdata("a79310a0").expect("model");

        // These images contain 2 colors. Build a mapping from source color to target color
        let train_pairs: Vec<ImagePair> = model.images_train().expect("pairs");
        let mut palette_image = Image::empty();
        for pair in &train_pairs {
            let image: Image = pair.input.palette_using_histogram(&pair.output, false).expect("image");
            palette_image = palette_image.hjoin(image).expect("image");
        }

        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {

            let mut image: Image = pair.input.offset_wrap(0, 1).expect("image");
            image = image.replace_colors_with_palette_image(&palette_image).expect("image");

            assert_eq!(image, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    const ADVANCED_PROGRAM_A79310A0: &'static str = r#"
    mov $40,0 ; palette image accumulated

    ; process "train" vector
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
        mov $0,$$81 ; load train[x].input image
        mov $1,$$82 ; load train[x].output image

        ; analyze the images
        f21 $0,101130 ; build palette image with color mapping from input to output
        mov $41,$0
        f21 $40,101030 ; hstack of the palette images

        ; next iteration
        add $81,10 ; jump to address of next training input image
        add $82,10 ; jump to address of next training output image
    lpe
    
    ; process "train"+"test" vectors
    mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
    mov $81,100 ; address of vector[0].input
    mov $82,102 ; address of vector[0].computed_output
    lps $80
        mov $0,$$81 ; load vector[x].input image

        ; change offset of the image
        mov $1,0 ; offset x=0
        mov $2,1 ; offset y=+1
        f31 $0,101180 ; offset x, y

        ; replace colors of the image using the palette image
        mov $1,$40 ; palette image
        f21 $0,101052 ; replace colors using palette image

        mov $$82,$0 ; save vector[x].computed_output image

        ; next iteration
        add $81,10 ; jump to address of next input image
        add $82,10 ; jump to address of next computed_output image
    lpe
    "#;

    #[test]
    fn test_100004_puzzle_a79310a0_loop_over_images_in_loda() {
        let model: Model = Model::load_testdata("a79310a0").expect("model");
        let program = ADVANCED_PROGRAM_A79310A0;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_advanced(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    /// Detect corners and edges
    fn mask_and_repair_areas(input: &Image) -> anyhow::Result<(Image, Image)> {
        // Assign 0 to background, assign 1 to foreground
        let mut mask: Image = input.clone();
        for pixel_value in 2..=255 {
            mask = mask.replace_color(pixel_value, 1).expect("image");
        }
        // println!("mask: {:?}", mask);

        // Detect corners and edges
        let repair_areas: Image = convolution2x2(&mask, |bm| {
            let pixel00: u8 = bm.get(0, 0).unwrap_or(255);
            let pixel10: u8 = bm.get(1, 0).unwrap_or(255);
            let pixel01: u8 = bm.get(0, 1).unwrap_or(255);
            let pixel11: u8 = bm.get(1, 1).unwrap_or(255);
            let number_of_zeros: u8 = 
                u8::min(pixel00, 1) + 
                u8::min(pixel10, 1) + 
                u8::min(pixel01, 1) + 
                u8::min(pixel11, 1);
            if number_of_zeros <= 1 {
                // 1 mask pixel turned on, and 3 pixels is the background, don't consider this as a corner.
                // 0 mask pixels turned on, all 4 pixels are the background then ignore.
                return Ok(0);
            }

            let mut mask: u8 = 0;
            if pixel00 == pixel10 { mask |= 1; }
            if pixel01 == pixel11 { mask |= 2; }
            if pixel00 == pixel01 { mask |= 4; }
            if pixel10 == pixel11 { mask |= 8; }
            let value: u8 = match mask {
                3 => 5, // edge, rows differ
                5 => 1, // corner top left
                6 => 2, // corner bottom left
                9 => 3, // corner top right
                10 => 4, // corner bottom right
                12 => 6, // edge, columns differ
                _ => 0,
            };
            Ok(value)
        }).expect("image");
        Ok((mask, repair_areas))
    }

    fn repair_corner_top_left(bitmap: &mut Image, x: i32, y: i32, trigram_x: &Vec<RecordTrigram>, trigram_y: &Vec<RecordTrigram>) -> anyhow::Result<()> {
        if x < 1 || y < 1 {
            println!("repair corner top left: {}, {} - insufficient room to make bigram", x, y);
            return Ok(());
        }
        println!("repair corner top left: {}, {}", x, y);

        let pixel_top_top: u8 = bitmap.get(x, y - 2).unwrap_or(255);
        let pixel_top: u8 = bitmap.get(x, y - 1).unwrap_or(255);
        let pixel_left_left: u8 = bitmap.get(x - 2, y).unwrap_or(255);
        let pixel_left: u8 = bitmap.get(x - 1, y).unwrap_or(255);

        let mut bitset_trigram_x = BitSet::with_capacity(256);
        let mut bitset_trigram_y = BitSet::with_capacity(256);
        for candidate in 0..255u8 {
            for record in trigram_x.iter() {
                if record.word0 == pixel_left_left && record.word1 == pixel_left && record.word2 == candidate {
                    bitset_trigram_x.insert(candidate as usize);
                }
            }
            for record in trigram_y.iter() {
                if record.word0 == pixel_top_top && record.word1 == pixel_top && record.word2 == candidate {
                    bitset_trigram_y.insert(candidate as usize);
                }
            }
        }
        let mut bitset_trigram = BitSet::with_capacity(256);
        bitset_trigram.clone_from(&bitset_trigram_x);
        bitset_trigram.intersect_with(&bitset_trigram_y);
        if bitset_trigram.len() >= 2 {
            println!("more than 1 candidate. trigram: {:?}", bitset_trigram);
        }

        let mut found_color = 255;
        for index in bitset_trigram.iter() {
            if index > 255 {
                return Err(anyhow::anyhow!("Integrity error. Encountered bitset index outside of u8 range [0..255]"));
            }
            found_color = index as u8;
            break;
        }
        println!("repair ({}, {}) = {:?}", x, y, found_color);

        match bitmap.set(x, y, found_color) {
            Some(()) => {},
            None => {
                return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
            }
        }
        Ok(())
    }

    #[test]
    fn test_110000_puzzle_0dfd9992() -> anyhow::Result<()> {
        // TODO: port to LODA
        let model: Model = Model::load_testdata("0dfd9992")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");

        // TODO: make the rest of the tests pass OK. Currently these fails.
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // Bigrams
        // let bigram_x_unfiltered: Vec<RecordBigram> = input.bigram_x().expect("bigram");
        // let bigram_y_unfiltered: Vec<RecordBigram> = input.bigram_y().expect("bigram");
        // println!("bigram_x_unfiltered: {:?}", bigram_x_unfiltered);
        // println!("bigram_y_unfiltered: {:?}", bigram_y_unfiltered);
        // Remove bigrams where the background pixel (0) is contained in
        // let bigram_x_refs: Vec<&RecordBigram> = bigram_x_unfiltered.iter().filter(|&record| {
        //     record.word0 != 0 && record.word1 != 0
        // }).collect();
        // let bigram_y_refs: Vec<&RecordBigram> = bigram_y_unfiltered.iter().filter(|&record| {
        //     record.word0 != 0 && record.word1 != 0
        // }).collect();
        // let bigram_x: Vec<RecordBigram> = bigram_x_refs.iter().map(|&i| i.clone()).collect();
        // let bigram_y: Vec<RecordBigram> = bigram_y_refs.iter().map(|&i| i.clone()).collect();
        // println!("bigram_x: {:?}", bigram_x);
        // println!("bigram_y: {:?}", bigram_y);

        // Trigrams
        let trigram_x_unfiltered: Vec<RecordTrigram> = input.trigram_x().expect("trigram");
        let trigram_y_unfiltered: Vec<RecordTrigram> = input.trigram_y().expect("trigram");
        // println!("trigram_x_unfiltered: {:?}", trigram_x_unfiltered);
        // println!("trigram_y_unfiltered: {:?}", trigram_y_unfiltered);
        // Remove trigrams where the background pixel (0) is contained in
        let trigram_x_refs: Vec<&RecordTrigram> = trigram_x_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0 && record.word2 != 0
        }).collect();
        let trigram_y_refs: Vec<&RecordTrigram> = trigram_y_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0 && record.word2 != 0
        }).collect();
        let trigram_x: Vec<RecordTrigram> = trigram_x_refs.iter().map(|&i| i.clone()).collect();
        let trigram_y: Vec<RecordTrigram> = trigram_y_refs.iter().map(|&i| i.clone()).collect();
        // println!("trigram_x: {:?}", trigram_x);
        // println!("trigram_y: {:?}", trigram_y);
        
        let mut result_bitmap: Image = input.clone();

        let mut last_repair_count: usize = 0;
        for iteration in 0..13 {
            let (mask, repair_areas) = mask_and_repair_areas(&result_bitmap)?;
            println!("iteration#{} mask: {:?}", iteration, mask);
            println!("iteration#{} repair areas: {:?}", iteration, repair_areas);
            let mut repair_count: usize = 0;
            for y in 0..repair_areas.height() {
                for x in 0..repair_areas.width() {
                    let pixel_value: u8 = repair_areas.get(x as i32, y as i32).unwrap_or(255);
                    if pixel_value == 0 {
                        continue;
                    }
                    repair_count += 1;
    
                    // repair position
                    let repair_x = (x as i32) + 1;
                    let repair_y = (y as i32) + 1;
    
                    // if pixel_value >= 1 && pixel_value <= 4 {
                    //     println!("repair corner: {}, {}", x, y);
                    //     TODO: deal with all the cases
                    // }
                    // if pixel_value >= 5 {
                    //     println!("repair edge: {}, {}", x, y);
                    // }
    
                    if pixel_value == 1 {
                        repair_corner_top_left(&mut result_bitmap, repair_x, repair_y, &trigram_x, &trigram_y)?;
                    }
                }
            }
            println!("iteration#{} repair_count: {}", iteration, repair_count);
            if iteration > 0 {
                if last_repair_count == repair_count {
                    println!("making no progress with repairs done. aborting");
                    break;
                }
            }
            if repair_count == 0 {
                println!("repair done. no more pixels to be repaired");
                break;
            }
            last_repair_count = repair_count;
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_120000_puzzle_3bdb4ada() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let mut image = input.clone();
            for yy in 0..((image.height() as i32) - 2) {
                for xx in 0..((image.width() as i32) - 2) {
                    let top_left_pixel_value: u8 = image.get(xx, yy).unwrap_or(255);
                    let mut same_count = 1;
                    for y in 0..3 {
                        for x in 0..3 {
                            if x == 1 && y == 1 {
                                continue;
                            }
                            if x == 0 && y == 0 {
                                continue;
                            }
                            let pixel_value: u8 = image.get(xx + x, yy + y).unwrap_or(255); 
                            if pixel_value == top_left_pixel_value {
                                same_count += 1;
                            }
                        }
                    }
                    if same_count == 8 {
                        match image.set(xx + 1, yy + 1, 0) {
                            Some(()) => {},
                            None => {
                                return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
                            }
                        }
                    }
                }
            }
            Ok(image)
        };
        let model: Model = Model::load_testdata("3bdb4ada").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_130000_puzzle_7fe24cdd() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let row0: Image = Image::hstack(vec![input.clone(), input.rotate(1).expect("image")]).expect("image");
            let row1: Image = Image::hstack(vec![input.rotate(3).expect("image"), input.rotate(2).expect("image")]).expect("image");
            let result_image = Image::vstack(vec![row0.clone(), row1.clone()]).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("7fe24cdd").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_7FE24CDD: &'static str = "
    mov $5,$0 ; original corner

    ; construct top half
    mov $1,$0
    mov $2,1
    f21 $1,101170 ; rotate cw
    f21 $0,101030 ; hstack
    ; $0 is top half

    ; construct bottom half
    mov $6,2
    f21 $5,101170 ; rotate cw cw
    mov $1,$5
    mov $2,1
    f21 $1,101170 ; rotate cw
    mov $2,$5
    f21 $1,101030 ; hstack
    ; $1 is bottom half

    ; join top half and bottom half
    f21 $0,101040 ; vstack
    ";

    #[test]
    fn test_130001_puzzle_7fe24cdd_loda() {
        let model: Model = Model::load_testdata("7fe24cdd").expect("model");
        let program = PROGRAM_7FE24CDD;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_140000_puzzle_9565186b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let pixel_color: u8 = input.most_popular_color().expect("color");
            let result_image: Image = input.replace_colors_other_than(pixel_color, 5).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("9565186b").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 4);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_9565186B: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color
    mov $2,5
    f31 $0,101051 ; replace colors other than color
    ";

    #[test]
    fn test_140001_puzzle_9565186b_loda() {
        let model: Model = Model::load_testdata("9565186b").expect("model");
        let program = PROGRAM_9565186B;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 4);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_150000_puzzle_3af2c5a8() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let row0: Image = Image::hstack(vec![input.clone(), input.flip_x().expect("image")]).expect("image");
            let row1: Image = row0.flip_y().expect("image");
            let result_image = Image::vstack(vec![row0.clone(), row1.clone()]).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("3af2c5a8").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_3AF2C5A8: &'static str = "
    mov $1,$0
    f11 $1,101190 ; flip x
    f21 $0,101030 ; hstack
    mov $1,$0
    f11 $1,101191 ; flip y
    f21 $0,101040 ; vstack
    ";

    #[test]
    fn test_150001_puzzle_3af2c5a8_loda() {
        let model: Model = Model::load_testdata("3af2c5a8").expect("model");
        let program = PROGRAM_3AF2C5A8;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_44F52BB0: &'static str = "
    mov $1,$0
    f11 $1,101190 ; flip x
    cmp $0,$1
    mov $2,1 ; color when there is symmetry
    mul $2,$0
    cmp $0,0
    mul $0,7 ; color when there is no symmetry
    add $2,$0
    mov $0,1 ; output image width
    mov $1,1 ; output image height
    f31 $0,101010 ; create image
    ";

    #[test]
    fn test_160000_puzzle_44f52bb0_loda() {
        let model: Model = Model::load_testdata("44f52bb0").expect("model");
        let program = PROGRAM_44F52BB0;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 6);
        assert_eq!(result.count_test_correct(), 2);
    }

    #[test]
    fn test_170000_puzzle_496994bd() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_pixel_color: u8 = input.most_popular_color().expect("color");
            let flipped_image: Image = input.flip_y().expect("image");
            let result_image: Image = input.overlay_with_mask_color(
                &flipped_image, 
                background_pixel_color
            ).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("496994bd").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_496994BD: &'static str = "
    mov $1,$0
    mov $2,$0
    f11 $2,101060 ; most popular color
    f11 $1,101191 ; flip y
    f31 $0,101150 ; overlay
    ";

    #[test]
    fn test_170001_puzzle_496994bd_loda() {
        let model: Model = Model::load_testdata("496994bd").expect("model");
        let program = PROGRAM_496994BD;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_180000_puzzle_31aa019c() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let pixel_color: u8 = input.least_popular_color().expect("color");
            let image: Image = input.replace_colors_other_than(pixel_color, 0).expect("image");
            let outline_color: u8 = 2;
            let background_color: u8 = 0;
            let result_image: Image = image.outline_type1(outline_color, background_color).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("31aa019c").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_31AA019C: &'static str = "
    mov $1,$0
    f11 $1,101070 ; most unpopular color
    mov $2,0 ; background color
    f31 $0,101051 ; replace colors other than
    mov $1,2 ; outline color
    mov $2,0 ; background color
    f31 $0,101080 ; draw outline
    ";

    #[test]
    fn test_180001_puzzle_31aa019c_loda() {
        let model: Model = Model::load_testdata("31aa019c").expect("model");
        let program = PROGRAM_31AA019C;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_190000_puzzle_5ad4f10b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("color");

            let denoised_image: Image = input.denoise_type1(background_color).expect("image");
            // println!("denoised: {:?}", denoised_image);
    
            // Pick the most popular noise color
            let noise_color_vec: Vec<u8> = input.noise_color_vec(&denoised_image).expect("vec with colors");
            let noise_color: u8 = *noise_color_vec.first().expect("1 or more colors");
            // println!("noise color: {}", noise_color);
    
            // Remove background around the object
            let trimmed_image: Image = denoised_image.trim().expect("image");
    
            // Remove duplicate rows/columns
            let image_without_duplicates: Image = trimmed_image.remove_duplicates().expect("image");
    
            // Change color of the object
            let result_image: Image = image_without_duplicates.replace_colors_other_than(background_color, noise_color).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("5ad4f10b").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_5AD4F10B: &'static str = "
    mov $1,$0
    mov $2,$0
    mov $3,$0
    mov $9,$0

    f11 $3,101060 ; most popular color
    ; $3 is background_color

    mov $5,$0 ; noisy image
    mov $6,$3 ; background_color
    f21 $5,101090 ; denoise image
    ; $5 is denoised image

    ; $9 is noisy image
    mov $10,$5 ; denoised image
    f21 $9,101100 ; extract 1 noise color
    ; $9 is the most popular noise color

    mov $12,$5 ; denoised image
    f11 $12,101160 ; trim
    f11 $12,101140 ; remove duplicates

    mov $0,$12
    mov $1,$3 ; background color
    mov $2,$9 ; noise color
    f31 $0,101051 ; replace colors other than
    ";

    #[test]
    fn test_190001_puzzle_5ad4f10b_loda() {
        let model: Model = Model::load_testdata("5ad4f10b").expect("model");
        let program = PROGRAM_5AD4F10B;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_200000_puzzle_1190e5a7() {
        let solution: SolutionSimple = |data| {
            let without_duplicates: Image = data.image.remove_duplicates().expect("image");
            let result_image: Image = without_duplicates.remove_grid().expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("1190e5a7").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_1190E5A7: &'static str = "
    f11 $0,101140 ; remove duplicates
    f11 $0,101120 ; remove grid
    ";

    #[test]
    fn test_200001_puzzle_1190e5a7_loda() {
        let model: Model = Model::load_testdata("1190e5a7").expect("model");
        let program = PROGRAM_1190E5A7;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }
    
    #[test]
    fn test_210000_puzzle_39a8645d() {
        let solution: SolutionSimple = |data| {
            let result_image: Image = PopularObjects::most_popular_object(&data.image).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("39a8645d").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_39A8645D: &'static str = "
    f11 $0,102000 ; most popular object
    ";

    #[test]
    fn test_210001_puzzle_39a8645d_loda() {
        let model: Model = Model::load_testdata("39a8645d").expect("model");
        let program = PROGRAM_39A8645D;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }
    
    #[test]
    fn test_220000_puzzle_88a62173() {
        let solution: SolutionSimple = |data| {
            let result_image: Image = PopularObjects::least_popular_object(&data.image).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("88a62173").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_88A62173: &'static str = "
    f11 $0,102001 ; least popular object
    ";

    #[test]
    fn test_220001_puzzle_88a62173_loda() {
        let model: Model = Model::load_testdata("88a62173").expect("model");
        let program = PROGRAM_88A62173;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }
    
    #[test]
    fn test_230000_puzzle_bbc9ae5d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let repeat_count: u8 = input.width() / 2;
            let mut result_image: Image = Image::empty();
            for i in 0..repeat_count {
                let m = input.clone();
                let j = m.offset_clamp(i as i32, 0).expect("image");
                result_image = result_image.vjoin(j).expect("image");
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("bbc9ae5d").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_BBC9AE5D: &'static str = "
    mov $10,$0
    f11 $10,101000 ; get image width
    div $10,2
    ; $10 is the height of the final image
    
    mov $2,0
    mov $7,0
    lps $10

        ; clone the input image, and offset it
        mov $4,$7
        mov $5,0
        mov $3,$0
        f31 $3,101181 ; offset clamp

        ; glue onto the bottom of the result image
        f21 $2,101040 ; vstack

        add $7,1
    lpe
    mov $0,$2
    ";

    #[test]
    fn test_230001_puzzle_bbc9ae5d_loda() {
        let model: Model = Model::load_testdata("bbc9ae5d").expect("model");
        let program = PROGRAM_BBC9AE5D;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_240000_puzzle_ea32f347() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("color");
            let background_ignore_mask: Image = input.to_mask_where_color_is(background_color);
            // println!("background_ignore_mask: {:?}", background_ignore_mask);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, background_ignore_mask)
                .expect("find_objects_with_ignore_mask");
    
            // Count the number of pixels in each object
            let f = |image: &Image| -> (Image, u32) {
                let count: u32 = image.mask_count_one();
                (image.clone(), count)
            };
            let mut object_count_vec: Vec<(Image, u32)> = object_mask_vec.iter().map(f).collect();
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
            object_count_vec.reverse();
    
            // Object size to color value
            let mut color_mapping = HashMap::<usize, u8>::new();
            color_mapping.insert(0, 1); // biggest object
            color_mapping.insert(1, 4); // medium object
            color_mapping.insert(2, 2); // smallest object
    
            // Build the result image
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for (index, item) in object_count_vec.iter().enumerate() {
                let mask_image: Image = item.0.clone();
                // Obtain color for the object size
                let mut assign_color: u8 = 255;
                if let Some(color) = color_mapping.get(&index) {
                    assign_color = *color;
                }
                // Change color of the object
                let colored_object_image: Image = mask_image.replace_color(1, assign_color).expect("Image");
    
                // Overlay each object onto the result image
                result_image = mask_image.select_from_images(&result_image, &colored_object_image).expect("image");
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("ea32f347").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 4);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_250000_puzzle_7bb29440() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, object_mask.clone())
                .expect("find_objects_with_ignore_mask");
    
            // Traverse each object, and count holes in each object
            let mut object_count_vec = Vec::<(Image, u32)>::new();
            for mask_image in &object_mask_vec {
                let histogram: Histogram = input.histogram_with_mask(&mask_image).expect("histogram");
                let mut pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
    
                // Remove the background color of the rectangle
                pairs.pop();
    
                // Number of holes inside the rectangle
                let mut pixel_count: u32 = 0;
                for pair in &pairs {
                    pixel_count += pair.0;
                }
    
                object_count_vec.push((mask_image.clone(), pixel_count));
            }
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
    
            // Pick the first the object with lowest pixel count
            let (mask_image, _pixel_count) = object_count_vec.first().expect("first object");
    
            // Extract pixels from input image, just for the object
            let image: Image = mask_image.select_from_image(&input, background_color).expect("image");
    
            let result_image = image.trim().expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("7bb29440").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_260000_puzzle_5521c0d9() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, object_mask.clone())
                .expect("find_objects_with_ignore_mask");
    
            // Adjust offsets for all objects
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for mask_image in &object_mask_vec {
    
                // Bounding box of object
                let (_x0, y0, _x1, _y1) = match mask_image.bounding_box() {
                    Some(value) => value,
                    None => {
                        continue;
                    }
                };
    
                // Determine how much to adjust offset
                let distance_from_bottom: i32 = (input.height() as i32) - (y0 as i32);
                let offset_y: i32 = -distance_from_bottom;
    
                // Adjust offset
                let mask_with_offset: Image = mask_image.offset_wrap(0, offset_y).expect("image");
                let image_with_offset: Image = input.offset_wrap(0, offset_y).expect("image");
    
                result_image = mask_with_offset.select_from_images(&result_image, &image_with_offset).expect("image");
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("5521c0d9").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_270000_puzzle_7f4411dc() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let result_image: Image = input.denoise_type1(background_color).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("7f4411dc").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_7F4411DC: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; $0 is noisy image
    ; $1 is background_color
    f21 $0,101090 ; denoise type 1
    ; $0 is denoised image
    ";

    #[test]
    fn test_270001_puzzle_7f4411dc_loda() {
        let model: Model = Model::load_testdata("7f4411dc").expect("model");
        let program = PROGRAM_7F4411DC;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_280000_puzzle_aabf363d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, object_mask.clone())
                .expect("find_objects_with_ignore_mask");
    
            // Traverse each object, and measure object size
            let mut object_count_vec = Vec::<(Image, u32)>::new();
            for mask_image in &object_mask_vec {
                let histogram: Histogram = input.histogram_with_mask(&mask_image).expect("histogram");
                let pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
    
                // Measure size of the object
                let mut pixel_count: u32 = 0;
                for pair in &pairs {
                    if pair.1 == background_color {
                        continue;
                    }
                    pixel_count += pair.0;
                }
    
                object_count_vec.push((mask_image.clone(), pixel_count));
            }
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
    
            // Pick the first the object with lowest pixel count
            let (mask_image_biggest, _pixel_count) = object_count_vec.last().expect("first object");
            let (mask_image_smallest, _pixel_count) = object_count_vec.first().expect("first object");
    
            let histogram_smallest: Histogram = input.histogram_with_mask(&mask_image_smallest).expect("histogram");
            let fill_color: u8 = histogram_smallest.most_popular_color().expect("color");
    
            let mut result_image: Image = mask_image_biggest.clone();
            result_image = result_image.replace_color(0, background_color).expect("image");
            result_image = result_image.replace_color(1, fill_color).expect("image");
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("aabf363d").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 2);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_290000_puzzle_00d62c1b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let replacement_color: u8 = 4;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let border_mask_image: Image = Image::border_inside(input.width(), input.height(), 0, 1, 1).expect("image");
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("find_objects");
    
            // Traverse the interior objects. Replace color for the interior object.
            let mut result_image: Image = input.clone();
            for mask_image in &object_mask_vec {
                let mask_image_border_overlap: Image = border_mask_image.select_from_image(mask_image, 0).expect("image");
                let border_histogram: Histogram = input.histogram_with_mask(&mask_image_border_overlap).expect("histogram");
                if let Some(border_color) = border_histogram.most_popular_color() {
                    if border_color == background_color {
                        // println!("skip background object: {:?}", mask_image);
                        continue;
                    }
                }
                
                let mask_neighbour: Image = mask_image.outline_mask_neighbour().expect("image");
    
                // println!("mask_image: {:?}", mask_image);
                // println!("mask_neighbour: {:?}", mask_neighbour);
                let histogram: Histogram = input.histogram_with_mask(&mask_neighbour).expect("histogram");
                let pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
                if pairs.len() != 1 {
                    println!("expected 1 color in the histogram, but got: {:?}", pairs); 
                    continue;
                }
                let outline_color: u8 = histogram.most_popular_color().expect("expected 1 color");
                if outline_color == background_color {
                    // Ignore non-interior objects
                    continue;
                }
                // println!("outline_color: {:?}", outline_color);
                // println!("mask_image: {:?}", mask_image);
                // println!("mask_neighbour: {:?}", mask_neighbour);
    
                // Replace color only for the interior objects
                let mask_inverted: Image = mask_image.invert_mask();
                result_image = mask_inverted.select_from_image(&result_image, replacement_color).expect("image");
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("00d62c1b").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_300000_puzzle_ae3edfdc() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;
    
            let neighbour_up: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Up, color_when_there_is_no_neighbour).expect("image");
            let neighbour_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, color_when_there_is_no_neighbour).expect("image");
            let neighbour_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Right, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Down, color_when_there_is_no_neighbour).expect("image");
    
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for y in 0..(input.height() as i32) {
                for x in 0..(input.width() as i32) {
                    let mask_value: u8 = ignore_mask.get(x, y).unwrap_or(255);
                    if mask_value == 1 {
                        continue;
                    }
    
                    let color_up: u8 = neighbour_up.get(x, y).unwrap_or(255);
                    let color_down: u8 = neighbour_down.get(x, y).unwrap_or(255);
                    let color_left: u8 = neighbour_left.get(x, y).unwrap_or(255);
                    let color_right: u8 = neighbour_right.get(x, y).unwrap_or(255);
    
                    let mut histogram = Histogram::new();
                    if color_up != color_when_there_is_no_neighbour {
                        histogram.increment(color_up);
                    }
                    if color_down != color_when_there_is_no_neighbour {
                        histogram.increment(color_down);
                    }
                    if color_left != color_when_there_is_no_neighbour {
                        histogram.increment(color_left);
                    }
                    if color_right != color_when_there_is_no_neighbour {
                        histogram.increment(color_right);
                    }
    
                    if let Some(count) = histogram.most_popular_count() {
                        if count < 2 {
                            continue;
                        }
                    } else {
                        continue;
                    }
    
                    let color_value: u8 = input.get(x, y).unwrap_or(255);
                    let _ = result_image.set(x, y, color_value);
                    if color_up != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y - 1, color_up);
                    }
                    if color_down != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y + 1, color_down);
                    }
                    if color_left != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x - 1, y, color_left);
                    }
                    if color_right != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x + 1, y, color_right);
                    }
                }
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("ae3edfdc").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_310000_puzzle_1f876c06() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;
    
            let neighbour_up_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_up_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, color_when_there_is_no_neighbour).expect("image");
    
            let mut output: Image = input.clone();
            output.set_pixel_where_two_images_agree(&neighbour_down_left, &neighbour_up_right, color_when_there_is_no_neighbour).expect("ok");
            output.set_pixel_where_two_images_agree(&neighbour_up_left, &neighbour_down_right, color_when_there_is_no_neighbour).expect("ok");
            Ok(output)
        };
        let model: Model = Model::load_testdata("1f876c06").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_1F876C06: &'static str = "
    mov $20,255 ; color when there is no neighbour

    ; ignore mask
    mov $1,$0
    mov $2,$0
    f11 $2,101060 ; most popular color
    f21 $1,101250 ; mask where color is
    ; $2 is most popular color
    ; $1 is the ignore mask

    ; neighbour_up_left
    mov $10,$0
    mov $11,$1
    mov $12,$20
    f31 $10,102064 ; neighbour 'UpLeft'
    mov $3,$10

    ; neighbour_up_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102065 ; neighbour 'UpRight'
    mov $4,$10

    ; neighbour_down_left
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102066 ; neighbour 'DownLeft'
    mov $5,$10

    ; neighbour_down_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102067 ; neighbour 'DownRight'
    mov $6,$10

    ; prepare the output image
    mov $14,$0 ; clone input image

    ; set pixel where the two images agree
    mov $17,$20 ; color to ignore
    mov $16,$5 ; neighbour_down_left
    mov $15,$4 ; neighbour_up_right
    f41 $14,102100 ; set pixel where two images agree

    ; set pixel where the two images agree
    mov $17,$20 ; color to ignore
    mov $16,$6 ; neighbour_down_right
    mov $15,$3 ; neighbour_up_left
    f41 $14,102100 ; set pixel where two images agree

    mov $0,$14
    ";

    #[test]
    fn test_310001_puzzle_1f876c06_loda() {
        let model: Model = Model::load_testdata("1f876c06").expect("model");
        let program = PROGRAM_1F876C06;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_320000_puzzle_623ea044() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;

            let neighbour_up_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_up_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, color_when_there_is_no_neighbour).expect("image");

            let mut result_image: Image = input.clone();
            for y in 0..(input.height() as i32) {
                for x in 0..(input.width() as i32) {
                    let color_up_left: u8 = neighbour_up_left.get(x, y).unwrap_or(255);
                    let color_up_right: u8 = neighbour_up_right.get(x, y).unwrap_or(255);
                    let color_down_left: u8 = neighbour_down_left.get(x, y).unwrap_or(255);
                    let color_down_right: u8 = neighbour_down_right.get(x, y).unwrap_or(255);

                    if color_up_left != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y, color_up_left);
                    }
                    if color_up_right != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y, color_up_right);
                    }
                    if color_down_left != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y, color_down_left);
                    }
                    if color_down_right != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y, color_down_right);
                    }
                }
            }
            Ok(result_image)
        };
        let model: Model = Model::load_testdata("623ea044").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_330000_puzzle_f8b3ba0a() {
        let solution: SolutionSimple = |data| {
            let histogram: Histogram = data.image.histogram_all();
            let histogram_image: Image = histogram.to_image().expect("image");
    
            // Take the row with the colors, discard the row with the counters
            let colors = histogram_image.bottom_rows(1).expect("image");
    
            // Discard the 2 most popular colors
            let trimmed = colors.remove_left_columns(2).expect("image");
    
            let output = trimmed.rotate(1).expect("image");
            Ok(output)
        };
        let model: Model = Model::load_testdata("f8b3ba0a").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 4);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_340000_puzzle_f8ff0b80() {
        let solution: SolutionSimple = |data| {
            let histogram: Histogram = data.image.histogram_all();
            let histogram_image: Image = histogram.to_image().expect("image");
    
            // Take the row with the colors, discard the row with the counters
            let colors = histogram_image.bottom_rows(1).expect("image");
    
            // Discard the 1 most popular color
            let trimmed = colors.remove_left_columns(1).expect("image");
    
            let output = trimmed.rotate(1).expect("image");
            Ok(output)
        };
        let model: Model = Model::load_testdata("f8ff0b80").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_350000_puzzle_a68b268e() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let histogram: Histogram = input.histogram_all();
            let background_color: u8 = histogram.most_popular_color().expect("color");

            let top: Image = input.top_rows(4).expect("image");
            let top_left: Image = top.left_columns(4).expect("image");
            let top_right: Image = top.right_columns(4).expect("image");
            let bottom: Image = input.bottom_rows(4).expect("image");
            let bottom_left: Image = bottom.left_columns(4).expect("image");
            let bottom_right: Image = bottom.right_columns(4).expect("image");

            let mut output: Image = bottom_right;
            output = output.overlay_with_mask_color(&bottom_left, background_color).expect("image");
            output = output.overlay_with_mask_color(&top_right, background_color).expect("image");
            output = output.overlay_with_mask_color(&top_left, background_color).expect("image");
            Ok(output)
        };
        let model: Model = Model::load_testdata("a68b268e").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 6);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_A68B268E: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; W = compute (width-1) / 2
    mov $2,$0
    f11 $2,101000 ; Get width of image
    sub $2,1
    div $2,2

    ; H = compute (height-1) / 2
    mov $3,$0
    f11 $3,101001 ; Get height of image
    sub $3,1
    div $3,2

    ; top left corner of size WxH
    mov $10,$0
    mov $11,$3
    f21 $10,101220 ; get N top rows
    mov $11,$2
    f21 $10,101222 ; get N left columns
  
    ; top right corner of size WxH
    mov $15,$0
    mov $16,$3
    f21 $15,101220 ; get N top rows
    mov $16,$2
    f21 $15,101223 ; get N right columns
  
    ; bottom left corner of size WxH
    mov $20,$0
    mov $21,$3
    f21 $20,101221 ; get N bottom rows
    mov $21,$2
    f21 $20,101222 ; get N left columns

    ; bottom right corner of size WxH
    mov $25,$0
    mov $26,$3
    f21 $25,101221 ; get N bottom rows
    mov $26,$2
    f21 $25,101223 ; get N right columns

    ; zstack where the images are placed on top of each other
    ; zindex 0 - the bottom
    mov $30,$25 ; bottom right

    ; zindex 1
    mov $31,$20 ; bottom left
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    ; zindex 2
    mov $31,$15 ; top right
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    ; zindex 3 - the top
    mov $31,$10 ; top left
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    mov $0,$30
    ";

    #[test]
    fn test_350001_puzzle_a68b268e_loda() {
        let model: Model = Model::load_testdata("a68b268e").expect("model");
        let program = PROGRAM_A68B268E;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 6);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_360000_puzzle_6b9890af() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let histogram: Histogram = input.histogram_all();
            let background_color: u8 = histogram.most_popular_color().expect("color");

            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let mut objects: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, ignore_mask).expect("images");

            if objects.len() != 2 {
                return Err(anyhow::anyhow!("Expected exactly 2 objects, but got a different count"));
            }

            objects.sort_unstable_by(|lhs, rhs| { 
                let a = lhs.mask_count_one();
                let b = rhs.mask_count_one();
                a.cmp(&b)
            });

            let smallest_object: Image = match objects.first() {
                Some(image) => image.clone(),
                None => {
                    return Err(anyhow::anyhow!("Expected an object, but got none"));
                }
            };

            let biggest_object: Image = match objects.last() {
                Some(image) => image.clone(),
                None => {
                    return Err(anyhow::anyhow!("Expected an object, but got none"));
                }
            };

            // Extract the biggest object
            let biggest_image_full: Image = biggest_object.select_from_image(&input, background_color).expect("image");
            let biggest_image: Image = biggest_image_full.trim().expect("image");

            // Extract the smallest object
            let smallest_image_full: Image = smallest_object.select_from_image(&input, background_color).expect("image");
            let smallest_image: Image = smallest_image_full.trim().expect("image");

            let width: u8 = biggest_image.width();
            let x_ratio = width / smallest_image.width();
            let x_ratio_remain = width % smallest_image.width();
            // println!("x_ratio: {} {}", x_ratio, x_ratio_remain);

            let height: u8 = biggest_image.height();
            let y_ratio = height / smallest_image.height();
            let y_ratio_remain = height % smallest_image.height();
            // println!("y_ratio: {} {}", y_ratio, y_ratio_remain);

            if x_ratio != y_ratio {
                return Err(anyhow::anyhow!("Expected same ratio, but different x y ratio: {} {}", x_ratio, y_ratio));
            }

            // Scale up the smallest object so it fits inside the biggest object
            let new_width: u8 = smallest_image.width() * x_ratio;
            let new_height: u8 = smallest_image.height() * y_ratio;
            let fit_image: Image = smallest_image.resize(new_width, new_height).expect("image");
            
            // Overlay the smallest object on top of the biggest object
            let mut output: Image = biggest_image;
            let x = (x_ratio_remain / 2) as i32;
            let y = (y_ratio_remain / 2) as i32;
            output = output.overlay_with_position(&fit_image, x, y).expect("image");

            Ok(output)
        };
        let model: Model = Model::load_testdata("6b9890af").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_370000_puzzle_2281f1f4() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = 0;
            let set_value: u8 = 2;
            let row: Image = input.top_rows(1).expect("image");
            let column: Image = input.right_columns(1).expect("image");
            let mut output: Image = input.clone();
            for y in 0..output.height() {
                for x in 0..output.width() {
                    if y == 0 {
                        continue;
                    }
                    if x + 1 == output.width() {
                        continue;
                    }
                    let value0: u8 = row.get(x as i32, 0).unwrap_or(255); 
                    let value1: u8 = column.get(0, y as i32).unwrap_or(255);
                    if value0 > background_color && value1 > background_color {
                        _ = output.set(x as i32, y as i32, set_value);
                    }
                }
            }
            Ok(output)
        };
        let model: Model = Model::load_testdata("2281f1f4").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_380000_puzzle_d687bc17_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let mut area: Image = input.remove_left_columns(1).expect("image");
            area = area.remove_right_columns(1).expect("image");
            area = area.remove_top_rows(1).expect("image");
            area = area.remove_bottom_rows(1).expect("image");
            let histogram_rows: Vec<Histogram> = area.histogram_rows();
            let histogram_columns: Vec<Histogram> = area.histogram_columns();

            // Empty overlay image with the most popular color
            let most_popular_color: u8 = area.most_popular_color().expect("color");
            let mut overlay: Image = Image::color(area.width(), area.height(), most_popular_color);

            // Draw pixels for histogram_rows
            for (y, histogram_row) in histogram_rows.iter().enumerate() {
                let y1: i32 = (y + 1) as i32;
                let counters: &[u32; 256] = histogram_row.counters();
                {
                    let color: u8 = input.get(0, y1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(i as i32, y as i32, color);
                    }
                }
                {
                    let color: u8 = input.get((input.width() as i32) - 1, y1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set((overlay.width() as i32) - (i + 1) as i32, y as i32, color);
                    }
                }
            }

            // Draw pixels for histogram_columns
            for (x, histogram_column) in histogram_columns.iter().enumerate() {
                let x1: i32 = (x + 1) as i32;
                let counters: &[u32; 256] = histogram_column.counters();
                {
                    let color: u8 = input.get(x1, 0).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(x as i32, i as i32, color);
                    }
                }
                {
                    let color: u8 = input.get(x1, (input.height() as i32) - 1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(x as i32, (overlay.height() as i32) - (i + 1) as i32, color);
                    }
                }
            }

            let output: Image = input.overlay_with_position(&overlay, 1, 1).expect("image");
            Ok(output)
        };
        let model: Model = Model::load_testdata("d687bc17").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 3);
        assert_eq!(result.count_test_correct(), 1);
    }

    #[test]
    fn test_390000_puzzle_5b6cbef5() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = 0;
            let mask: Image = input.to_mask_where_color_is_different(background_color);
            let tile1: Image = input.clone();
            let tile0: Image = Image::color(tile1.width(), tile1.height(), background_color);
            let output: Image = mask.select_two_tiles(&tile0, &tile1)?;
            Ok(output)
        };
        let model: Model = Model::load_testdata("5b6cbef5").expect("model");
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_solution(solution).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_5B6CBEF5: &'static str = "
    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$5 ; tile0
    mov $12,$0 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_390001_puzzle_5b6cbef5_loda() {
        let model: Model = Model::load_testdata("5b6cbef5").expect("model");
        let program = PROGRAM_5B6CBEF5;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 5);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_CCD554AC: &'static str = "
    mov $1,$0
    f11 $1,101000 ; Get width of image

    mov $2,$0
    f11 $2,101001 ; Get height of image

    ; $1 is count x = width of the image
    ; $2 is count y = height of the image
    f31 $0,102120 ; Make a big image by repeating the current image.
    ";

    #[test]
    fn test_400001_puzzle_ccd554ac_loda() {
        let model: Model = Model::load_testdata("ccd554ac").expect("model");
        let program = PROGRAM_CCD554AC;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 6);
        assert_eq!(result.count_test_correct(), 1);
    }

    const PROGRAM_27F8CE4F: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$0 ; tile0
    mov $12,$5 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_410000_puzzle_27f8ce4f_loda() {
        let model: Model = Model::load_testdata("27f8ce4f").expect("model");
        let program = PROGRAM_27F8CE4F;
        let instance = RunWithProgram::new(model, true).expect("RunWithProgram");
        let result: RunWithProgramResult = instance.run_simple(program).expect("result");
        assert_eq!(result.messages(), "");
        assert_eq!(result.count_train_correct(), 4);
        assert_eq!(result.count_test_correct(), 1);
    }
}
