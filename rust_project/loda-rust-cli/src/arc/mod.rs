//! ARC challenge experiments
mod arc_json_model;
mod arc_puzzles;
mod bitmap_try_create;
mod convolution2x2;
mod convolution3x3;
mod convolution_with_program;
mod find;
mod histogram;
mod image;
mod image_to_number;
mod index_for_pixel;
mod ngram;
mod number_to_bitmap;
mod padding;
mod program_with_callback;
mod offset;
mod read_testdata;
mod resize;
mod register_arc_functions;
mod remove_duplicates;
mod replace_color;
mod rotate;
mod symmetry;
mod test_convert;
mod trim;

pub use arc_json_model::{Grid, GridToBitmap, Model, TaskPair};
pub use bitmap_try_create::BitmapTryCreate;
pub use convolution2x2::convolution2x2;
pub use convolution3x3::convolution3x3;
pub use find::BitmapFind;
pub use histogram::BitmapHistogram;
pub use image::Image;
pub use image_to_number::ImageToNumber;
pub use ngram::{BitmapNgram, RecordBigram, RecordTrigram};
pub use number_to_bitmap::NumberToBitmap;
pub use offset::BitmapOffset;
pub use padding::Padding;
pub use read_testdata::read_testdata;
pub use register_arc_functions::register_arc_functions;
pub use replace_color::BitmapReplaceColor;
pub use resize::BitmapResize;
pub use remove_duplicates::BitmapRemoveDuplicates;
pub use rotate::BitmapRotate;
pub use symmetry::BitmapSymmetry;
pub use trim::BitmapTrim;
