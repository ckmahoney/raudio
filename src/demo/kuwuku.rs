use super::*;
use crate::analysis::delay;
use crate::files;

static demo_name:&str = "kuwuku";

use crate::render;
use crate::reverb;
use crate::types::render::{Stem, Melody, Feel, Instrument};
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use presets::kuwuku::{vibe, sine};

fn sine_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32,1i32), 
        (3i32,4i32),
        (5i32,4i32),
        (1i32, 1i32),
        (1i32, 2i32),
        (1i32, 2i32),
        (1i32, 1i32), 
        (2i32, 1i32)
    ];

    let amps:Vec<Ampl> = vec![
        1f32, 0.66f32, 0.66f32, 1f32,
        0.66f32, 1f32, 0.66f32, 0.75f32
    ];

    let tones:Vec<Tone> = vec![
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 5i8)),
        (5, (0i8, 0i8, 3i8)),
        (5, (1i8, 0i8, 1i8)),
        (5, (1i8, 0i8, 5i8)),
        (5, (1i8, 0i8, 3i8)),
        (5, (-1i8, 0i8, 5i8)),
        (5, (-1i8, 0i8, 1i8)),
    ];

    vec![
        zip_line(tala, tones, amps)
    ]
}

fn vibe_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32,1i32), 
        (3i32,2i32), 
        (1i32,1i32),
        (1i32,1i32),
        (3i32, 2i32),
        (1i32, 1i32),
        (1i32, 1i32),
    ];

    let amps:Vec<Ampl> = vec![
        1f32, 0.5f32, 0.66f32,
        0.5f32, 1f32, 0.5f32, 0.75f32
    ];

    let tones:Vec<Tone> = vec![
        (7, (0i8, 0i8, 5i8)),
        (8, (0i8, 0i8, 3i8)),
        (8, (1i8, 0i8, 1i8)),
        (7, (1i8, 0i8, 5i8)),
        (8, (1i8, 0i8, 3i8)),
        (9, (-1i8, 0i8, 5i8)),
        (8, (-1i8, 0i8, 1i8)),
    ];

    vec![
        zip_line(tala, tones, amps)
    ]
}

fn sine_arf() -> Arf {
    Arf {
        mode: Mode::Melodic,
        role: Role::Bass,
        register: 5,
        visibility: Visibility::Visible,
        energy: Energy::Low,
        presence: Presence::Legato,
    }
}

fn vibe_arf() -> Arf {
    Arf {
        mode: Mode::Melodic,
        role: Role::Lead,
        register: 8,
        visibility: Visibility::Foreground,
        energy: Energy::Medium,
        presence: Presence::Legato,
    }
}

fn demonstrate() {
    let path:String = location(demo_name);
    files::with_dir(&path);

    let cps:f32 = 1.15;
    let root:f32 = 1.2;
    let labels:Vec<&str> = vec!["vibe", "sine", "brush"];
    let melody = vibe_melody();
    let arf = vibe_arf();
    let ely_vibe = vibe::driad(&arf);
    let ely_sine = sine::driad(&arf);
    let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
    let feel_vibe:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_vibe.modders,
        clippers: (0f32, 1f32)
    };
    let feel_sine:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_sine.modders,
        clippers: (0f32, 1f32)
    };
    let delays:Vec<DelayParams> = vec![delay::passthrough];
    let stems:[Stem;2] = [
        (&vibe_melody(), ely_vibe.soids, vibe::expr(&arf), feel_vibe, ely_vibe. knob_mods, vec![delay::passthrough]),
        (&sine_melody(), ely_sine.soids, expr, feel_sine, ely_sine.knob_mods, vec![delay::passthrough]),
    ];

    let group_reverbs:Vec<reverb::convolution::ReverbParams> = vec![];
    let keep_stems = Some(path.as_str());

    render::combine(cps, root, &stems.to_vec(), &group_reverbs, keep_stems);
}

#[test]
fn test_demonstrate() {
    demonstrate()
}