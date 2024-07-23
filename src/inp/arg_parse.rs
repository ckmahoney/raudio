use serde::{Deserialize, Serialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
 
use std::fs;
use std::io;    

use crate::types::synthesis;
use crate::types::synthesis::*;
use crate::types::timbre;
use crate::types::timbre::*;
use crate::types::render::*;

pub fn load_score_from_file(filepath:&str) -> Result<DruidicScore, fmt::Error> {
    match fs::read_to_string(&filepath) {
        Ok(str) => {
            let score:DruidicScore = serde_json::from_str(&str).expect(&format!("Failed to parse score from file at path {}", filepath));
            Ok(score)
        },
        _ => Err(fmt::Error)
    }
}

mod test_unit {
    use super::*;
    use crate::types::render::Score;
    use serde_json::Error as SerdeError;
    use std::fs;

    /// Verify raudio accepts input from external applications
    #[test]
    fn test_parse_tin_pan_score() {
        let input_score_path = "src/demo/test-druidic-score.json";

        let file_content = fs::read_to_string(&input_score_path)
            .expect(&format!("Failed to read file {}", input_score_path));
        
        let score: Result<DruidicScore, SerdeError> = serde_json::from_str(&file_content);
        println!("Score: {:?}",score);
        assert!(
            score.is_ok(),
            "Failed to parse a druidic score from {}: {}",
            input_score_path,
            score.unwrap_err()
        );
    }
}