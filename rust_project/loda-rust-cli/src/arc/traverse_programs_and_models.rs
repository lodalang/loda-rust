use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use super::{Prediction, TestItem, TaskItem, Tasks};
use super::{Label, LabelSet, PropertyInput, PropertyOutput};
use super::{Image, Histogram, ImageHistogram, ImageMask, ImageMaskCount, ImageSegment, ImageSegmentAlgorithm, ImageTrim};
use crate::analytics::{AnalyticsDirectory, Analytics};
use crate::config::Config;
use crate::common::{find_json_files_recursively, parse_csv_file, create_csv_file};
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, CreateGenomeMutateContextMode, create_genome_mutate_context, GenomeMutateContext};
use bloomfilter::*;
use anyhow::Context;
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use chrono::prelude::*;
use std::fmt;
use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use console::Style;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

static SOLUTIONS_FILENAME: &str = "solution_notXORdinary.json";

/// There is a penalty if the ARCathon executable is running longer than 24 hours.
/// Some of the solutions takes minutes to evaluate, so the executable cannot stop instantly. 
/// Thus the limit is several minutes shorter so we are sure that the executable has stopped.
static ARC_COMPETITION_EXECUTE_DURATION_SECONDS: u64 = ((23 * 60) + 30) * 60;

static ARC_COMPETITION_INITIAL_RANDOM_SEED: u64 = 1;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum RulePriority {
    Simple,
    Medium,
    Advanced,
}


#[derive(Clone, Debug)]
struct BufferOutput {
    id: String,
    image: Image,
    histogram: Histogram,
    label_set: LabelSet,
}

#[derive(Clone, Debug)]
struct BufferInput {
    id: String,
    image: Image,
    histogram: Histogram,
    label_set: LabelSet,
    input_properties: HashMap<PropertyInput, u8>,

    // TODO: caching of computed properties such as: number of unique colors, background color.
    // TODO: label_set pending to be computed
    // TODO: label_set that cannot be computed
    // TODO: rerun analyze until all pending properties have been computed
}

impl BufferInput {
    fn update_input_properties(&mut self) {
        self.input_properties = self.resolve_input_properties();
    }

    fn resolve_input_properties(&self) -> HashMap<PropertyInput, u8> {
        let width_input: u8 = self.image.width();
        let height_input: u8 = self.image.height();

        let mut width_input_plus1: Option<u8> = None;
        {
            let value: u16 = (width_input as u16) + 1;
            if value <= (u8::MAX as u16) {
                width_input_plus1 = Some(value as u8);
            }
        }

        let mut height_input_plus1: Option<u8> = None;
        {
            let value: u16 = (height_input as u16) + 1;
            if value <= (u8::MAX as u16) {
                height_input_plus1 = Some(value as u8);
            }
        }

        let mut width_input_plus2: Option<u8> = None;
        {
            let value: u16 = (width_input as u16) + 2;
            if value <= (u8::MAX as u16) {
                width_input_plus2 = Some(value as u8);
            }
        }

        let mut height_input_plus2: Option<u8> = None;
        {
            let value: u16 = (height_input as u16) + 2;
            if value <= (u8::MAX as u16) {
                height_input_plus2 = Some(value as u8);
            }
        }

        let mut width_input_minus1: Option<u8> = None;
        {
            if width_input >= 1 {
                width_input_minus1 = Some(width_input - 1);
            }
        }

        let mut height_input_minus1: Option<u8> = None;
        {
            if height_input >= 1 {
                height_input_minus1 = Some(height_input - 1);
            }
        }
        
        let mut width_input_minus2: Option<u8> = None;
        {
            if width_input >= 2 {
                width_input_minus2 = Some(width_input - 2);
            }
        }

        let mut height_input_minus2: Option<u8> = None;
        {
            if height_input >= 2 {
                height_input_minus2 = Some(height_input - 2);
            }
        }

        let input_unique_color_count_raw: u32 = self.histogram.number_of_counters_greater_than_zero();
        let mut input_unique_color_count: Option<u8> = None;
        if input_unique_color_count_raw <= (u8::MAX as u32) {
            input_unique_color_count = Some(input_unique_color_count_raw as u8);
        }

        let mut input_unique_color_count_minus1: Option<u8> = None;
        if let Some(value) = input_unique_color_count {
            if value >= 1 {
                input_unique_color_count_minus1 = Some(value - 1);
            }
        }

        let mut input_number_of_pixels_with_most_popular_color: Option<u8> = None;
        let mut input_number_of_pixels_with_2nd_most_popular_color: Option<u8> = None;
        let histogram_pairs: Vec<(u32,u8)> = self.histogram.pairs_descending();
        for (histogram_index, histogram_pair) in histogram_pairs.iter().enumerate() {
            if histogram_index >= 2 {
                break;
            }
            let pixel_count: u32 = histogram_pair.0;
            if pixel_count <= (u8::MAX as u32) {
                if histogram_index == 0 {
                    input_number_of_pixels_with_most_popular_color = Some(pixel_count as u8);
                }
                if histogram_index == 1 {
                    input_number_of_pixels_with_2nd_most_popular_color = Some(pixel_count as u8);
                }
            }
        }

        let mut dict = HashMap::<PropertyInput, u8>::new();
        dict.insert(PropertyInput::InputWidth, width_input);
        dict.insert(PropertyInput::InputHeight, height_input);
        if let Some(value) = width_input_plus1 {
            dict.insert(PropertyInput::InputWidthPlus1, value);
        }
        if let Some(value) = width_input_plus2 {
            dict.insert(PropertyInput::InputWidthPlus2, value);
        }
        if let Some(value) = width_input_minus1 {
            dict.insert(PropertyInput::InputWidthMinus1, value);
        }
        if let Some(value) = width_input_minus2 {
            dict.insert(PropertyInput::InputWidthMinus2, value);
        }
        if let Some(value) = height_input_plus1 {
            dict.insert(PropertyInput::InputHeightPlus1, value);
        }
        if let Some(value) = height_input_plus2 {
            dict.insert(PropertyInput::InputHeightPlus2, value);
        }
        if let Some(value) = height_input_minus1 {
            dict.insert(PropertyInput::InputHeightMinus1, value);
        }
        if let Some(value) = height_input_minus2 {
            dict.insert(PropertyInput::InputHeightMinus2, value);
        }
        if let Some(value) = input_unique_color_count {
            dict.insert(PropertyInput::InputUniqueColorCount, value);
        }
        if let Some(value) = input_unique_color_count_minus1 {
            dict.insert(PropertyInput::InputUniqueColorCountMinus1, value);
        }
        if let Some(value) = input_number_of_pixels_with_most_popular_color {
            dict.insert(PropertyInput::InputNumberOfPixelsWithMostPopularColor, value);
        }
        if let Some(value) = input_number_of_pixels_with_2nd_most_popular_color {
            dict.insert(PropertyInput::InputNumberOfPixelsWith2ndMostPopularColor, value);
        }
        dict
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BufferPairType {
    Train,
    Test,
}

#[derive(Clone, Debug)]
struct BufferInputOutputPair {
    id: String,
    pair_type: BufferPairType,
    input: BufferInput,
    output: BufferOutput,
    removal_histogram: Histogram,
    insert_histogram: Histogram,
    label_set: LabelSet,
}

#[derive(Clone, Debug)]
struct BufferTask {
    id: String,
    displayName: String,
    pairs: Vec<BufferInputOutputPair>,
    input_histogram_union: Histogram,
    input_histogram_intersection: Histogram,
    output_histogram_union: Histogram,
    output_histogram_intersection: Histogram,
    removal_histogram_intersection: Histogram,
    insert_histogram_intersection: Histogram,
    input_label_set: LabelSet,
    input_properties_intersection: HashMap<PropertyInput, u8>,
    output_label_set: LabelSet,
    meta_label_set: LabelSet,
}

impl BufferTask {
    fn update_input_label_set(&mut self) {
        let mut label_set = LabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            if is_first {
                label_set = pair.input.label_set.clone();
                is_first = false;
                continue;
            }
            label_set = label_set.intersection(&pair.input.label_set).map(|l| l.clone()).collect();
        }
        self.input_label_set = label_set;
    }

    fn update_output_label_set(&mut self) {
        let mut label_set = LabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            if is_first {
                label_set = pair.output.label_set.clone();
                is_first = false;
                continue;
            }
            label_set = label_set.intersection(&pair.output.label_set).map(|l| l.clone()).collect();
        }
        self.output_label_set = label_set;
    }

    fn update_meta_label_set(&mut self) {
        let mut label_set = LabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            if is_first {
                label_set = pair.label_set.clone();
                is_first = false;
                continue;
            }
            label_set = label_set.intersection(&pair.label_set).map(|l| l.clone()).collect();
        }
        self.meta_label_set = label_set;
    }

    fn update_input_properties_intersection(&mut self) {
        let mut input_properties_intersection: HashMap<PropertyInput, u8> = HashMap::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            if is_first {
                input_properties_intersection = pair.input.input_properties.clone();
                is_first = false;
                continue;
            }

            // Intersection between `input_properties_intersection` and `pair.input.input_properties`.
            let mut keys_for_removal: HashSet<PropertyInput> = HashSet::new();
            for key in input_properties_intersection.keys() {
                keys_for_removal.insert(*key);
            }
            for (key, value) in &pair.input.input_properties {
                if let Some(other_value) = input_properties_intersection.get(key) {
                    if *value == *other_value {
                        // Both hashmaps agree about the key and value. This is a keeper.
                        keys_for_removal.remove(key);
                    }
                }
            }
            for key in &keys_for_removal {
                input_properties_intersection.remove(key);
            }
        }
        self.input_properties_intersection = input_properties_intersection;
    }

    fn assign_labels_input_size_output_size(&mut self) {
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            let width_output: u8 = pair.output.image.width();
            let height_output: u8 = pair.output.image.height();

            pair.output.label_set.insert(Label::OutputSizeWidth { width: width_output });
            pair.output.label_set.insert(Label::OutputSizeHeight { height: height_output });
        }
    }

    fn assign_labels_related_to_removal_histogram(&mut self) {
        let removal_pairs: Vec<(u32,u8)> = self.removal_histogram_intersection.pairs_descending();
        if removal_pairs.len() != 1 {
            return;
        }
        let background_color: u8 = match removal_pairs.first() {
            Some((_count, color)) => *color,
            None => {
                return;
            }
        };
                        
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }

            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            // if self.id == "0934a4d8,task" {
            //     HtmlLog::image(&image_mask);
            // }

            // Determine if the removed color is a rectangle
            {
                match image_mask.trim_color(1) {
                    Ok(image) => {
                        // HtmlLog::image(&image);
                        let mass: u32 = image.mask_count_one();
                        if mass == 0 {
                            // println!("this is a rectangle");
                            pair.input.input_properties.insert(PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval, image.width());
                            pair.input.input_properties.insert(PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval, image.height());
                        } else {
                            // println!("this is not a rectangle");
                        }
                    },
                    Err(_) => {}
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            // let result = image_mask.find_objects(ImageSegmentAlgorithm::All);
            let result = image_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            // println!("number of objects: {} task: {}", object_images.len(), self.displayName);
            // if self.id == "8a371977,task" {
            //     for image in &object_images {
            //         HtmlLog::image(image);
            //     }
            // }
            let mut mass_max: u32 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u32 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u32) {
                let mass_value: u8 = mass_max as u8;
                pair.input.input_properties.insert(PropertyInput::InputMassOfPrimaryObjectAfterSingleColorRemoval, mass_value);
            }

            if let Some(index) = found_index_mass_max {
                if let Some(image) = object_images.get(index) {

                    let trimmed_image: Image = match image.trim_color(0) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    
                    let width: u8 = trimmed_image.width();
                    let height: u8 = trimmed_image.height();
                    // println!("biggest object: {}x{}", width, height);

                    pair.input.input_properties.insert(PropertyInput::InputWidthOfPrimaryObjectAfterSingleColorRemoval, width);
                    pair.input.input_properties.insert(PropertyInput::InputHeightOfPrimaryObjectAfterSingleColorRemoval, height);
                }
            }
        }
    }

    fn assign_labels_related_to_input_histogram_intersection(&mut self) {
        let removal_pairs: Vec<(u32,u8)> = self.input_histogram_intersection.pairs_descending();
        if removal_pairs.len() != 1 {
            return;
        }
        let background_color: u8 = match removal_pairs.first() {
            Some((_count, color)) => *color,
            None => {
                return;
            }
        };
                        
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }

            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            // if self.id == "28bf18c6,task" {
            //     HtmlLog::image(&image_mask);
            // }
            {
                let mass: u32 = image_mask.mask_count_zero();
                if mass > 0 && mass <= (u8::MAX as u32) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }
            {
                let mass: u32 = image_mask.mask_count_one();
                if mass > 0 && mass <= (u8::MAX as u32) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            // let result = image_mask.find_objects(ImageSegmentAlgorithm::All);
            let result = image_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            // println!("number of objects: {} task: {}", object_images.len(), self.displayName);
            // if self.id == "28bf18c6,task" {
            //     for image in &object_images {
            //         HtmlLog::image(image);
            //     }
            // }
            let mut mass_max: u32 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u32 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u32) {
                let mass_value: u8 = mass_max as u8;
                pair.input.input_properties.insert(PropertyInput::InputMassOfPrimaryObjectAfterSingleIntersectionColor, mass_value);
            }

            if let Some(index) = found_index_mass_max {
                if let Some(image) = object_images.get(index) {

                    let trimmed_image: Image = match image.trim_color(0) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    
                    let width: u8 = trimmed_image.width();
                    let height: u8 = trimmed_image.height();
                    // println!("biggest object: {}x{}", width, height);

                    pair.input.input_properties.insert(PropertyInput::InputWidthOfPrimaryObjectAfterSingleIntersectionColor, width);
                    pair.input.input_properties.insert(PropertyInput::InputHeightOfPrimaryObjectAfterSingleIntersectionColor, height);
                }
            }
        }
    }

    fn assign_labels(&mut self) -> anyhow::Result<()> {
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }
            pair.input.update_input_properties();
        }
        self.update_input_properties_intersection();
        self.assign_labels_input_size_output_size();
        self.assign_labels_related_to_removal_histogram();
        self.assign_labels_related_to_input_histogram_intersection();


        let input_properties: [PropertyInput; 24] = [
            PropertyInput::InputWidth, 
            PropertyInput::InputWidthPlus1, 
            PropertyInput::InputWidthPlus2, 
            PropertyInput::InputWidthMinus1, 
            PropertyInput::InputWidthMinus2, 
            PropertyInput::InputHeight,
            PropertyInput::InputHeightPlus1,
            PropertyInput::InputHeightPlus2,
            PropertyInput::InputHeightMinus1,
            PropertyInput::InputHeightMinus2,
            PropertyInput::InputUniqueColorCount,
            PropertyInput::InputUniqueColorCountMinus1,
            PropertyInput::InputNumberOfPixelsWithMostPopularColor,
            PropertyInput::InputNumberOfPixelsWith2ndMostPopularColor,
            PropertyInput::InputWidthOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputHeightOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputMassOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputWidthOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputHeightOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputMassOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor,
            PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor,
            PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval,
            PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval,
        ];
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        for pair in &mut self.pairs {
            if pair.pair_type == BufferPairType::Test {
                continue;
            }

            let width_output: u8 = pair.output.image.width();
            let height_output: u8 = pair.output.image.height();

            for input_property in &input_properties {
                let input_value_option: Option<&u8> = pair.input.input_properties.get(input_property);
                let input_value: u8 = match input_value_option {
                    Some(value) => *value,
                    None => {
                        continue;
                    }
                };
                // TODO: skip, if input_property is not yet computed
                // TODO: skip, if input_property is cannot be computed
                // TODO: save the computed input_property in HashSet

                for output_property in &output_properties {
                    let output_value: u8 = match output_property {
                        PropertyOutput::OutputWidth => width_output,
                        PropertyOutput::OutputHeight => height_output,
                    };
                    let input_image_size: u8 = match output_property {
                        PropertyOutput::OutputWidth => pair.input.image.width(),
                        PropertyOutput::OutputHeight => pair.input.image.height(),
                    };
                    // TODO: skip, if output_property is not yet computed
                    // TODO: skip, if output_property is cannot be computed
                    // TODO: save the computed output_property in HashSet
    
                    let is_same = input_value == output_value;
                    if is_same {
                        let label = Label::OutputPropertyIsEqualToInputProperty { output: *output_property, input: *input_property };
                        pair.label_set.insert(label);
                    }

                    for scale in 2..8u8 {
                        let input_value_scaled: u32 = (input_value as u32) * (scale as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyMultipliedBy { output: *output_property, input: *input_property, scale };
                            pair.label_set.insert(label0);
                            let label1 = Label::OutputPropertyIsInputPropertyMultipliedBySomeScale { output: *output_property, input: *input_property };
                            pair.label_set.insert(label1);
                            break;
                        }
                    }

                    for scale in 2..8u8 {
                        let value: u32 = (input_value as u32) / (scale as u32);
                        let value_remain: u32 = (input_value as u32) % (scale as u32);
                        if value_remain == 0 && value == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyDividedBy { output: *output_property, input: *input_property, scale };
                            pair.label_set.insert(label0);
                            let label1 = Label::OutputPropertyIsInputPropertyDividedBySomeScale { output: *output_property, input: *input_property };
                            pair.label_set.insert(label1);
                            break;
                        }
                    }

                    {
                        let input_value_scaled: u32 = (input_value as u32) * (input_image_size as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output: *output_property, input: *input_property };
                            pair.label_set.insert(label0);
                        }
                    }
                }
            }

        }

        self.update_input_label_set();
        self.update_output_label_set();
        self.update_meta_label_set();

        for label in &self.output_label_set {
            match label {
                Label::OutputSizeWidth { width } => {
                    let label = Label::OutputPropertyIsConstant { 
                        output: PropertyOutput::OutputWidth, 
                        value: *width,
                        reason: "All the training outputs have this width".to_string()
                    };
                    self.meta_label_set.insert(label);
                },
                Label::OutputSizeHeight { height } => {
                    let label = Label::OutputPropertyIsConstant { 
                        output: PropertyOutput::OutputHeight, 
                        value: *height,
                        reason: "All the training outputs have this height".to_string()
                    };
                    self.meta_label_set.insert(label);
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn output_size_rules_for(&self, property_output: &PropertyOutput) -> Vec<String> {
        let mut rules: Vec<String> = vec!();

        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for label in &self.output_label_set {
            match label {
                Label::OutputSizeWidth { width } => {
                    found_width = Some(*width);
                },
                Label::OutputSizeHeight { height } => {
                    found_height = Some(*height);
                },
                _ => {}
            }
        }

        match property_output {
            PropertyOutput::OutputWidth => {
                if let Some(width) = found_width {
                    // TODO: instead of using `a` as prefix to rank the confidence, then use a priority enum.
                    let s = format!("a width is always {:?}", width);
                    rules.push(s);
                }
            },
            PropertyOutput::OutputHeight => {
                if let Some(height) = found_height {
                    let s = format!("a height is always {:?}", height);
                    rules.push(s);
                }
            }
        };


        for label in &self.meta_label_set {
            match label {
                Label::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    // TODO: instead of using `b` as prefix to rank the confidence, then use a priority enum.
                    let s = format!("b {:?} = {:?}", output, input);
                    rules.push(s);
                },
                Label::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("c {:?} = {:?} * {}", output, input, scale);
                    rules.push(s);
                },
                Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_name: &str = match property_output {
                        PropertyOutput::OutputWidth => "InputWidth",
                        PropertyOutput::OutputHeight => "InputHeight"
                    };
                    let s = format!("c {:?} = {:?} * {}", output, input, input_name);
                    rules.push(s);
                },
                Label::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("c {:?} = {:?} / {}", output, input, scale);
                    rules.push(s);
                },
                _ => {}
            }
        }
        rules.sort();
        rules
    }

    fn estimated_output_size(&self) -> String {
        // TODO: make a fitness function of what combo of labels leads to what output
        // TODO: loop over all the puzzles and update the scoring of the most significant labels
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        let mut rules_vec: Vec<String> = vec!();
        for output_property in &output_properties {
            let rules: Vec<String> = self.output_size_rules_for(output_property);
            if rules.is_empty() {
                break;
            }
            let name: &str = match output_property {
                PropertyOutput::OutputWidth => "width",
                PropertyOutput::OutputHeight => "height"
            };
            let combined_rule = format!("{}: {}", name, rules.join(", "));
            rules_vec.push(combined_rule);
        }
        if rules_vec.len() == output_properties.len() {
            let rules_pretty: String = rules_vec.join("<br>");
            return rules_pretty;
        }

        "Undecided".to_string()
    }


    /// Returns an array of tuples. Each tuple is a priority and a value.
    fn predict_output_size_for_output_property_and_input(&self, property_output: &PropertyOutput, buffer_input: &BufferInput) -> Vec<(RulePriority, u8)> {
        let mut rules: Vec<(RulePriority, u8)> = vec!();

        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for label in &self.output_label_set {
            match label {
                Label::OutputSizeWidth { width } => {
                    found_width = Some(*width);
                },
                Label::OutputSizeHeight { height } => {
                    found_height = Some(*height);
                },
                _ => {}
            }
        }

        match property_output {
            PropertyOutput::OutputWidth => {
                if let Some(width) = found_width {
                    rules.push((RulePriority::Medium, width));
                }
            },
            PropertyOutput::OutputHeight => {
                if let Some(height) = found_height {
                    rules.push((RulePriority::Medium, height));
                }
            }
        };

        let dict: HashMap<PropertyInput, u8> = buffer_input.resolve_input_properties();
        for label in &self.meta_label_set {
            match label {
                Label::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let mut priority = RulePriority::Medium;
                    if *output == PropertyOutput::OutputWidth && *input == PropertyInput::InputWidth {
                        priority = RulePriority::Simple;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == PropertyInput::InputHeight {
                        priority = RulePriority::Simple;
                    }
                    rules.push((priority, input_value));
                },
                Label::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let computed_value: u32 = (input_value as u32) * (*scale as u32);
                    if computed_value > (u8::MAX as u32) {
                        continue;
                    }
                    let value: u8 = computed_value as u8;
                    rules.push((RulePriority::Advanced, value));
                },
                Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let input_size: u8 = match property_output {
                        PropertyOutput::OutputWidth => buffer_input.image.width(),
                        PropertyOutput::OutputHeight => buffer_input.image.height()
                    };
                    let computed_value: u32 = (input_value as u32) * (input_size as u32);
                    if computed_value > (u8::MAX as u32) {
                        continue;
                    }
                    let value: u8 = computed_value as u8;
                    rules.push((RulePriority::Advanced, value));
                },
                Label::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let computed_value_remain: u8 = input_value % (*scale);
                    if computed_value_remain != 0 {
                        continue;
                    }
                    let computed_value: u8 = input_value / (*scale);
                    if computed_value < 1 {
                        continue;
                    }
                    rules.push((RulePriority::Advanced, computed_value));
                },
                _ => {}
            }
        }
        if rules.is_empty() {
            return vec!();
        }

        // Simplest rules first, Advanced rules last
        rules.sort();

        rules
    }

    fn predict_output_size_for_input(&self, input: &BufferInput) -> String {
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for output_property in &output_properties {
            let rules: Vec<(RulePriority, u8)> = self.predict_output_size_for_output_property_and_input(output_property, input);

            // pick the simplest rule
            let value: u8 = match rules.first() {
                Some((_prio, value)) => *value,
                None => {
                    break;
                }
            };

            // TODO: compute confidence score, if there are many advanced items that agree on a value, then it may be more likely
            // if there is one simple rule, and no advanced rules, then it may be the most likely
            // if all the rules agree on a single value, then it may be the most likely.
            // If there is a `IsSquare` label, then prefer the square above other choices
            // let strings: Vec<String> = rules.iter().map(|(_prio,value)| format!("{}", value)).collect();

            match output_property {
                PropertyOutput::OutputWidth => { found_width = Some(value) },
                PropertyOutput::OutputHeight => { found_height = Some(value) }
            }
        }

        match (found_width, found_height) {
            (Some(width), Some(height)) => {
                return format!("{}x{}", width, height);
            },
            _ => {
                return "Undecided".to_string()
            }
        }
    }
}

impl TryFrom<&Model> for BufferTask {
    type Error = anyhow::Error;

    fn try_from(model: &Model) -> Result<Self, Self::Error> {
        let model_identifier: String = model.id().identifier();
        let mut result_pairs: Vec<BufferInputOutputPair> = vec!();

        let mut input_histogram_union: Histogram = Histogram::new();
        let mut input_histogram_intersection: Histogram = Histogram::new();
        let mut output_histogram_union: Histogram = Histogram::new();
        let mut output_histogram_intersection: Histogram = Histogram::new();
        let mut removal_histogram_intersection: Histogram = Histogram::new();
        let mut insert_histogram_intersection: Histogram = Histogram::new();
        {
            let pairs: Vec<ImagePair> = model.images_train()?;
            for (index, pair) in pairs.iter().enumerate() {
                let histogram_input: Histogram = pair.input.histogram_all();
                let histogram_output: Histogram = pair.output.histogram_all();

                let mut histogram_removal: Histogram = histogram_input.clone();
                histogram_removal.subtract_histogram(&histogram_output);

                let mut histogram_insert: Histogram = histogram_output.clone();
                histogram_insert.subtract_histogram(&histogram_input);

                input_histogram_union.add_histogram(&histogram_input);
                output_histogram_union.add_histogram(&histogram_output);
                if index == 0 {
                    input_histogram_intersection = histogram_input.clone();
                    output_histogram_intersection = histogram_output.clone();
                    removal_histogram_intersection = histogram_removal.clone();
                    insert_histogram_intersection = histogram_insert.clone();
                } else {
                    input_histogram_intersection.intersection_histogram(&histogram_input);
                    output_histogram_intersection.intersection_histogram(&histogram_output);
                    removal_histogram_intersection.intersection_histogram(&histogram_removal);
                    insert_histogram_intersection.intersection_histogram(&histogram_insert);
                }
                let buffer_input = BufferInput {
                    id: format!("{},input{},train", model_identifier, index),
                    image: pair.input.clone(),
                    histogram: histogram_input,
                    label_set: LabelSet::new(),
                    input_properties: HashMap::new(),
                };
                let buffer_output = BufferOutput {
                    id: format!("{},output{},train", model_identifier, index),
                    image: pair.output.clone(),
                    histogram: histogram_output,
                    label_set: LabelSet::new(),
                };
                let result_pair = BufferInputOutputPair {
                    id: format!("{},pair{},train", model_identifier, index),
                    pair_type: BufferPairType::Train,
                    input: buffer_input,
                    output: buffer_output,
                    removal_histogram: histogram_removal,
                    insert_histogram: histogram_insert,
                    label_set: LabelSet::new(),
                };
                result_pairs.push(result_pair);
            }
        }
        {
            let pairs: Vec<ImagePair> = model.images_test()?;
            for (index, pair) in pairs.iter().enumerate() {
                let histogram_input: Histogram = pair.input.histogram_all();
                let histogram_output: Histogram = pair.output.histogram_all();
                let buffer_input = BufferInput {
                    id: format!("{},input{},test", model_identifier, index),
                    image: pair.input.clone(),
                    histogram: histogram_input,
                    label_set: LabelSet::new(),
                    input_properties: HashMap::new(),
                };
                let buffer_output = BufferOutput {
                    id: format!("{},output{},test", model_identifier, index),
                    image: pair.output.clone(),
                    histogram: histogram_output,
                    label_set: LabelSet::new(),
                };
                let result_pair = BufferInputOutputPair {
                    id: format!("{},pair{},test", model_identifier, index),
                    pair_type: BufferPairType::Test,
                    input: buffer_input,
                    output: buffer_output,
                    removal_histogram: Histogram::new(),
                    insert_histogram: Histogram::new(),
                    label_set: LabelSet::new(),
                };
                result_pairs.push(result_pair);
            }
        }
    
        let task = BufferTask {
            id: format!("{},task", model_identifier),
            displayName: model_identifier,
            pairs: result_pairs,
            input_histogram_union,
            input_histogram_intersection,
            output_histogram_union,
            output_histogram_intersection,
            removal_histogram_intersection,
            insert_histogram_intersection,
            input_label_set: LabelSet::new(),
            input_properties_intersection: HashMap::new(),
            output_label_set: LabelSet::new(),
            meta_label_set: LabelSet::new(),
        };
        return Ok(task);
    }
}


pub struct TraverseProgramsAndModels {
    config: Config,
    arc_config: RunArcCompetitionConfig,
    context: GenomeMutateContext,
    model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    locked_instruction_hashset: HashSet<String>,
    dependency_manager: DependencyManager,
}

impl TraverseProgramsAndModels {
    pub fn arc_competition() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.run_arc_competition()?;
        Ok(())
    }

    pub fn eval_single_puzzle_with_all_existing_solutions(pattern: String) -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.eval_single_puzzle_with_all_existing_solutions_inner(&pattern)?;
        Ok(())
    }

    pub fn check_all_existing_solutions() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.check_all_existing_solutions_inner()?;
        Ok(())
    }

    /// Compare all puzzles with all solutions and output a CSV file
    pub fn generate_solution_csv() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.generate_solution_csv_inner()?;
        Ok(())
    }

    /// Traverse all puzzles and classify each puzzle.
    pub fn label_all_puzzles() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;

        let mut buffer_task_vec: Vec<BufferTask> = vec!();
        for model_item in &instance.model_item_vec {
            let model: Model = model_item.borrow().model.clone();
            let mut buffer_task: BufferTask = BufferTask::try_from(&model)?;
            buffer_task.assign_labels()?;
            buffer_task_vec.push(buffer_task);
        }

        let mut count_good = 0;
        let mut count_undecided = 0;
        for buffer_task in &buffer_task_vec {
            let estimate: String = buffer_task.estimated_output_size();
            if estimate == "Undecided" {
                count_undecided += 1;
                continue;
            }
            count_good += 1;
        }
        println!("Estimated output size. good: {}  missing: {}", count_good, count_undecided);
        
        // Compute the output size with the test data, and compare with the expected output
        let mut count_predict_correct: usize = 0;
        let mut count_predict_incorrect: usize = 0;
        let mut count_predict_correct_task: usize = 0;
        let mut count_predict_incorrect_task: usize = 0;
        for buffer_task in &buffer_task_vec {
            let estimate: String = buffer_task.estimated_output_size();
            if estimate == "Undecided" {
                continue;
            }

            let mut all_correct = true;
            for pair in &buffer_task.pairs {
                if pair.pair_type != BufferPairType::Test {
                    continue;
                }

                // TODO: update input properties for this test, otherwise the input properties aren't available for computing the predicted size.
                // buffer_task.update_input_properties_intersection();
                // buffer_task.assign_labels_input_size_output_size();
                // buffer_task.assign_labels_related_to_removal_histogram();
                // buffer_task.assign_labels_related_to_input_histogram_intersection();
        
                let predicted: String = buffer_task.predict_output_size_for_input(&pair.input);
                let expected: String = format!("{}x{}", pair.output.image.width(), pair.output.image.height());
                if predicted == expected {
                    count_predict_correct += 1;
                } else {
                    println!("Wrong output size. Expected {}, but got {}. Task id: {}", expected, predicted, buffer_task.id);
                    count_predict_incorrect += 1;
                    all_correct = false;
                }
            }
            if all_correct {
                count_predict_correct_task += 1;
            } else {
                // Self::inspect_task(buffer_task)?;
                count_predict_incorrect_task += 1;
            }
        }
        {
            let percent: usize = (100 * count_predict_correct) / (count_predict_correct + count_predict_incorrect).max(1);
            println!("Predicted single-image: correct: {} incorrect: {} correct-percent: {}%", count_predict_correct, count_predict_incorrect, percent);
        }
        {
            let percent: usize = (100 * count_predict_correct_task) / (count_predict_correct_task + count_predict_incorrect_task).max(1);
            println!("Predicted task: correct: {} incorrect: {} correct-percent: {}%", count_predict_correct_task, count_predict_incorrect_task, percent);
        }

        Self::inspect_undecided(&buffer_task_vec)?;
        // Self::inspect_decided(&buffer_task_vec)?;
        // Self::inspect_task_id(&buffer_task_vec, "28bf18c6,task")?;
        Ok(())
    }

    fn inspect_undecided(buffer_task_vec: &Vec<BufferTask>) -> anyhow::Result<()> {
        let mut count = 0;
        for buffer_task in buffer_task_vec {
            let estimate: String = buffer_task.estimated_output_size();
            if estimate != "Undecided" {
                continue;
            }
            if count > 0 {
                Self::inspect_task(buffer_task)?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        Ok(())
    }

    fn inspect_decided(buffer_task_vec: &Vec<BufferTask>) -> anyhow::Result<()> {
        let mut count = 0;
        for buffer_task in buffer_task_vec {
            let estimate: String = buffer_task.estimated_output_size();
            if estimate == "Undecided" {
                continue;
            }
            if count > 0 {
                Self::inspect_task(buffer_task)?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        Ok(())
    }

    fn inspect_task_id(buffer_task_vec: &Vec<BufferTask>, task_id: &str) -> anyhow::Result<()> {
        let mut count = 0;
        for buffer_task in buffer_task_vec {
            if buffer_task.id == task_id {
                Self::inspect_task(buffer_task)?;
                break;
            }
        }
        Ok(())
    }

    fn labelset_to_html(label_set: &LabelSet) -> String {
        let mut label_vec: Vec<String> = label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", label_vec.join(""))
    }

    fn input_properties_to_html(input_properties: &HashMap<PropertyInput, u8>) -> String {
        let mut items: Vec<String> = input_properties.iter().map(|(key,value)| format!("{:?} {}", key, value)).collect();
        if items.is_empty() {
            return "empty".to_string();
        }
        items.sort();
        let list_vec: Vec<String> = items.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", list_vec.join(""))
    }

    fn inspect_task(buffer_task: &BufferTask) -> anyhow::Result<()> {
        let mut row_title: String = "<tr><td></td>".to_string();
        let mut row_input_image: String = "<tr><td>Input image</td>".to_string();
        let mut row_input_labels: String = "<tr><td>Input labels</td>".to_string();
        let mut row_input_properties: String = "<tr><td>Input properties</td>".to_string();
        let mut row_output_image: String = "<tr><td>Output image</td>".to_string();
        let mut row_output_labels: String = "<tr><td>Output labels</td>".to_string();
        let mut row_action: String = "<tr><td>Action</td>".to_string();
        let mut row_meta_labels: String = "<tr><td>Meta labels</td>".to_string();
        for pair in &buffer_task.pairs {
            {
                row_title += "<td>";
                let title: &str = match pair.pair_type {
                    BufferPairType::Train => "Train",
                    BufferPairType::Test => "Test",
                };
                row_title += title;
                row_title += "</td>";
            }
            {
                row_input_image += "<td>";
                row_input_image += &pair.input.image.to_html();
                row_input_image += "</td>";
            }
            {
                row_input_labels += "<td>";
                row_input_labels += &Self::labelset_to_html(&pair.input.label_set);
                row_input_labels += "</td>";
            }
            {
                row_input_properties += "<td>";
                row_input_properties += &Self::input_properties_to_html(&pair.input.input_properties);
                row_input_properties += "</td>";
            }
            {
                row_output_image += "<td>";
                row_output_image += &pair.output.image.to_html();
                row_output_image += "</td>";
            }
            {
                row_output_labels += "<td>";
                row_output_labels += &Self::labelset_to_html(&pair.output.label_set);
                row_output_labels += "</td>";
            }
            {
                row_action += "<td>Removal<br>";
                match pair.removal_histogram.color_image() {
                    Ok(image) => {
                        row_action += &image.to_html();
                    },
                    Err(_) => {
                        row_action += "N/A";
                    }
                }
                row_action += "<br>Insert<br>";
                match pair.insert_histogram.color_image() {
                    Ok(image) => {
                        row_action += &image.to_html();
                    },
                    Err(_) => {
                        row_action += "N/A";
                    }
                }
                row_action += "</td>";
            }
            {
                row_meta_labels += "<td>";
                row_meta_labels += &Self::labelset_to_html(&pair.label_set);
                row_meta_labels += "</td>";
            }
        }

        row_title += "<td>Analysis</td>";

        row_input_image += "<td>Union<br>";
        match buffer_task.input_histogram_union.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "<br><br>Intersection<br>";
        match buffer_task.input_histogram_intersection.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "</td>";

        row_input_labels += "<td>";
        row_input_labels += &Self::labelset_to_html(&buffer_task.input_label_set);
        row_input_labels += "</td>";

        row_input_properties += "<td>";
        row_input_properties += &Self::input_properties_to_html(&buffer_task.input_properties_intersection);
        row_input_properties += "</td>";

        row_output_image += "<td>Union<br>";
        match buffer_task.output_histogram_union.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "<br><br>Intersection<br>";
        match buffer_task.output_histogram_intersection.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "</td>";

        row_output_labels += "<td>";
        row_output_labels += &Self::labelset_to_html(&buffer_task.output_label_set);
        row_output_labels += "</td>";

        row_action += "<td>Removal<br>";
        match buffer_task.removal_histogram_intersection.color_image() {
            Ok(image) => {
                row_action += &image.to_html();
            },
            Err(_) => {
                row_action += "N/A";
            }
        }
        row_action += "<br>Insert<br>";
        match buffer_task.insert_histogram_intersection.color_image() {
            Ok(image) => {
                row_action += &image.to_html();
            },
            Err(_) => {
                row_action += "N/A";
            }
        }
        row_action += "</td>";

        row_meta_labels += "<td>";
        row_meta_labels += &Self::labelset_to_html(&buffer_task.meta_label_set);
        row_meta_labels += "</td>";

        row_title += "</tr>";
        row_input_image += "</tr>";
        row_input_labels += "</tr>";
        row_input_properties += "</tr>";
        row_output_image += "</tr>";
        row_output_labels += "</tr>";
        row_action += "</tr>";
        row_meta_labels += "</tr>";

        let html = format!(
            "<h2>{}</h2><p>Estimate: {}</p><table>{}{}{}{}{}{}{}{}</table>",
            buffer_task.displayName, 
            buffer_task.estimated_output_size(),
            row_title,
            row_input_image, 
            row_input_labels, 
            row_input_properties, 
            row_output_image, 
            row_output_labels, 
            row_action,
            row_meta_labels
        );
        HtmlLog::html(html);
        Ok(())
    }


    fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let arc_config = RunArcCompetitionConfig::new(&config);
        let dependency_manager: DependencyManager = RunWithProgram::create_dependency_manager();

        let mut instance = Self { 
            config,
            arc_config,
            context: GenomeMutateContext::default(),
            model_item_vec: vec!(),
            program_item_vec: vec!(),
            locked_instruction_hashset: HashSet::new(),
            dependency_manager,
        };
        instance.load_puzzle_files()?;
        instance.load_solution_files()?;
        instance.init_locked_instruction_hashset()?;
        Ok(instance)
    }

    fn files_to_keep(path: &PathBuf) -> bool {
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy() == SOLUTIONS_FILENAME {
                debug!("ignoring the SOLUTIONS_FILENAME. path: {:?}", path);
                return false;
            }
        }
        true
    }

    /// Load all the ARC puzzle files into memory
    fn load_puzzle_files(&mut self) -> anyhow::Result<()> {
        let repo_path: PathBuf = self.config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);

        // Ignore the solutions json file, since it's not an ARC puzzle json file
        let paths: Vec<PathBuf> = all_json_paths
            .into_iter()
            .filter(Self::files_to_keep)
            .collect();
        debug!("arc_repository_data. number of json files: {}", paths.len());

        let mut model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for path in &paths {
            let model = match Model::load_with_json_file(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("Ignoring file. Cannot parse arc_json_model file. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
            let instance = ModelItem {
                id: ModelItemId::Path { path: path.clone() },
                model,
            };
            let item = Rc::new(RefCell::new(instance));
            model_item_vec.push(item);
        }
        if model_item_vec.len() != paths.len() {
            error!("Skipped some models. paths.len()={}, but model_item_vec.len()={}", paths.len(), model_item_vec.len());
        }
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    /// Load all `.asm` programs into memory
    fn load_solution_files(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
        debug!("loda_arc_challenge_repository_programs. number of asm files: {}", paths.len());

        let mut program_item_vec: Vec<Rc<RefCell<ProgramItem>>> = vec!();
        for path in &paths {

            let program_string: String = match fs::read_to_string(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot read the file: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let is_simple: bool = program_string.contains("Program Type: simple");
            let is_advanced: bool = program_string.contains("Program Type: advanced");
            let program_type: ProgramType;
            match (is_simple, is_advanced) {
                (false, false) => {
                    error!("Cannot find 'Program Type: simple' nor 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                },
                (false, true) => {
                    program_type = ProgramType::Advance;
                },
                (true, false) => {
                    program_type = ProgramType::Simple;
                },
                (true, true) => {
                    error!("Ambiguous use of 'Program Type'. Should be either 'Program Type: simple' or 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                }
            }

            let program_content: String;
            match program_type {
                ProgramType::Simple => {
                    program_content = RunWithProgram::convert_simple_to_full(&program_string);
                },
                ProgramType::Advance => {
                    program_content = program_string.clone();
                }
            }
            let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_content) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot parse the program. Skipping program. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let program_runner: ProgramRunner = match self.dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot create ProgramRunner. Skipping program. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let instance = ProgramItem {
                id: ProgramItemId::Path { path: path.clone() },
                program_string,
                program_type,
                parsed_program,
                program_runner,
            };
            let item = Rc::new(RefCell::new(instance));
            program_item_vec.push(item);
        }
        if program_item_vec.len() != paths.len() {
            error!("Skipped some programs. paths.len()={}, but program_item_vec.len()={}", paths.len(), program_item_vec.len());
        }
        self.program_item_vec = program_item_vec;
        Ok(())
    }

    const INSTRUCTIONS_TO_LOCK: &'static str = r#"
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
      mov $0,$$81 ; load train[x].input image
      mov $1,$$82 ; load train[x].output image
    
      ; do stuff
      
      ; next iteration
      add $81,10 ; jump to address of next training input image
      add $82,10 ; jump to address of next training output image
    lpe
    "#;

    fn init_locked_instruction_hashset(&mut self) -> anyhow::Result<()> {
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_PRE)?;
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_POST)?;
        self.insert_program_into_locked_instruction_hashset(Self::INSTRUCTIONS_TO_LOCK)?;
        Ok(())
    }

    fn insert_program_into_locked_instruction_hashset<S: AsRef<str>>(&mut self, program: S) -> anyhow::Result<()> {
        let program_str: &str = program.as_ref();
        let parsed_program: ParsedProgram = ParsedProgram::parse_program(program_str)
            .map_err(|e| anyhow::anyhow!("parse with program: {:?}. error: {:?}", program_str, e))?;
        for instruction in &parsed_program.instruction_vec {
            let s: String = instruction.to_string();
            self.locked_instruction_hashset.insert(s);
        }
        Ok(())
    }

    /// Create mutations of a single program.
    /// 
    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    /// 
    /// Returns a vector with length `number_of_programs_to_generate`.
    fn create_mutations_of_program(
        &mut self, 
        program_item: RcProgramItem, 
        mutation_index: u64,
        number_of_programs_to_generate: usize, 
        bloom: &mut Bloom::<String>
    ) -> anyhow::Result<RcProgramItemVec> {
        let mut genome = Genome::new();
        genome.append_message(format!("template: {:?}", program_item.borrow().id.file_name()));

        let mut genome_vec: Vec<GenomeItem> = program_item.borrow().parsed_program.to_genome_item_vec();

        // locking rows that are not to be mutated
        for genome_item in genome_vec.iter_mut() {
            let program_line: String = genome_item.to_line_string();
            if self.locked_instruction_hashset.contains(&program_line) {
                genome_item.set_mutation_locked(true);
            }
        }

        genome.set_genome_vec(genome_vec);
        
        let mut result_program_item_vec: RcProgramItemVec = RcProgramItemVec::with_capacity(number_of_programs_to_generate);

        let max_number_of_iterations = 100;
        for iteration in 0..max_number_of_iterations {

            // Notes about random seed.
            //
            // Originally the random generator was initialized once before entering the loop.
            // The initial random seed was based on datetime.
            // It was non-deterministic, and would yield different results.
            // When new files got added to the solutions repo, then the random seed would change.
            //
            // Lesson learned: Reproducibility is highly valuable. 
            // Reproduce the same results under the same circumstances, makes it possible to compare algorithms.
            // In order to make the code deterministic:
            // The random seed is unaffected of how many files there are. When a new file gets added, it's still the same random_seed.
            // The random generator is reinitialized for every iteration.
            // The random seed is unaffected by how many threads are running in parallel.
            // However there are still several non-deterministic things that may affect the outcome,
            // Such as the analytics file on disk, how are the rows arranged in the csv file. Bloomfilter.
            // Such as the way the Genome::mutate() picks a mutation strategy.
            let random_seed: u64 = mutation_index * 0x10000 + iteration + ARC_COMPETITION_INITIAL_RANDOM_SEED;
            let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

            let mutate_success: bool = genome.mutate(&mut rng, &self.context);
            if !mutate_success {
                continue;
            }

            let parsed_program: ParsedProgram = genome.to_parsed_program();
            let bloom_key: String = parsed_program.to_string();
            if bloom.check(&bloom_key) {
                // It's likely that this program mutation has already has been explored in the past. Ignore it.
                // debug!("skip program mutation that already have been tried out");
                continue;                
            }

            // This program mutation is not contained in the bloomfilter.

            // This ensures that we don't try out this mutation again.
            bloom.set(&bloom_key);
            
            // Proceed making a program out of it.
            let program_runner: ProgramRunner = match self.dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program) {
                Ok(value) => value,
                Err(error) => {
                    error!("ignoring program mutation. parse_stage2 with program: {:?}. error: {:?}", genome.to_string(), error);
                    continue;
                }
            };
    
            // println!("program: {:?} random_seed: {:#x}", program_item.borrow().id.file_name(), random_seed);
            let mut serializer = ProgramSerializer::new();
            serializer.append_comment("Submitted by Simon Strandgaard");
            serializer.append_comment("Program Type: advanced");
            serializer.append_empty_line();
            program_runner.serialize(&mut serializer);
            serializer.append_empty_line();
            for message in genome.message_vec() {
                serializer.append_comment(message);
            }
            serializer.append_empty_line();
            let candidate_program: String = serializer.to_string();
            // println!("; ------\n\n{}", candidate_program);

            let mutated_program_item = ProgramItem {
                id: ProgramItemId::None,
                program_string: candidate_program,
                program_type: ProgramType::Advance,
                parsed_program,
                program_runner,
            };
            result_program_item_vec.push(Rc::new(RefCell::new(mutated_program_item)));
            if result_program_item_vec.len() >= number_of_programs_to_generate {
                return Ok(result_program_item_vec);
            }
        }
        if result_program_item_vec.is_empty() {
            return Err(anyhow::anyhow!("unable to mutate in {} attempts, {:?}", max_number_of_iterations, program_item.borrow().id.file_name()));
        }
        Ok(result_program_item_vec)
    }

    /// Create mutations of all the existing programs.
    /// 
    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    /// 
    /// Returns a vector with length `number_of_programs_to_generate` x number of available programs.
    fn create_mutations_of_all_programs(
        &mut self,
        mutation_index: u64, 
        number_of_programs_to_generate: usize, 
        bloom: &mut Bloom::<String>
    ) -> RcProgramItemVec {
        let mut result_program_item_vec: RcProgramItemVec = RcProgramItemVec::new();
        for program_item in self.program_item_vec.clone() {
            match self.create_mutations_of_program(program_item, mutation_index, number_of_programs_to_generate, bloom) {
                Ok(mut mutated_programs) => {
                    result_program_item_vec.append(&mut mutated_programs);
                },
                Err(error) => {
                    debug!("Skipping mutation. {:?}", error);
                }
            }
        }
        result_program_item_vec
    }

    fn read_solutions_json(&self) -> anyhow::Result<Tasks> {
        let path: &Path = &self.arc_config.path_solution_teamid_json;
        let solution_teamid_json_string: String = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("something went wrong reading the file: {:?} error: {:?}", path, error));
            }
        };
        let tasks: Tasks = match serde_json::from_str(&solution_teamid_json_string) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Could not parse archaton_solution_json file, path: {:?} error: {:?} json: {:?}", path, error, solution_teamid_json_string));
            }
        };
        Ok(tasks)
    }

    fn eval_single_puzzle_with_all_existing_solutions_inner(&self, pattern: &String) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        // Extract the puzzle model
        let mut candidate_model_items = Vec::<ModelItem>::new();
        for model_item in &self.model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if file_stem.contains(pattern) {
                candidate_model_items.push(model_item.borrow().clone());
            }
        }
        // There is supposed to be exactly 1 puzzle with this name.
        if candidate_model_items.len() >= 2 {
            return Err(anyhow::anyhow!("There are {} puzzles that matches the pattern, please specify a longer pattern: {:?}", candidate_model_items.len(), pattern));
        }
        let model_item: ModelItem = match candidate_model_items.pop() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("No puzzle matches the specified pattern: {:?}", pattern));
            }
        };

        let pairs_train: Vec<ImagePair> = model_item.model.images_train().expect("pairs");
        let pairs_test: Vec<ImagePair> = model_item.model.images_test().expect("pairs");
        println!("Evaluating the puzzle: {:?} train-pairs: {} test-pairs: {}", model_item.id, pairs_train.len(), pairs_test.len());

        let mut count_ok: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;

        let pb = ProgressBar::new(self.program_item_vec.len() as u64);
        pb.tick();
        for (program_index, program_item) in self.program_item_vec.iter().enumerate() {
            if program_index > 0 {
                pb.inc(1);
            }

            let instance = RunWithProgram::new(model_item.model.clone(), verify_test_output).expect("RunWithProgram");

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
            let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
            if actual != expected {
                if result.count_train_correct() == pairs_train.len() && result.count_test_correct() != pairs_test.len() {
                    pb.println(format!("Dangerous false positive. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    count_dangerous_false_positive += 1;
                } else {
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("Partial solution. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    }
                }
                if verbose {
                    pb.println(format!("ERROR: in row {}. program: {:?}. Expected {}, but got {}", program_index, program_item, expected, actual));
                }
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
            pb.println(format!("Solution: {:?}", program_item.borrow().id.file_name()));
        }
        pb.finish_and_clear();

        debug!("STATS:");
        debug!("count_partial_match: {}", count_partial_match);
        debug!("count_error_compute: {}", count_error_compute);
        debug!("count_error_incorrect: {}", count_error_incorrect);
        if count_dangerous_false_positive > 0 {
            error!("Encountered {} dangerous false positive solutions. These are unwanted.", count_dangerous_false_positive);
        }

        if count_ok > 0 {
            let green_bold = Style::new().green().bold();        
            let s = format!("Status: Found {} solutions", count_ok);
            println!("{}", green_bold.apply_to(&s));
        } else {
            let green_bold = Style::new().red().bold();        
            println!("{}", green_bold.apply_to("Status: Found no solutions among the existing programs"));
        }
        Ok(())
    }

    fn check_all_existing_solutions_inner(&self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        if !path_solutions_csv.is_file() {
            return Err(anyhow::anyhow!("there is no existing solutions.csv file, so the solutions cannot be checked. path_solutions_csv: {:?}", path_solutions_csv));
        }

        let record_vec: Vec<Record> = Record::load_record_vec(&path_solutions_csv)?;
        debug!("solutions.csv: number of rows: {}", record_vec.len());

        let mut count_ok: usize = 0;
        let mut count_error_other: usize = 0;
        let mut count_error_duplicate: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;

        let mut unique_records = HashSet::<Record>::new();

        let pb = ProgressBar::new(record_vec.len() as u64);
        for (record_index, record) in record_vec.iter().enumerate() {
            if record_index > 0 {
                pb.inc(1);
            }

            // The rows are supposed to be unique
            if unique_records.contains(&record) {
                pb.println(format!("ERROR: in row {}. Expected unique rows, but this is a duplicate.", record_index));
                count_error_duplicate += 1;
                continue;
            }
            unique_records.insert(record.clone());

            // Extract the puzzle model
            let mut candidate_model_items = Vec::<ModelItem>::new();
            for model_item in &self.model_item_vec {
                let file_name: String = model_item.borrow().id.file_name();
                if file_name == record.model_filename {
                    candidate_model_items.push(model_item.borrow().clone());
                }
            }
            // There is supposed to be exactly 1 puzzle with this name.
            if candidate_model_items.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 puzzle for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let model_item: ModelItem = match candidate_model_items.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. Missing puzzle.", record_index));
                    count_error_other += 1;
                    continue;
                }
            };

            // Extract the solution model
            let mut candidate_programs = Vec::<Rc::<RefCell::<ProgramItem>>>::new();
            let program_filename: String = record.program_filename.clone();
            for program_item in &self.program_item_vec {
                let this_file_name: String = program_item.borrow_mut().id.file_name();
                if this_file_name == program_filename {
                    candidate_programs.push(program_item.clone());
                }
            }
            // There is supposed to be exactly 1 solution with this name.
            if candidate_programs.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 solution for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let program_item: Rc<RefCell<ProgramItem>> = match candidate_programs.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. record: {:?}. Missing solution.", record_index, record));
                    count_error_other += 1;
                    continue;
                }
            };
    
            let instance = RunWithProgram::new(model_item.model.clone(), verify_test_output).expect("RunWithProgram");
            let pairs_train: Vec<ImagePair> = model_item.model.images_train().expect("pairs");
            let pairs_test: Vec<ImagePair> = model_item.model.images_test().expect("pairs");

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
            let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
            if actual != expected {
                pb.println(format!("ERROR: in row {}. record: {:?}. Expected {}, but got {}", record_index, record, expected, actual));
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
        }
        pb.finish_and_clear();

        if count_ok == record_vec.len() {
            let green_bold = Style::new().green().bold();        
            println!("{}", green_bold.apply_to("Status: All solutions passes ok"));
        } else {
            println!("count_ok: {}", count_ok);
            println!("count_error_other: {}", count_error_other);
            println!("count_error_duplicate: {}", count_error_duplicate);
            println!("count_error_compute: {}", count_error_compute);
            println!("count_error_incorrect: {}", count_error_incorrect);
            let sum: usize = count_error_other + count_error_duplicate + count_error_compute + count_error_incorrect;
            error!("There are {} errors that needs to be resolved. csv file: {:?}", sum, path_solutions_csv);
        }
        Ok(())
    }

    fn generate_solution_csv_inner(&mut self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        
        let mut unique_records = HashSet::<Record>::new();
        Record::save_solutions_csv(&unique_records, &path_solutions_csv);
        
        let start = Instant::now();
        
        let mut visited_program_paths = HashSet::<PathBuf>::new();
        let mut count_ok: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_incorrect: usize = 0;
        let mut count_compute_error: usize = 0;

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Puzzle  ");
        pb.tick();

        for (model_index, model_item) in self.model_item_vec.iter_mut().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }

            let print_prefix_puzzle_id: String = format!("Puzzle#{} {:?}", model_index, model_item.borrow().id.file_name());

            let model: Model = model_item.borrow().model.clone();
            let pairs_train: Vec<ImagePair> = model.images_train().expect("pairs");
            let pairs_test: Vec<ImagePair> = model.images_test().expect("pairs");

            let instance = RunWithProgram::new(model, verify_test_output).expect("RunWithProgram");
    
            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new( self.program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Solution");
            pb2.tick();
            for (program_index, program_item) in self.program_item_vec.iter_mut().enumerate() {
                if program_index > 0 {
                    pb2.inc(1);
                }

                let result: RunWithProgramResult;
                match program_item.borrow().program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                let program_id: ProgramItemId = program_item.borrow().id.clone();

                if verbose {
                    let s = format!("model: {:?} program: {:?} result: {:?}", model_item.borrow().id, program_id, result);
                    pb.println(s);
                }

                let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
                let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
                if actual != expected {
                    if result.count_train_correct() == pairs_train.len() && result.count_test_correct() != pairs_test.len() {
                        pb.println(format!("{} - Dangerous false positive. Expected {} but got {}. {:?}", print_prefix_puzzle_id, expected, actual, program_id.file_name()));
                        count_dangerous_false_positive += 1;
                        continue;
                    }
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("{} - Partial solution. Expected {} but got {}. {:?}", print_prefix_puzzle_id, expected, actual, program_id.file_name()));
                        continue;
                    }
                    if verbose {
                        pb.println(format!("ERROR: in row {}. program: {:?}. Expected {}, but got {}", program_index, program_item, expected, actual));
                    }
                    count_incorrect += 1;
                    continue;
                }

    
                pb.println(format!("{} - Solution: {:?}", print_prefix_puzzle_id, program_id.file_name()));
                count_ok += 1;
                match program_id {
                    ProgramItemId::Path { path } => {
                        visited_program_paths.insert(path.clone());
                    },
                    ProgramItemId::None => {
                        pb.println(format!("{} - Encountered a solution without a path.", print_prefix_puzzle_id));
                    }
                }

                let model_filename: String = model_item.borrow().id.file_name();
                let program_filename: String = program_item.borrow().id.file_name();
                let record = Record {
                    model_filename: model_filename,
                    program_filename,
                };
                unique_records.insert(record);
                Record::save_solutions_csv(&unique_records, &path_solutions_csv);
            }

            pb2.finish_and_clear();
        }
        pb.finish_and_clear();
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} processing all puzzles with all solutions in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        // Print out names of unused programs that serves no purpose and can be removed
        let mut unused_programs = Vec::<String>::new();
        for program_item in &self.program_item_vec {
            let program_id: ProgramItemId = program_item.borrow().id.clone();
            let path: PathBuf = match program_id {
                ProgramItemId::Path { ref path } => path.clone(),
                ProgramItemId::None => {
                    continue;
                }
            };
            if !visited_program_paths.contains(&path) {
                unused_programs.push(program_id.file_name());
            }
        }
        if !unused_programs.is_empty() {
            error!("There are {} unused programs. These doesn't solve any of the models, and can be removed.", unused_programs.len());
            for filename in unused_programs {
                println!("UNUSED {:?}", filename);
            }
        }
    
        // Stats
        println!("row count in solutions csv file: {}", unique_records.len());
        println!("count_ok: {}", count_ok);
        println!("count_incorrect: {}", count_incorrect);
        println!("count_compute_error: {}", count_compute_error);
        println!("count_partial_match: {}", count_partial_match);
        if count_dangerous_false_positive > 0 {
            error!("count_dangerous_false_positive: {}", count_dangerous_false_positive);
        } else {
            println!("count_dangerous_false_positive: {}", count_dangerous_false_positive);
        }
        Ok(())
    }

    /// Eliminate duplicates in the program_item_vec
    fn dedup_program_item_vec(&mut self) {
        let count_before: usize = self.program_item_vec.len();
        let mut uniques = HashSet::<ProgramItemId>::new();
        self.program_item_vec.retain(|program_item| {
            let program_id: ProgramItemId = program_item.borrow().id.clone();
            uniques.insert(program_id)
        });
        let count_after: usize = self.program_item_vec.len();
        if count_before != count_after {
            println!("Removed duplicates from program_item_vec. count_before: {} count_after: {}", count_before, count_after);
        } else {
            println!("Great, no duplicates found");
        }
    }

    fn reload_analytics_dir(&mut self) -> anyhow::Result<()> {
        println!("loading genome mutate context");
        let start = Instant::now();

        Analytics::arc_run_force()?;

        let analytics_directory = AnalyticsDirectory::new(
            self.arc_config.path_analytics_arc_dir.clone()
        ).with_context(||"unable to create AnalyticsDirectory instance")?;    

        let context: GenomeMutateContext = create_genome_mutate_context(CreateGenomeMutateContextMode::ARC, analytics_directory)?;
        self.context = context;
        println!("loaded genome mutate context. elapsed: {}", HumanDuration(start.elapsed()));
        Ok(())
    }

    /// Print out lots of useful info.
    /// 
    /// I have tried submitting a docker image built with the wrong architecture. I don't want to repeat that.
    fn print_system_info() {
        println!("env::consts::ARCH: {}", std::env::consts::ARCH);
        println!("env::consts::OS: {}", std::env::consts::OS);

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let build_mode: &str;
        if cfg!(debug_assertions) {
            build_mode = "DEBUG (terrible performance!)";
        } else {
            build_mode = "RELEASE";
        }
        println!("LODA-RUST version: {}, build: {}", VERSION, build_mode);
    }

    fn run_arc_competition(&mut self) -> anyhow::Result<()> {
        let execute_start_time: Instant = Instant::now();
        let execute_time_limit: Duration = Duration::from_secs(ARC_COMPETITION_EXECUTE_DURATION_SECONDS);

        // When participating in the contest, then we want first to try out the existing solutions.
        // This may be a solution to one of the hidden puzzles.
        // However it's slow, so it's disabled while developing, where we only want to explore mutations.
        let try_existing_solutions = true;

        let number_of_programs_to_generate: usize = 3;

        println!("{} - Start of program", Self::human_readable_utc_timestamp());
        Self::print_system_info();

        println!("initial random seed: {}", ARC_COMPETITION_INITIAL_RANDOM_SEED);

        println!("initial number of solutions: {}", self.program_item_vec.len());
        println!("initial number of tasks: {}", self.model_item_vec.len());

        self.dedup_program_item_vec();
        self.reload_analytics_dir()?;

        let mut scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = self.model_item_vec.clone();

        let initial_tasks: Tasks = match self.read_solutions_json() {
            Ok(value) => value,
            Err(error) => {
                error!("Starting out with zero tasks. Unable to load existing solutions file: {:?}", error);
                vec!()
            }
        };
        println!("initial_tasks.len: {}", initial_tasks.len());

        let mut puzzle_names_to_ignore = HashSet::<String>::new();
        for task in &initial_tasks {
            puzzle_names_to_ignore.insert(task.task_name.clone());
        }

        let mut unique_records = HashSet::<Record>::new();

        let ignore_puzzles_with_a_solution: bool = self.arc_config.path_solutions_csv.is_file();
        if ignore_puzzles_with_a_solution {
            let record_vec = Record::load_record_vec(&self.arc_config.path_solutions_csv)?;
            debug!("solutions.csv: number of rows: {}", record_vec.len());
    
            for record in &record_vec {
                unique_records.insert(record.clone());
            }

            for record in &record_vec {
                let puzzle_filename_with_json_suffix: String = record.model_filename.clone();
                let puzzle_filename = puzzle_filename_with_json_suffix.replace(".json", "");
                puzzle_names_to_ignore.insert(puzzle_filename);
            }
        }
        debug!("puzzle_names_to_ignore: {:?}", puzzle_names_to_ignore);

        scheduled_model_item_vec = ModelItem::remove_model_items_where_filestem_contains(
            &scheduled_model_item_vec, 
            &puzzle_names_to_ignore
        );

        // println!("scheduled_model_item_vec.len(): {}", scheduled_model_item_vec.len());

        // Summary of what puzzles are to be solved
        {
            let mut number_of_solved_puzzles: usize = 0;
            let mut number_of_unsolved_puzzles: usize = 0;
            for model_item in &self.model_item_vec {
                let mut is_same = false;
                for model_item2 in &scheduled_model_item_vec {
                    if Rc::ptr_eq(&model_item, &model_item2) {
                        is_same = true;
                        break;
                    }
                }
                if is_same {
                    number_of_unsolved_puzzles += 1;
                } else {
                    number_of_solved_puzzles += 1;
                }
            }
            println!("puzzles solved: {}", number_of_solved_puzzles);
            println!("puzzles unsolved: {}", number_of_unsolved_puzzles);
        }

        let current_tasks: Tasks = initial_tasks;
        save_solutions_json(
            &self.arc_config.path_solution_dir,
            &self.arc_config.path_solution_teamid_json,
            &current_tasks
        );

        let bloom_items_count = 1000000;
        let false_positive_rate = 0.01;
        let mut bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        // Register the existing programs in the bloomfilter, so that these never gets suggested as a candidate solution
        for program_item in &self.program_item_vec {
            match program_item.borrow().bloom_key() {
                Ok(bloom_key) => {
                    bloom.set(&bloom_key);
                },
                Err(error) => {
                    error!("unable to create bloom_key for program: {:?}", error);
                }
            }
        }

        let plan = BatchPlan {
            execute_start_time,
            execute_time_limit,
            scheduled_model_item_vec,
            scheduled_program_item_vec: self.program_item_vec.clone(),
        };
        
        let mut state = BatchState {
            remove_model_items: vec!(),
            discovered_program_item_vec: vec!(),
            unique_records,
            current_tasks,
            terminate_due_to_timeout: false,
        };

        let mut runner = BatchRunner {
            config: self.arc_config.clone(),
            plan,
        };

        if try_existing_solutions {
            println!("{} - Run existing solutions without mutations", Self::human_readable_utc_timestamp());
            runner.run_one_batch(&mut state)?;
            self.transfer_discovered_programs(&mut state)?;
        }

        // loop until all puzzles have been solved
        let mut mutation_index: u64 = 0;
        loop {
            if runner.plan.scheduled_model_item_vec.is_empty() {
                println!("{} - It seems all the puzzles have been solved.", Self::human_readable_utc_timestamp());
                break;
            }
            if state.terminate_due_to_timeout {
                println!("{} - Terminating due to timeout.", Self::human_readable_utc_timestamp());
                break;
            }
            println!("{} - Mutation: {}", Self::human_readable_utc_timestamp(), mutation_index);

            // Create new mutated programs in every iteration
            runner.plan.scheduled_program_item_vec = self.create_mutations_of_all_programs(
                mutation_index, 
                number_of_programs_to_generate, 
                &mut bloom
            );

            // Evaluate all puzzles with all candidate programs
            runner.run_one_batch(&mut state)?;
            self.transfer_discovered_programs(&mut state)?;
            
            mutation_index += 1;
        }
        println!("{} - Executable elapsed: {}.", Self::human_readable_utc_timestamp(), HumanDuration(execute_start_time.elapsed()));

        println!("Done!");
        Ok(())
    }

    fn human_readable_utc_timestamp() -> String {
        let datetime: DateTime<Utc> = Utc::now();
        datetime.to_rfc3339_opts(SecondsFormat::Secs, true).to_string()
    }

    /// Move discovered programs to the original programs vector
    fn transfer_discovered_programs(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        if state.discovered_program_item_vec.is_empty() {
            return Ok(());
        }
        println!("transferred {:?} solutions", state.discovered_program_item_vec.len());

        self.program_item_vec.append(&mut state.discovered_program_item_vec);
        if !state.discovered_program_item_vec.is_empty() {
            error!("Expected state.discovered_program_item_vec to be empty after moving the elements");
        }

        // When a program solves multiple puzzles, 
        // then the program gets appended multiple times. 
        // This eliminates the duplicates.
        self.dedup_program_item_vec();

        // Regenerate analytics when new programs have been mined
        self.reload_analytics_dir()?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct RunArcCompetitionConfig {
    path_analytics_arc_dir: PathBuf,
    path_solutions_csv: PathBuf,
    path_programs: PathBuf,
    path_solution_dir: PathBuf,
    path_solution_teamid_json: PathBuf,
}

impl RunArcCompetitionConfig {
    fn new(config: &Config) -> Self {
        let path_solutions_csv = config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));

        let path_solution_dir: PathBuf = config.arc_repository_data().join(Path::new("solution"));
        let path_solution_teamid_json: PathBuf = path_solution_dir.join(Path::new(SOLUTIONS_FILENAME));

        RunArcCompetitionConfig {
            path_analytics_arc_dir: config.analytics_arc_dir(),
            path_solutions_csv,
            path_programs: config.loda_arc_challenge_repository_programs(),
            path_solution_dir,
            path_solution_teamid_json,
        }
    }
}

#[derive(Debug)]
struct BatchPlan {
    execute_start_time: Instant,
    execute_time_limit: Duration,
    scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    scheduled_program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
}

impl BatchPlan {
    /// Outer loop traverses the unsolved puzzles.
    /// 
    /// Inner loop traverses the candidate solutions.
    fn run_one_batch(
        &self, 
        config: &RunArcCompetitionConfig,
        state: &mut BatchState,
    ) -> anyhow::Result<()> {
        let verify_test_output = false;
        let verbose = false;
        let max_duration_seconds: u64 = 60;

        let mut start_time = Instant::now();
        let mut slowest_program_elapsed = Duration::ZERO;
        let mut slowest_program_name = String::new();

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.scheduled_model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Unsolved puzzle   ");
        for (model_index, model_item) in self.scheduled_model_item_vec.iter().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }
    
            let model: Model = model_item.borrow().model.clone();
            if verbose {
                let number_of_train_pairs: usize = model.train().len();
                let number_of_test_pairs: usize = model.test().len();
                pb.println(format!("puzzle: {} train: {} test: {}", model.id().identifier(), number_of_train_pairs, number_of_test_pairs));
            }

            let instance = RunWithProgram::new(model, verify_test_output).expect("RunWithProgram");

            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new(self.scheduled_program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Candidate solution");
            for program_index in 0..self.scheduled_program_item_vec.len() {
                if program_index > 0 {
                    pb.tick();
                    pb2.inc(1);
                }

                if self.execute_start_time.elapsed() >= self.execute_time_limit {
                    state.terminate_due_to_timeout = true;
                    let message = format!(
                        "{} - Exceeded time limit for executable", 
                        TraverseProgramsAndModels::human_readable_utc_timestamp(), 
                    );
                    pb.println(message);
                    pb2.finish_and_clear();
                    pb.finish_and_clear();
                    return Ok(());
                }

                let elapsed: Duration = start_time.elapsed();
                if elapsed.as_secs() >= max_duration_seconds {
                    let total_number_of_solutions: usize = state.current_tasks.len();
                    let message = format!(
                        "{} - Status.  Total number of solutions: {}  Slowest program: {:?} {}", 
                        TraverseProgramsAndModels::human_readable_utc_timestamp(), 
                        total_number_of_solutions,
                        slowest_program_name,
                        HumanDuration(slowest_program_elapsed)
                    );
                    pb.println(message);
                    start_time = Instant::now();
                    slowest_program_elapsed = Duration::ZERO;
                    slowest_program_name = String::new();
                }
    
                let program_item: &Rc<RefCell<ProgramItem>> = &self.scheduled_program_item_vec[program_index];
                
                let run_with_program_result: RunWithProgramResult;
                {
                    let before_run_program = Instant::now();

                    let program_runner: &ProgramRunner = &program_item.borrow().program_runner;
                    let result = instance.run_program_runner(program_runner);

                    let program_run_elapsed: Duration = before_run_program.elapsed();
                    if program_run_elapsed > slowest_program_elapsed {
                        slowest_program_elapsed = program_run_elapsed;
                        slowest_program_name = program_item.borrow().id.file_name();
                    }
    
                    run_with_program_result = match result {
                        Ok(value) => value,
                        Err(error) => {
                            if verbose {
                                error!("run_program_runner model: {:?} program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                            }
                            continue;
                        }
                    };
                    if verbose {
                        let s = format!("model: {:?} program: {:?} result: {:?}", model_item.borrow().id, program_item.borrow().id, run_with_program_result);
                        pb.println(s);
                    }
                }

                if run_with_program_result.count_train_correct() == 0 {
                    // None of the training pairs match.
                    // This is not a solution. Proceed to the next candidate solution.
                    // pb.println(format!("Puzzle {:?}, count_train_correct is zero. Ignoring.", model_item.borrow().id));
                    continue;
                }

                let count_test_empty: usize = run_with_program_result.count_test_empty();
                if count_test_empty > 0 {
                    // All the "test" outputs must be non-empty, to ensure that it's not a raw copy/paste of the input.
                    // This is not a solution. Proceed to the next candidate solution.
                    pb.println(format!("{} - Puzzle {:?}, ignoring dangerous false-positive, that copies the expected output to the actual output.", TraverseProgramsAndModels::human_readable_utc_timestamp(), model_item.borrow().id));
                    continue;
                }

                let count_train_correct: usize = run_with_program_result.count_train_correct();
                let count_train_incorrect: usize = run_with_program_result.count_train_incorrect();
                if count_train_incorrect > 0 {
                    // Partial solution. One or more incorrect training pairs. We want all the training pairs to be satisfied.
                    // This is not a full solution. Proceed to the next candidate solution.
                    if verbose {
                        pb.println(format!("{} - Puzzle {:?}, partial solution. correct: {} incorrect: {}", TraverseProgramsAndModels::human_readable_utc_timestamp(), model_item.borrow().id, count_train_correct, count_train_incorrect));
                    }
                    continue;
                }

                // All the train pairs are correct.
                // The test pairs are unverified, and have a size of 1x1 or bigger.
                // This may be a solution.
                pb.println(format!("{} - Puzzle {:?}, possible solution. correct: {} incorrect: {}", TraverseProgramsAndModels::human_readable_utc_timestamp(), model_item.borrow().id, count_train_correct, count_train_incorrect));

                let save_result = state.save_solution(
                    config, 
                    Rc::clone(model_item), 
                    Rc::clone(program_item), 
                    run_with_program_result, 
                    &pb
                );

                match save_result {
                    Ok(()) => {
                        // This is a solution to this puzzle. No need to loop through the remaining programs.
                        break;
                    },
                    Err(error) => {
                        error!("Unable to save solution. model: {:?} error: {:?}", model_item.borrow().id, error);
                        // Something went wrong saving this solution. Consider this puzzle as still being unsolved.
                        // Loop through the remaining programs to check for another solution.
                        continue;
                    }
                }
            }
            pb2.finish_and_clear();
        }
        pb.finish_and_clear();

        Ok(())
    }

    fn reschedule(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        if state.remove_model_items.is_empty() {
            return Ok(());
        }
        
        // Remove solved puzzles from the scheduled_model_item_vec
        self.scheduled_model_item_vec = ModelItem::remove_model_items(
            &self.scheduled_model_item_vec, 
            &state.remove_model_items
        );
        state.remove_model_items.clear();

        Ok(())
    }
}

struct BatchState {
    remove_model_items: Vec<Rc<RefCell<ModelItem>>>,
    discovered_program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    unique_records: HashSet::<Record>,
    current_tasks: Tasks,
    terminate_due_to_timeout: bool,
}

impl BatchState {
    fn save_solution(
        &mut self, 
        config: &RunArcCompetitionConfig, 
        model_item: Rc<RefCell<ModelItem>>, 
        program_item: Rc<RefCell<ProgramItem>>,
        run_with_program_result: RunWithProgramResult,
        progress_bar: &ProgressBar,
    ) -> anyhow::Result<()> {
        let model_id: ModelItemId = model_item.borrow().id.clone(); 

        // Save the program to disk.
        //
        // Don't save the program when it already exist in the file system.
        // On launch of the miner, then first try out all the existing programs with the puzzles. This may yield a match.
        // In which case we don't want to save the already existing program to disk.
        let is_new_program: bool = program_item.borrow().id == ProgramItemId::None;
        if is_new_program {
            let name: String = model_id.file_stem();
            let program_filename: String = match ProgramItem::unique_name_for_saving(&config.path_programs, &name) {
                Ok(filename) => filename,
                Err(error) => {
                    return Err(anyhow::anyhow!("cannot save file, because of error: {:?}", error));
                }
            };
            let program_path: PathBuf = config.path_programs.join(Path::new(&program_filename));
            let mut file = File::create(&program_path)?;
            let content: String = program_item.borrow().program_string.clone();
            file.write_all(content.as_bytes())?;
            program_item.borrow_mut().id = ProgramItemId::Path { path: program_path };
        }

        let program_id: ProgramItemId = program_item.borrow().id.clone(); 
        if program_id == ProgramItemId::None {
            return Err(anyhow::anyhow!("Expected ProgramItem.id to be a Path, but got None. {:?}", program_item));
        }

        // Print that the puzzle has been solved using a new/existing program
        let solution_type: &str;
        if is_new_program {
            solution_type = "a new";
        } else {
            solution_type = "an existing";
        }
        let message = format!("Puzzle {:?} solved with {} program: {:?}", model_id.file_stem(), solution_type, program_id.file_name());
        progress_bar.println(message);

        // Update CSV file
        let record = Record {
            model_filename: model_id.file_name(),
            program_filename: program_id.file_name(),
        };
        self.unique_records.insert(record);
        Record::save_solutions_csv(&self.unique_records, &config.path_solutions_csv);
        
        // Update JSON file
        let predictions: Vec<Prediction> = run_with_program_result.predictions().clone();
        let test_item = TestItem { 
            output_id: 0,
            number_of_predictions: predictions.len() as u8,
            predictions: predictions,
        };
        let task_name: String = model_id.file_stem();
        let task_item = TaskItem {
            task_name: task_name,
            test_vec: vec![test_item],
        };
        self.current_tasks.push(task_item);
        save_solutions_json(
            &config.path_solution_dir,
            &config.path_solution_teamid_json,
            &self.current_tasks
        );

        // Append the puzzle to the solved puzzles
        self.remove_model_items.push(Rc::clone(&model_item));

        // Append new programs to discovered programs
        // Ignore existing programs
        if is_new_program {
            self.discovered_program_item_vec.push(program_item);
        }

        Ok(())
    }
}

struct BatchRunner {
    config: RunArcCompetitionConfig,
    plan: BatchPlan,
}

impl BatchRunner {
    fn run_one_batch(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        self.plan.run_one_batch(&self.config, state)?;
        self.plan.reschedule(state)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum ModelItemId {
    None,
    Path { path: PathBuf },
}

impl ModelItemId {
    fn file_name(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }

    fn file_stem(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_stem() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_stem".to_string();
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ModelItem {
    id: ModelItemId,
    model: Model,
    // It's costly to convert the json representation to image over and over. Do it once.
    // TODO: convert model to images, LabelSet several places in the model
}

impl ModelItem {
    fn remove_model_items_where_filestem_contains(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        names_for_removal: &HashSet<String>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        let mut result_items: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if !names_for_removal.contains(&file_stem) {
                result_items.push(Rc::clone(model_item));
            }
        }
        result_items
    }

    fn remove_model_items(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        model_item_vec_for_removal: &Vec<Rc<RefCell<ModelItem>>>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        if model_item_vec_for_removal.is_empty() {
            return model_item_vec.clone();
        }
        let count_before: usize = model_item_vec.len();
        let mut result_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let mut keep = true;
            for remove_model_item in model_item_vec_for_removal {
                if Rc::ptr_eq(&remove_model_item, &model_item) {
                    keep = false;
                    break;
                }
            }
            if keep {
                result_model_item_vec.push(Rc::clone(model_item));
            }
        }
        let count_after: usize = result_model_item_vec.len();
        if count_after > count_before {
            error!("Expected removal to shrink vector, but it grows. {} != {} + {}", count_before, count_after, model_item_vec_for_removal.len());
        }
        result_model_item_vec
    }
}

#[derive(Clone, Debug)]
enum ProgramType {
    Simple,
    Advance,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ProgramItemId {
    None,
    Path { path: PathBuf },
}

impl ProgramItemId {
    fn file_name(&self) -> String {
        match self {
            ProgramItemId::None => {
                return "None".to_string();
            },
            ProgramItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }
}

type RcProgramItem = Rc<RefCell<ProgramItem>>;
type RcProgramItemVec = Vec<RcProgramItem>;

struct ProgramItem {
    id: ProgramItemId,
    program_string: String,
    program_type: ProgramType,
    parsed_program: ParsedProgram,
    program_runner: ProgramRunner,
}

impl ProgramItem {
    /// Returns a compacted version of the program, that is only intended for use in the bloomfilter.
    /// Inserts header/footer if it's a simple program. Keeps the program if it's an adavanced program.
    /// There are no comments or unneccessary spacing.
    fn bloom_key(&self) -> anyhow::Result<String> {
        let compact_program_string: String = self.parsed_program.to_string();
        Ok(compact_program_string)
    }

    fn unique_name_for_saving(dir_path: &Path, name: &str) -> anyhow::Result<String> {
        assert!(dir_path.is_dir());
        assert!(dir_path.is_absolute());
        let max_number_of_variants: usize = 30;
        for variant_index in 1..max_number_of_variants {
            let filename = format!("{}-{}.asm", name, variant_index);
            let file_path: PathBuf = dir_path.join(&filename);
            if !file_path.is_file() {
                return Ok(filename);
            }
        }
        Err(anyhow::anyhow!("ProgramItem: Cannot construct unique filename for {:?} inside dir: {:?}", name, dir_path))
    }
}

impl fmt::Debug for ProgramItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ProgramItem {:?} program {:?}", self.id, self.program_string)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
struct Record {
    #[serde(rename = "model filename")]
    model_filename: String,
    #[serde(rename = "program filename")]
    program_filename: String,
}

impl Record {
    fn load_record_vec(csv_path: &Path) -> anyhow::Result<Vec<Record>> {
        let record_vec: Vec<Record> = parse_csv_file(csv_path)
            .map_err(|e| anyhow::anyhow!("unable to parse csv file. error: {:?}", e))?;
        Ok(record_vec)
    }

    fn save_solutions_csv(unique_records: &HashSet<Record>, path_csv: &Path) {
        let mut record_vec: Vec<Record> = unique_records.iter().map(|record| record.clone()).collect();
        record_vec.sort_unstable_by_key(|item| (item.model_filename.clone(), item.program_filename.clone()));
        match create_csv_file(&record_vec, &path_csv) {
            Ok(()) => {},
            Err(error) => {
                error!("Unable to save csv file: {:?}", error);
            }
        }
    }
}

fn save_solutions_json(path_solution_dir: &Path, path_solution_teamid_json: &Path, tasks: &Tasks) {
    if !path_solution_dir.exists() {
            match fs::create_dir(path_solution_dir) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to create solution directory: {:?}, error: {:?}", path_solution_dir, err);
            }
        }
    }
    let json: String = match serde_json::to_string(&tasks) {
        Ok(value) => value,
        Err(error) => {
            error!("unable to serialize tasks to json: {:?}", error);
            return;
        }
    };
    match fs::write(&path_solution_teamid_json, json) {
        Ok(()) => {},
        Err(error) => {
            error!("unable to save solutions file. path: {:?} error: {:?}", path_solution_teamid_json, error);
            return;
        }
    }
    debug!("updated solutions file: tasks.len(): {}", tasks.len());
}
