use crate::types::synthesis::{Note, Progression};
use crate::types::timbre::{Positioning, Distance,Echo, Role, Visibility, Energy, Mode, Presence, Arf};
use crate::types::render::{Midi, ScoreEntry, PlayerTrack, Melody, Conf, Dimensions, Entry};


/// Monic representation of Happy Birthday
/// This song is public domain
pub const root:f32 = 1.3f32;
pub const cps:f32 = 2.1f32;


pub fn lead_melody() -> Melody<Note> {
    vec![
        vec![
            ((3, 4), (6, (0, 0, 3)), 1.0),
            ((1, 4), (6, (0, 0, 3)), 1.0),
            
            ((1, 1), (6, (-1, 0, 5)), 1.0),
            ((1, 1), (6, (0, 0, 3)), 1.0),
            ((1, 1), (7, (0, 0, 1)), 1.0),

            ((2, 1), (6, (1, 0, 5)), 1.0),
            ((3, 4), (6, (1, 0, 1)), 1.0),
            ((1, 4), (6, (1, 0, 1)), 1.0),
            
            ((1, 1), (6, (2, 0, 3)), 1.0),
            ((1, 1), (6, (1, 0, 1)), 1.0),
            ((1, 1), (7, (1, 0, 3)), 1.0),

            ((2, 1), (7, (0, 0, 1)), 1.0),
            ((3, 4), (6, (0, 0, 3)), 1.0),
            ((1, 4), (6, (0, 0, 3)), 1.0),

            ((1, 1), (7, (0, 0, 3)), 1.0),
            ((1, 1), (7, (0, 0, 5)), 1.0),
            ((1, 1), (7, (0, 0, 1)), 1.0),

            ((1, 1), (6, (1, 0, 5)), 1.0),
            ((1, 1), (6, (-1, 0, 5)), 1.0),
            ((3, 4), (7, (3, 1, 5)), 1.0),
            ((1, 4), (7, (3, 1, 5)), 1.0),
            
            ((1, 1), (7, (0, 0, 5)), 1.0),
            ((1, 1), (7, (0, 0, 1)), 1.0),
            ((1, 1), (7, (1, 0, 3)), 1.0),

            ((3, 1), (7, (0, 0, 1)), 1.0)
        ]
    ]
}

/// performed in the key of (0, 0) 
pub fn get_track() -> PlayerTrack<Note> {
    let progression: Progression = vec![
        ((1, 1), (0, 0)),

        ((3, 1), (0, 0)),
        ((3, 1), (1, 0)),
        ((3, 1), (1, 0)),
        ((3, 1), (0, 0)),

        ((3, 1), (0, 0)),
        ((3, 1), (4, 1)),

        ((2, 1), (0, 0)),
        ((1, 1), (0, 0)),

        ((3, 1), (0, 0))
    ];

    let lead: Melody<Note> = lead_melody();

    let flute:Entry = (
        Arf {
            role: Role::Lead,
            register: 8,
            mode: Mode::Melodic,
            visibility: Visibility::Visible,
            energy: Energy::High,
            presence: Presence::Tenuto,
            positioning: Positioning {
                distance: Distance::Adjacent,
                echo: Echo::None,
                complexity: 0f32
            }
        },
        lead
    );
    let parts: Vec<ScoreEntry<Note>> = vec![flute];
    PlayerTrack {
        conf: Conf {
            root,
            cps
        },
        duration: parts[0].1.iter().fold(0f32, |acc_melody, line|
            acc_melody + line.iter().fold(0f32, |acc, &note| acc + note.0.1 as f32/note.0.0 as f32)
        ),
        dimensions: Dimensions {
            size: 4i8,
            cpc: 3i16,
            base: 2i8,
        },
        parts,
    }
}

