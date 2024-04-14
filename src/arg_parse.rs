use crate::types::*;

use serde::{Deserialize, Serialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;


impl<'de> Deserialize<'de> for BaseOsc {
    fn deserialize<D>(deserializer: D) -> Result<BaseOsc, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BaseOscVisitor;

        impl<'de> Visitor<'de> for BaseOscVisitor {
            type Value = BaseOsc;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`sine`, `square`, `sawtooth`, or `triangle`")
            }

            fn visit_str<E>(self, value: &str) -> Result<BaseOsc, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "sine" => Ok(BaseOsc::Sine),
                    "square" => Ok(BaseOsc::Square),
                    "sawtooth" => Ok(BaseOsc::Sawtooth),
                    "triangle" => Ok(BaseOsc::Triangle),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `sine`, `square`, `sawtooth`, `triangle`", value))),
                }
            }
        }
        deserializer.deserialize_str(BaseOscVisitor)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SCSound {
    osc_type: Option<BaseOsc>,
    filepath: Option<String>,
    min_freq: f64,
    max_freq: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TinPanContrib {
    sound: SCSound,
    motes: Vec<Mote>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conf {
    cps: f32,
    root: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Template {
    conf: Conf,
    parts: Vec<TinPanContrib>,
}

#[derive(Debug, Serialize)]
struct Ratio (i32, i32);

mod test_unit {
    use super::*;
    use std::fs;
    use std::io;

    #[test]
    fn test_parse_template() {
        let motes:Vec<Mote> = vec![ (4.0, 220., 1.0), (2.0, 880., 0.5), (2.0, 440., 1.)];

        let sound = SCSound {
            osc_type: Some(BaseOsc::Sine),
            // filepath previously represented sample path
            filepath: Some(String::from("path/to/soundfile.wav")),
            min_freq: 440.0,
            max_freq: 880.0,
        };

        let part = TinPanContrib {
            sound,
            motes,
        };

        let conf = Conf {
            cps: 60.0,
            root: 440.0,
        };

        let expected = Template {
            conf,
            parts: vec![part],
        };

        match fs::read_to_string("test-score.json") {
            Ok(str) => {
                let actual:Template = serde_json::from_str(&str).expect("Bad parser");
                println!("parsed score: {:?}", actual);
            },
            Err(msg) => {
                println!("Missing test score 'test-score.json'");
                assert!(false);
            }
        };
    }

    #[test]
    fn test_parse_tin_pan_score() {
        let score_path = "test-tin-pan-score.json";

        match fs::read_to_string(&score_path) {
            Ok(str) => {
                let actual:Template = serde_json::from_str(&str).expect("Bad parser");
                println!("parsed score: {:?}", actual);
            },
            Err(msg) => {
                println!("Missing test score 'test-score.json'");
                assert!(false);
            }
        };
    }
}

