pub use std::collections::HashMap;
use crate::midi::*;

#[derive(Debug)]
enum Fill {
    Frame,
    Support,
    Focus
}

#[derive(Debug)]
enum Visibility {
    Foreground,
    Visible,
    Background,
    Hidden,
}

#[derive(Debug)]
enum Mode {
    Melodic,
    Enharmonic,
    Vagrant,
    Noise
}

#[derive(Debug)]
enum BaseOsc {
    Sine,
    Pulse,
    Saw,
    Tri
}

#[derive(Debug)]
enum Role {
    Kick,
    Perc,
    Hats,
    Bass,
    Chords,
    Lead
}


pub type Melody<C> = Vec<Vec<C>>;


#[derive(Debug)]
pub struct ContribComp {
    pub base: BaseOsc,
    pub fill: Fill,
    pub mode: Mode,
    pub register: i32,
    pub role: Role,
    pub visibility: Visibility,
}

#[derive(Debug)]
pub struct PlayerTrack<C> {
    pub conf: Conf,
    pub composition: Composition<C>,
}

#[derive(Debug, Clone)]
pub struct Conf {
    pub origin: &'static str,
    pub duration: i32,
    pub cps: f32,
    pub title: &'static str,
    pub transposition: i32,
}

pub type ScoreEntry<C> = (ContribComp, Melody<C>);

#[derive(Debug)]
pub struct Composition<C> {
    pub composition_id: i32,
    pub duration: i32,
    pub quality: &'static str,
    pub dimensions: Dimensions,
    pub parts: Vec<ScoreEntry<C>>,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub size: i32,
    pub cpc: i32,
    pub base: i32,
}

pub mod x_files {
    use super::*;

    
    pub fn get_track() -> PlayerTrack<Midi> {
        let PIANO_LINE: Vec<Midi> = vec![
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
    ];

    let FLUTE_LINE: Vec<Midi> = vec![
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
    ];

        let piano:ScoreEntry<Midi> = (
            ContribComp {
                role: Role::Bass,
                register: 5,
                mode: Mode::Melodic,
                fill: Fill::Support,
                visibility: Visibility::Foreground,
                base: BaseOsc::Sine,
            },
            vec![PIANO_LINE.to_vec()]
        );

        let flute:ScoreEntry<Midi> = (
            ContribComp {
                role: Role::Lead,
                register: 8,
                mode: Mode::Melodic,
                visibility: Visibility::Visible,
                fill: Fill::Focus,
                base: BaseOsc::Sine,
            },
            vec![FLUTE_LINE.to_vec()]
        );
        let parts = vec![piano, flute];
        PlayerTrack {
            conf: Conf {
                origin: "A minor",
                duration: -1,
                cps: 1.2,
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
                parts,
            }
        }
    }
}
