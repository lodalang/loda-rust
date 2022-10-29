use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PostMineError {
    UnableToExtractFilenameFromPath,
    CannotMutateCandidateProgramWithAlreadyResolvedState,
}

impl fmt::Display for PostMineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnableToExtractFilenameFromPath =>
                write!(f, "Unable to extract filename from path"),
            Self::CannotMutateCandidateProgramWithAlreadyResolvedState =>
                write!(f, "Cannot mutate candidate program with already resolved state"),
        }
    }
}

impl Error for PostMineError {}
