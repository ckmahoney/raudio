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



impl<'de> Deserialize<'de> for timbre::BaseOsc {
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
// impl<'de> Deserialize<'de> for synthesis::Q {
//     fn deserialize<D>(deserializer: D) -> Result<Q, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct QVisitor;

//         impl<'de> Visitor<'de> for QVisitor {
//             type Value = Q;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("`O` or `Q`")
//             }

//             fn visit_str<E>(self, value: &str) -> Result<Q, E>
//             where
//                 E: de::Error,
//             {
//                 match value.to_lowercase().as_str() {
//                     "o" => Ok(Q::O),
//                     "u" => Ok(Q::U),
//                     _ => Err(E::custom(format!("unknown variant `{}`, expected one of `O`, `Q`", value))),
//                 }
//             }
//         }
//         deserializer.deserialize_str(QVisitor)
//     }
// }

// impl<'de> Deserialize<'de> for timbre::Fill {
//     fn deserialize<D>(deserializer: D) -> Result<Fill, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct FillVisitor;

//         impl<'de> Visitor<'de> for FillVisitor {
//             type Value = Fill;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("`frame`, `support`, or `focus``")
//             }

//             fn visit_str<E>(self, value: &str) -> Result<Fill, E>
//             where
//                 E: de::Error,
//             {
//                 match value.to_lowercase().as_str() {
//                     "frame" => Ok(Fill::Frame),
//                     "support" => Ok(Fill::Support),
//                     "focus" => Ok(Fill::Focus),
//                     _ => Err(E::custom(format!("unknown variant `{}`, expected one of `frame`, `support`, `focus` ", value))),
//                 }
//             }
//         }
//         deserializer.deserialize_str(FillVisitor)
//     }
// }

impl<'de> Deserialize<'de> for timbre::Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Visibility, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VisibilityVisitor;

        impl<'de> Visitor<'de> for VisibilityVisitor {
            type Value = Visibility;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`foreground`, `visible`, `background`, or `hidden`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Visibility, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "foreground" => Ok(Visibility::Foreground),
                    "visible" => Ok(Visibility::Visible),
                    "background" => Ok(Visibility::Background),
                    "hidden" => Ok(Visibility::Hidden),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `foreground`, `visible`, `background`, `hidden`", value))),
                }
            }
        }
        deserializer.deserialize_str(VisibilityVisitor)
    }
}

impl<'de> Deserialize<'de> for timbre::Mode {
    fn deserialize<D>(deserializer: D) -> Result<Mode, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ModeVisitor;

        impl<'de> Visitor<'de> for ModeVisitor {
            type Value = Mode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`melodic`, `enharmonic`, `vagrant`, or `noise`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Mode, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "melodic" => Ok(Mode::Melodic),
                    "enharmonic" => Ok(Mode::Enharmonic),
                    "vagrant" => Ok(Mode::Vagrant),
                    "noise" => Ok(Mode::Noise),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `melodic`, `enharmonic`, `vagrant`, `noise`", value))),
                }
            }
        }
        deserializer.deserialize_str(ModeVisitor)
    }
}


impl<'de> Deserialize<'de> for timbre::Role {
    fn deserialize<D>(deserializer: D) -> Result<Role, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RoleVisitor;

        impl<'de> Visitor<'de> for RoleVisitor {
            type Value = Role;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`kick`, `perc`, `hats`, `bass`, `chords`, or `lead`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Role, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "kick" => Ok(Role::Kick),
                    "perc" => Ok(Role::Perc),
                    "hats" => Ok(Role::Hats),
                    "bass" => Ok(Role::Bass),
                    "chords" => Ok(Role::Chords),
                    "lead" => Ok(Role::Lead),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `kick`, `perc`, `hats`, `bass`, `chords`, or `lead`", value))),
                }
            }
        }
        deserializer.deserialize_str(RoleVisitor)
    }
}


impl<'de> Deserialize<'de> for timbre::Energy {
    fn deserialize<D>(deserializer: D) -> Result<Energy, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnergyVisitor;

        impl<'de> Visitor<'de> for EnergyVisitor {
            type Value = Energy;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`low`, `medium`, or `high`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Energy, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "low" => Ok(Energy::Low),
                    "medium" => Ok(Energy::Medium),
                    "high" => Ok(Energy::High),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `low`, `medium`,or `high`", value))),
                }
            }
        }
        deserializer.deserialize_str(EnergyVisitor)
    }
}

impl<'de> Deserialize<'de> for timbre::Presence {
    fn deserialize<D>(deserializer: D) -> Result<Presence, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PresenceVisitor;

        impl<'de> Visitor<'de> for PresenceVisitor {
            type Value = Presence;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`staccatto`, `legato`, or `tenuto`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Presence, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "staccatto" => Ok(Presence::Staccatto),
                    "legato" => Ok(Presence::Legato),
                    "tenuto" => Ok(Presence::Tenuto),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `staccatto`, `legato`, or `tenuto`", value))),
                }
            }
        }
        deserializer.deserialize_str(PresenceVisitor)
    }
}

impl<'de> Deserialize<'de> for timbre::Cube {
    fn deserialize<D>(deserializer: D) -> Result<Cube, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CubeVisitor;

        impl<'de> Visitor<'de> for CubeVisitor {
            type Value = Cube;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`room`, `hall`, or `vast`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Cube, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "room" => Ok(Cube::Room),
                    "hall" => Ok(Cube::Hall),
                    "vast" => Ok(Cube::Vast),
                    _ => Err(E::custom(format!("unknown variant `{}`, expected one of `room`, `hall`, or `vast`", value))),
                }
            }
        }
        deserializer.deserialize_str(CubeVisitor)
    }
}
pub fn load_score_from_file(filepath:&str) -> Result<Score, fmt::Error> {
    match fs::read_to_string(&filepath) {
        Ok(str) => {
            let score:Score = serde_json::from_str(&str).expect("Bad parser");
            Ok(score)
        },
        _ => Err(fmt::Error)
    }
}
mod test_unit {
    use super::*;
    use crate::types::render::Score;

    #[test]
    fn test_parse_tin_pan_score() {
        let score_path = "test-tin-pan-score.json";

        match fs::read_to_string(&score_path) {
            Ok(str) => {
                let score:Score = serde_json::from_str(&str).expect("Bad parser");
                assert!(true)
            },
            Err(msg) => {
                println!("Missing test score 'test-score.json'");
                assert!(false);
            }
        };
    }
}

