//! Generate a dataset with basic trivial simple ARC tasks.
use super::{Image, HtmlLog, ReverseColorPopularity, ImageRotate90, ImageTryCreate, ExportARCTaskJson};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum Curriculum {
    Small,
    SmallMedium,
    SmallMediumBig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum TwoPixelTransformation {
    LandscapeOrientationFlip,
    LandscapeOrientationRotateCW,
    LandscapeOrientationRotateCCW,
    PortraitOrientationFlip,
    PortraitOrientationRotateCW,
    PortraitOrientationRotateCCW,
    MixedOrientationFlip,
    MixedOrientationRotateCW,
    MixedOrientationRotateCCW,

    // Ideas for more transformations
    // MixedOrientationOutputSolidColor,
    // MixedOrientationRotateOutputSolidColor,
    // LandscapeInputIsOneSolidColorButOutputIsTwoDifferentColors, // needs more than 5 training pairs.
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct DatasetItem {
    curriculum: Curriculum,
    text: String,
}

#[allow(dead_code)]
pub struct GenerateDataset {
    dataset_items: Vec<DatasetItem>,
}

impl GenerateDataset {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            dataset_items: vec!(),
        }
    }

    #[allow(dead_code)]
    fn populate(&mut self, curriculum: Curriculum, number_of_items: u32, print_to_htmllog: bool) -> anyhow::Result<()> {

        for i in 0..number_of_items {
            if print_to_htmllog {
                HtmlLog::text(format!("iteration: {}", i));
            }
            if i % 100000 == 0 {
                println!("iteration: {} number_of_items: {} curriculum: {:?}", i, number_of_items, curriculum);
            }
            let random_seed: u64 = i as u64;

            let transformation: TwoPixelTransformation = match i % 6 {
                0 => TwoPixelTransformation::LandscapeOrientationFlip,
                1 => TwoPixelTransformation::LandscapeOrientationRotateCW,
                2 => TwoPixelTransformation::LandscapeOrientationRotateCCW,
                3 => TwoPixelTransformation::PortraitOrientationFlip,
                4 => TwoPixelTransformation::PortraitOrientationRotateCW,
                5 => TwoPixelTransformation::PortraitOrientationRotateCCW,
                // 6 => TwoPixelTransformation::MixedOrientationFlip,
                // 7 => TwoPixelTransformation::MixedOrientationRotateCW,
                // 8 => TwoPixelTransformation::MixedOrientationRotateCCW,
                _ => unreachable!(),
            };

            let dataset_item: DatasetItem = Self::generate_twopixels(transformation, random_seed, print_to_htmllog)?;
            self.dataset_items.push(dataset_item);
        }

        Ok(())
    }

    /// The two colors inside each pair are always different.
    /// 
    /// The pairs are always different from each other.
    /// 
    /// Each color is only used once.
    fn five_unique_color_pairs(rng: &mut StdRng) -> Vec<(u8, u8)> {
        let mut colors: Vec<u8> = (0..=9).collect();
        colors.shuffle(rng);
        let mut pairs = Vec::<(u8, u8)>::new();
        while colors.len() >= 2 {
            let color0: u8 = colors.remove(0);
            let color1: u8 = colors.remove(0);
            pairs.push((color0, color1));
        }
        assert!(pairs.len() == 5);
        pairs
    }

    fn generate_twopixels(transformation: TwoPixelTransformation, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let insert_same_color_when_reaching_this_limit: u8 = 50;
        let insert_same_value: u8 = rng.gen_range(0..=100);

        let pair_count_values: Vec<(u8, u8)> = vec![
            (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3), (4, 1), (4, 2), (4, 3)
        ];
        let (train_count, test_count) = *pair_count_values.choose(&mut rng).unwrap();
        let pair_count: u8 = train_count + test_count;

        // There are max 4 `train` pairs. Since there are 5 unique color pairs, we can be 
        // certain that the `train` pairs have different colors from each other.
        let mut color_pairs: Vec<(u8, u8)> = Self::five_unique_color_pairs(&mut rng);
        color_pairs.truncate(pair_count as usize);

        // Fill up with more random colors until there are enough color pairs.
        // The `test` pairs comes last, so it's ok if they are not as unique as the `train` pairs.
        while color_pairs.len() < pair_count as usize {
            let color0: u8 = rng.gen_range(0..=9);
            let color1: u8 = rng.gen_range(0..=9);
            if color0 == color1 {
                continue;
            }
            if color_pairs.contains(&(color0, color1)) {
                continue;
            }
            color_pairs.push((color0, color1));
        }
        assert!(color_pairs.len() == pair_count as usize);

        // Make one of the `test` pairs slightly ambiguous so it's more tricky to solve.
        // It doesn't make sense when it's a `flip` operation where the two pixels exchange places.
        // then don't make it ambiguous, because it would cause input and output to be identical.
        // we want input and output to always be different.
        let allow_same_color: bool = match transformation {
            TwoPixelTransformation::LandscapeOrientationFlip => false,
            TwoPixelTransformation::LandscapeOrientationRotateCW => true,
            TwoPixelTransformation::LandscapeOrientationRotateCCW => true,
            TwoPixelTransformation::PortraitOrientationFlip => false,
            TwoPixelTransformation::PortraitOrientationRotateCW => true,
            TwoPixelTransformation::PortraitOrientationRotateCCW => true,
            TwoPixelTransformation::MixedOrientationFlip => false,
            TwoPixelTransformation::MixedOrientationRotateCW => true,
            TwoPixelTransformation::MixedOrientationRotateCCW => true,
        };
        if allow_same_color && train_count >= 2 && test_count >= 1 && insert_same_value >= insert_same_color_when_reaching_this_limit {
            // Replace a color_pair so it uses the same color for both its colors, so it's ambiguous and more tricky to solve.
            let index: usize = rng.gen_range(train_count..pair_count) as usize;
            let color: u8 = rng.gen_range(0..=9);
            color_pairs[index] = (color, color);
        }

        if print_to_htmllog {
            HtmlLog::text(format!("pair_count: {}", pair_count));
        }
        let mut export = ExportARCTaskJson::new();
        let mut color_pair_strings = Vec::<String>::new();
        for i in 0..pair_count {
            let is_train: bool = i < train_count;

            // Pick two random colors
            // The colors are always different from each other for the `train` pairs.
            // The colors are sometimes the same and sometimes different for the `test` pairs.
            let (color0, color1) = color_pairs.remove(0);

            let input_landscape: Image = Image::try_create(2, 1, vec![color0, color1])?;
            let input_portrait: Image = input_landscape.rotate_cw()?;

            // Pick either input_landscape or input_portrait based on a random number
            // Make sure that both landscape and portrait orientations are used for the training pairs, so 2 or more train pairs.
            // Make sure that both landscape and portrait orientations are used for the test pairs, so 2 or more test pairs.
            let input_mixed: Image = match rng.gen_range(0..=1) {
                0 => input_landscape.clone(),
                1 => input_portrait.clone(),
                _ => unreachable!(),
            };

            let input: &Image = match transformation {
                TwoPixelTransformation::LandscapeOrientationFlip => &input_landscape,
                TwoPixelTransformation::LandscapeOrientationRotateCW => &input_landscape,
                TwoPixelTransformation::LandscapeOrientationRotateCCW => &input_landscape,
                TwoPixelTransformation::PortraitOrientationFlip => &input_portrait,
                TwoPixelTransformation::PortraitOrientationRotateCW => &input_portrait,
                TwoPixelTransformation::PortraitOrientationRotateCCW => &input_portrait,
                TwoPixelTransformation::MixedOrientationFlip => &input_mixed,
                TwoPixelTransformation::MixedOrientationRotateCW => &input_mixed,
                TwoPixelTransformation::MixedOrientationRotateCCW => &input_mixed,
            };

            let output_reversed: Image = ReverseColorPopularity::apply_to_image(input)?;
            let output_rotate_ccw: Image = input.rotate_ccw()?;
            let output_rotate_cw: Image = input.rotate_cw()?;

            let output: &Image = match transformation {
                TwoPixelTransformation::LandscapeOrientationFlip => &output_reversed,
                TwoPixelTransformation::LandscapeOrientationRotateCW => &output_rotate_cw,
                TwoPixelTransformation::LandscapeOrientationRotateCCW => &output_rotate_ccw,
                TwoPixelTransformation::PortraitOrientationFlip => &output_reversed,
                TwoPixelTransformation::PortraitOrientationRotateCW => &output_rotate_cw,
                TwoPixelTransformation::PortraitOrientationRotateCCW => &output_rotate_ccw,
                TwoPixelTransformation::MixedOrientationFlip => &output_reversed,
                TwoPixelTransformation::MixedOrientationRotateCW => &output_rotate_cw,
                TwoPixelTransformation::MixedOrientationRotateCCW => &output_rotate_ccw,
            };

            if print_to_htmllog {
                HtmlLog::compare_images(vec![input.clone(), output.clone()]);
            }
            assert!(input != output, "input and output must be different");
            if is_train {
                export.push_train(&input, &output);
            } else {
                export.push_test(&input, &output);
            }

            color_pair_strings.push(format!("{}{}", color0, color1));
        }

        let transformation_name: &str = match transformation {
            TwoPixelTransformation::LandscapeOrientationFlip => "landscape_flip",
            TwoPixelTransformation::LandscapeOrientationRotateCW => "landscape_cw",
            TwoPixelTransformation::LandscapeOrientationRotateCCW => "landscape_ccw",
            TwoPixelTransformation::PortraitOrientationFlip => "portrait_flip",
            TwoPixelTransformation::PortraitOrientationRotateCW => "portrait_cw",
            TwoPixelTransformation::PortraitOrientationRotateCCW => "portrait_ccw",
            TwoPixelTransformation::MixedOrientationFlip => "mixed_flip",
            TwoPixelTransformation::MixedOrientationRotateCW => "mixed_cw",
            TwoPixelTransformation::MixedOrientationRotateCCW => "mixed_ccw",
        };

        let color_pair_strings_joined: String = color_pair_strings.join("_");
        let filename: String = format!("two_{}_{}.json", transformation_name, color_pair_strings_joined);


        // let json: String = export.to_string()?;
        // println!("filename: {}", filename);
        // println!("{}", json);

        // filename = "twopixels_mixed_orientations_reverse_colors_53_91_72_08.json";
        // filename = "twopixels_rotate_53_91_72_08.json";
        // filename = "twopixels_flip_53_91_72_08.json";
        // filename = "twopixels_color0withsamesize_53_91_72_08.json";
        // filename = "twopixels_firstcolorwithsamesize_53_91_72_08.json";
        // filename = "twopixels_lastcolorwithsamesize_53_91_72_08.json";
        // filename = "twopixels_fixorientation_53_91_72_08.json";
        // Save task to file
        let basedir: PathBuf = PathBuf::from("/Users/neoneye/Downloads/output");
        let path: PathBuf = basedir.join(&filename);
        // println!("path: {}", path.display());
        export.save_json_file(&path)?;

        let dataset_item: DatasetItem = DatasetItem {
            curriculum: Curriculum::Small,
            text: String::new(),
        };
        Ok(dataset_item)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_five_unique_color_pairs() {
        let actual: Vec<(u8, u8)> = GenerateDataset::five_unique_color_pairs(&mut StdRng::seed_from_u64(0));
        assert_eq!(actual, vec![(5, 2), (9, 1), (6, 3), (4, 0), (7, 8)]);
    }

    #[allow(dead_code)]
    // #[test]
    fn test_20000_generate() {
        // Arrange
        let mut generate_dataset = GenerateDataset::new();

        // Act
        generate_dataset.populate(Curriculum::Small, 24, false).expect("ok");

        // Assert
    }
}
