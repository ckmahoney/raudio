use crate::midi::*;
use crate::types::synthesis::*;
use crate::types::timbre::*;
use crate::types::render::*;

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
                presence: Presence::Legato
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
}

pub mod happy_birthday {
    use super::*;

    /// in the key of 0 Major 
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

        let lead: Melody<Note> = vec![
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
        ];

        let flute:Entry = (
            Contrib {
                role: Role::Lead,
                register: 8,
                mode: Mode::Melodic,
                visibility: Visibility::Visible,
                energy: Energy::High,
                presence: Presence::Tenuto
            },
            lead
        );
        let parts: Vec<ScoreEntry<Note>> = vec![flute];
        PlayerTrack {
            conf: Conf {
                root: 1.3f32,
                cps: 1.1f32,
                cube: Cube::Room
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
}


#[test]
fn test_check_notes() {
    use crate::monic_theory::{Chrom, tone_to_chrom};
    let tones:Vec<Tone> = happy_birthday::get_track().parts[0].1[0].iter().map(|p| p.1).collect::<Vec<Tone>>();
    let chroms:Vec<Chrom> = tones.into_iter().map(|t| tone_to_chrom(t)).collect::<Vec<Chrom>>();
        
    let expected:Vec::<i8> = vec![
        43,
        43,
        45,
        43,
        48,
        47,
        43,
        43,
        45,
        43,
        50,
        48,
        43,
        43,
        55,
        52,
        48,
        47,
        45,
        53,
        53,
        52,
        48,
        50,
        48,
    ];
    println!("Got these chroms for the happy birthday melody");
    println!("{:#?}", chroms);
    assert_eq!(expected, chroms);
}