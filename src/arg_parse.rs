use crate::synth::{Mote};
use crate::song::{BaseOsc};

use serde::{Deserialize, Serialize};
use serde_json::Result;


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
    cps: i32,
    root: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Template {
    conf: Conf,
    parts: Vec<TinPanContrib>,
}

// pub fn typed_example() -> Result<()> {
//     let data = r#"
//         {
//             "name": "John Doe",
//             "age": 43,
//             "phones": [
//                 "+44 1234567",
//                 "+44 2345678"
//             ]
//         }"#;

//     let p: Person = serde_json::from_str(data)?;

//     println!("Please call {} at the number {}", p.name, p.phones[0]);

//     Ok(())
// }
#[derive(Debug, Serialize)]
struct Ratio (i32, i32);

mod test_unit {
    use super::*;
    use std::fs;
    use std::io;

    #[test]
    fn test_parse_template() -> Result<()> {
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
            cps: 60,
            root: 440,
        };

        let expected = Template {
            conf,
            parts: vec![part],
        };

        let str = fs::read_to_string("test-score.json").expect("Missing test score 'test-score.json'");
        
        let actual:Template = serde_json::from_str(&str)?;
        println!("parsed score: {:?}", actual);
        Ok(())
    }
}

