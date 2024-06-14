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
                    _ => Err(E::custom(format!("unknown Visibility variant `{}`, expected one of `foreground`, `visible`, `background`, `hidden`", value))),
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
                    _ => Err(E::custom(format!("unknown Mode variant `{}`, expected one of `melodic`, `enharmonic`, `vagrant`, `noise`", value))),
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
                    _ => Err(E::custom(format!("unknown Role variant `{}`, expected one of `kick`, `perc`, `hats`, `bass`, `chords`, or `lead`", value))),
                }
            }
        }
        deserializer.deserialize_str(RoleVisitor)
    }
}

impl<'de> Deserialize<'de> for AmpLifespan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AmpLifespanVisitor;

        impl<'de> Visitor<'de> for AmpLifespanVisitor {
            type Value = AmpLifespan;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`fall`, `snap`, `spring`, `pluck`, `bloom`, `burst`, `pad`, or `drone`")
            }

            fn visit_str<E>(self, value: &str) -> Result<AmpLifespan, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "fall" => Ok(AmpLifespan::Fall),
                    "snap" => Ok(AmpLifespan::Snap),
                    "spring" => Ok(AmpLifespan::Spring),
                    "pluck" => Ok(AmpLifespan::Pluck),
                    "bloom" => Ok(AmpLifespan::Bloom),
                    "burst" => Ok(AmpLifespan::Burst),
                    "pad" => Ok(AmpLifespan::Pad),
                    "drone" => Ok(AmpLifespan::Drone),
                    _ => Err(E::custom(format!("Unknown AmpLifespan variant `{}`, expected one of `fall`, `snap`, `spring`, `pluck`, `bloom`, `burst`, `pad`, or `drone`", value))),
                }
            }
        }
        deserializer.deserialize_str(AmpLifespanVisitor)
    }
}

impl<'de> Deserialize<'de> for AmpContour {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AmpContourVisitor;

        impl<'de> Visitor<'de> for AmpContourVisitor {
            type Value = AmpContour;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`fade`, `throb`, `surge`, `chops`, or `flutter`")
            }

            fn visit_str<E>(self, value: &str) -> Result<AmpContour, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "fade" => Ok(AmpContour::Fade),
                    "throb" => Ok(AmpContour::Throb),
                    "surge" => Ok(AmpContour::Surge),
                    "chops" => Ok(AmpContour::Chops),
                    "flutter" => Ok(AmpContour::Flutter),
                    _ => Err(E::custom(format!("Unknown AmpContour variant `{}`, expected one of `fade`, `throb`, `surge`, `chops`, or `flutter`", value))),
                }
            }
        }
        deserializer.deserialize_str(AmpContourVisitor)
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


mod test_unit {
    use super::*;
    use crate::types::render::Score;
    use serde_json::Error as SerdeError;

    /// Verify raudio accepts input from external applications
    #[test]
    fn test_parse_tin_pan_score() {
        let input_score_path = "src/demo/test-druidic-score.json";

        let file_content = fs::read_to_string(&input_score_path)
            .expect(&format!("Failed to read file {}", input_score_path));
        
        let score: Result<DruidicScore, SerdeError> = serde_json::from_str(&file_content);

        assert!(score.is_ok(), "Failed to parse a druidic score from {}", input_score_path);
    }
}

