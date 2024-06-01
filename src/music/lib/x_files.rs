use crate::midi::{Midi};
use crate::types::synthesis::{Note, Progression};
use crate::types::timbre::{Role, Visibility, Energy, Mode, Presence, Cube, Contrib};
use crate::types::render::{ScoreEntry, PlayerTrack, Melody, Conf, Dimensions};

/// MIDI representation of the X-Files theme
/// This song is under copyright; be careful

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
        Contrib {
            role: Role::Bass,
            register: 5,
            mode: Mode::Melodic,
            visibility: Visibility::Foreground,
            energy:Energy::High,
            presence:Presence::Tenuto,

        },
        vec![PIANO_LINE.to_vec()]
    );

    let flute:ScoreEntry<Midi> = (
        Contrib {
            role: Role::Lead,
            register: 8,
            mode: Mode::Melodic,
            visibility: Visibility::Visible,
            energy: Energy::High,
            presence: Presence::Legato,

        },
        vec![FLUTE_LINE.to_vec()]
    );
    let parts = vec![piano, flute];
    PlayerTrack {
        conf: Conf {
            root: 1.3f32,
            cps: 1.2f32,
            cube: Cube::Room
        },
        duration: 132f32,
        dimensions: Dimensions {
            size: 4i8,
            cpc: 3i16,
            base: 2i8,
        },
        parts,
    }
}
