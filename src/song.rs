use once_cell::sync::Lazy;
pub use std::collections::HashMap;
use crate::midi::*;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spec {
    pub role: &'static str,
    pub register: i32,
    pub fill: &'static str,
    pub spec_type: &'static str,
}

#[derive(Debug, Clone)]
pub struct PlayerTrack {
    pub conf: Conf,
    pub composition: Composition,
}

#[derive(Debug, Clone)]
pub struct Conf {
    pub origin: &'static str,
    pub duration: i32,
    pub cps: f32,
    pub title: &'static str,
    pub transposition: i32,
}

#[derive(Debug, Clone)]
pub struct Composition {
    pub composition_id: i32,
    pub duration: i32,
    pub quality: &'static str,
    pub dimensions: Dimensions,
    pub progression: Vec<(i32, &'static str)>,
    pub parts: HashMap<Spec, Vec<Vec<Midi>>>,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub size: i32,
    pub cpc: i32,
    pub base: i32,
}

pub mod x_files {
    use super::*;

    static PIANO_LINE: Lazy<Vec<Midi>> = Lazy::new(|| {
        vec![
            (0.33333333, 57, 127),
            (0.33333333, 60, 127),
            (0.33333333, 64, 127),
            (0.33333333, 65, 127),
            (0.33333333, 60, 101),
            (0.33333333, 64, 95),
            (0.33333333, 53, 101),
            (0.33333333, 48, 95),
            (0.33333333, 52, 84),
            (0.33333333, 53, 95),
            (0.33333333, 48, 84),
            (0.33333333, 52, 63),
            (1.0, 69, 84),
            (1.0, 69, 84),
            (1.0, 69, 63),
            (1.0, 60, 0),
            (0.33333333, 57, 127),
            (0.33333333, 60, 127),
            (0.33333333, 64, 127),
            (0.33333333, 65, 127),
            (0.33333333, 60, 101),
            (0.33333333, 64, 95),
            (0.33333333, 53, 101),
            (0.33333333, 48, 95),
            (0.33333333, 52, 84),
            (0.33333333, 53, 95),
            (0.33333333, 48, 84),
            (0.33333333, 52, 63),
            (1.0, 71, 84),
            (1.0, 71, 63),
            (1.0, 71, 95),
            (1.0, 71, 101),
            (0.33333333, 57, 127),
            (0.33333333, 60, 127),
            (0.33333333, 64, 127),
            (0.33333333, 65, 127),
            (0.33333333, 60, 101),
            (0.33333333, 64, 95),
            (0.33333333, 53, 101),
            (0.33333333, 48, 95),
            (0.33333333, 52, 84),
            (0.33333333, 53, 95),
            (0.33333333, 48, 84),
            (0.33333333, 52, 63),
            (1.0, 69, 84),
            (1.0, 69, 63),
            (1.0, 69, 95),
            (1.0, 69, 101),
            (0.33333333, 57, 127),
            (0.33333333, 60, 127),
            (0.33333333, 64, 127),
            (0.33333333, 65, 127),
            (0.33333333, 60, 101),
            (0.33333333, 64, 95),
            (0.33333333, 53, 101),
            (0.33333333, 48, 95),
            (0.33333333, 52, 84),
            (0.33333333, 53, 95),
            (0.33333333, 48, 84),
            (0.33333333, 52, 63),
            (4.0, 71, 127),
        ]
    });

    static FLUTE_LINE: Lazy<Vec<Midi>> = Lazy::new(|| {
        vec![
            (3.0, 45, 0),
            (1.0, 81, 95),
            (1.0, 88, 95),
            (1.0, 86, 63),
            (1.0, 88, 63),
            (1.0, 91, 63),
            (4.0, 88, 63),
            (2.0, 45, 63),
            (1.0, 45, 0),
            (1.0, 81, 127),
            (1.0, 88, 95),
            (1.0, 86, 63),
            (1.0, 88, 63),
            (1.0, 93, 95),
            (4.0, 88, 63),
            (2.0, 45, 63),
            (1.0, 45, 0),
            (1.0, 96, 127),
            (1.0, 95, 127),
            (1.0, 93, 63),
            (1.0, 91, 63),
            (1.0, 93, 95),
            (4.0, 88, 63),
            (2.0, 45, 63),
            (1.0, 45, 0),
            (1.0, 96, 127),
            (1.0, 95, 127),
            (1.0, 93, 63),
            (1.0, 91, 63),
            (1.0, 95, 95),
        ]
    });

    pub static TRACK: Lazy<PlayerTrack> = Lazy::new(|| {
        let mut parts = HashMap::new();

        let piano_spec = Spec {
            role: "bass",
            register: 5,
            fill: "frame",
            spec_type: "sine",
        };

        let flute_spec = Spec {
            role: "lead",
            register: 8,
            fill: "focus",
            spec_type: "sine",
        };

        parts.insert(flute_spec, vec![FLUTE_LINE.to_vec()]);
        parts.insert(piano_spec, vec![PIANO_LINE.to_vec()]);

        PlayerTrack {
            conf: Conf {
                origin: "A minor",
                duration: -1,
                cps: 1.75,
                title: "The X Files Theme Song",
                transposition: 10,
            },
            composition: Composition {
                composition_id: -1,
                duration: -1,
                quality: "minor",
                dimensions: Dimensions {
                    size: 4,
                    cpc: 3,
                    base: 2,
                },
                progression: vec![(48, "A minor")],
                parts,
            },
        }
    });
}
