use chrono::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub struct LastAnalyticsTimestamp {
    datetime: DateTime<Utc>
}

impl LastAnalyticsTimestamp {
    pub fn load(timestamp_file_path: &Path) -> anyhow::Result<Self> {
        if !timestamp_file_path.is_file() {
            return Err(anyhow::anyhow!("No timestamp file found at path {:?}", timestamp_file_path));
        }
        let contents: String = match fs::read_to_string(timestamp_file_path) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Cannot load timestamp file. path: {:?} error: {:?}", timestamp_file_path, error));
            }
        };
        let datetime: DateTime<Utc> = Self::parse_utc_string(&contents)?;
        let instance = Self {
            datetime: datetime,
        };
        Ok(instance)
    }

    fn parse_utc_string(utc_string: &String) -> anyhow::Result<DateTime<Utc>> {
        let datetime: DateTime<FixedOffset> = match DateTime::parse_from_rfc3339(utc_string) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Cannot parse timestamp file as UTC. path: {:?} error: {:?}", utc_string, error));
            }
        };
        let datetime: DateTime<Utc> = datetime.with_timezone(&Utc);
        Ok(datetime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_parse_utc_string_ok() {
        let date_str = "1999-03-24T21:59:33Z".to_string(); // release date of "the matrix"
        let dt: DateTime<Utc> = LastAnalyticsTimestamp::parse_utc_string(&date_str).unwrap();
        assert_eq!(dt.year(), 1999);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 24);
        assert_eq!(dt.hour(), 21);
        assert_eq!(dt.minute(), 59);
    }

    #[test]
    fn test_10001_parse_utc_string_error() {
        static INPUT: &'static [&'static str] = &[
            "",
            "junk",
            "1999-03-24T21:59:33",  // Missing "Z" suffix
            "1999-03-24 21:59:33Z", // Missing "T" infix
            "1999-03-24T21:59:33Zjunk",
            "junk1999-03-24T21:59:33Z",
        ];
        for input in INPUT {
            LastAnalyticsTimestamp::parse_utc_string(&input.to_string()).expect_err("is supposed to fail");
        }
    }

    #[test]
    fn test_10002_format() {
        let dt: DateTime<Utc> = Utc.ymd(1999, 3, 24).and_hms_micro(21, 59, 33, 453_829);
        let s = dt.to_rfc3339_opts(SecondsFormat::Secs, true).to_string();
        assert_eq!(s, "1999-03-24T21:59:33Z"); // release date of "the matrix"
    }
}
