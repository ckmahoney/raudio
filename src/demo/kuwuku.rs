use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name:&str = "kuwuku";

use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Stem, Melody, Feel};
use crate::presets::Instrument;
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::analysis::volume::db_to_amp;

use presets::kuwuku::{vibe, sine, brush, chords, kick, hats};

fn sine_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32, 1i32), 
        (3i32, 4i32),
        (5i32, 4i32),
        (1i32, 1i32),
        (1i32, 2i32),
        (1i32, 2i32),
        (1i32, 1i32), 
        (2i32, 1i32)
    ];

    let amps:Vec<Ampl> = vec![
        1f32, 0.66f32, 0.66f32, 1f32,
        0.66f32, 1f32, 0.66f32, 0.75f32
    ].iter().map(|x| x * db_to_amp(-6f32)).collect::<Vec<f32>>();

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

fn chords_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (2i32, 1i32), 
        (2i32, 1i32),
        (2i32, 1i32), 
        (2i32, 1i32), 
    ];

    let amps:Vec<Ampl> = vec![
        1f32, 0.66f32, 0.66f32, 1f32,
    ].iter().map(|x| x * db_to_amp(-30f32)).collect::<Vec<f32>>();

    let line_1:Vec<Tone> = vec![
        (8, (0i8, 0i8, 1i8)),
        (8, (-1i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (1i8, 0i8, 1i8)),
    ];

    let line_2:Vec<Tone> = vec![
        (8, (0i8, 0i8, 3i8)),
        (8, (-1i8, 0i8, 3i8)),
        (8, (0i8, 0i8, 3i8)),
        (8, (1i8, 0i8, 3i8)),
    ];

    let line_3:Vec<Tone> = vec![
        (8, (0i8, 0i8, 5i8)),
        (8, (-1i8, 0i8, 5i8)),
        (8, (0i8, 0i8, 5i8)),
        (8, (1i8, 0i8, 5i8)),
    ];

    vec![
        zip_line(tala.clone(), line_1, amps.clone()),
        zip_line(tala.clone(), line_2, amps.clone()),
        zip_line(tala.clone(), line_3, amps.clone()),
    ]
}

fn kick_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32,1i32), 
        (1i32,1i32), 
        (2i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,2i32), 
        (1i32,2i32), 
    ];

    let amps:Vec<Ampl> = vec![
        1f32, 0.66f32, 1f32, 
        1f32, 0.5f32, 0.75f32, 1f32, 0.66f32
    ].iter().map(|x| x * db_to_amp(-12f32)).collect::<Vec<f32>>();

    let tones:Vec<Tone> = vec![
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
        (5, (0i8, 0i8, 1i8)),
    ];

    vec![
        zip_line(tala, tones, amps)
    ]
}


fn hats_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (-1i32,2i32), 
        (1i32,2i32),
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (1i32,2i32), 
        (-1i32,2i32), 
        (1i32,2i32), 
        (-1i32,2i32), 
        (1i32,2i32), 
    ];

    let amps:Vec<Ampl> = vec![
        0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32,
        0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32,
    ].iter().map(|x| x * db_to_amp(-24f32)).collect::<Vec<f32>>();

    let tones:Vec<Tone> = vec![
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
        (12, (0i8, 0i8, 1i8)),
    ];

    vec![
        zip_line(tala, tones, amps)
    ]
}


fn brush_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
        (1i32,1i32), 
    ];

    let amps:Vec<Ampl> = vec![
        0f32, 0.66f32, 0f32, 0.75f32,
        0f32, 0.66f32, 0f32, 0.5f32,
    ].iter().map(|x| x * db_to_amp(-12f32)).collect::<Vec<f32>>();

    let tones:Vec<Tone> = vec![
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
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
        energy: Energy::Medium,
        presence: Presence::Legato,
    }
}


fn chords_arf() -> Arf {
    Arf {
        mode: Mode::Melodic,
        role: Role::Chords,
        register: 8,
        visibility: Visibility::Background,
        energy: Energy::Medium,
        presence: Presence::Tenuto,
    }
}


fn kick_arf() -> Arf {
    Arf {
        mode: Mode::Enharmonic,
        role: Role::Perc,
        register: 5,
        visibility: Visibility::Visible,
        energy: Energy::Medium,
        presence: Presence::Tenuto,
    }
}


fn brush_arf() -> Arf {
    Arf {
        mode: Mode::Enharmonic,
        role: Role::Perc,
        register: 7,
        visibility: Visibility::Visible,
        energy: Energy::Low,
        presence: Presence::Staccatto,
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

    use rand::Rng;
    let mut rng = rand::thread_rng();

    let cps:f32 = 1.15;
    let root:f32 = 1.2;
    let labels:Vec<&str> = vec!["vibe", "sine", "brush"];
    let ely_chords = vibe::driad(&vibe_arf());
    let ely_brush = brush::driad(&brush_arf());
    let ely_sine = sine::driad(&sine_arf());
    let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);

    let feel_brush:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_brush.modders,
        clippers: (0f32, 1f32)
    };

    let feel_sine:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_sine.modders,
        clippers: (0f32, 1f32)
    };

    let feel_chords:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_chords.modders,
        clippers: (0f32, 1f32)
    };

    let delays:Vec<DelayParams> = vec![delay::passthrough];
    let vibe_melody = vibe_melody();
    let hats_melody = hats_melody();
    let chords_melody = chords_melody();
    let stem_vibe = vibe::stem(&vibe_melody, &vibe_arf());
    let stem_hats = hats::renderable(&hats_melody, &vibe_arf());
    let stem_chords =chords::renderable(&chords_melody, &chords_arf());
    let stem_sine = (&sine_melody(), ely_sine.soids, expr, feel_sine, ely_sine.knob_mods, vec![delay::passthrough]);
    let stem_brush = (&brush_melody(), ely_brush.soids, brush::expr(&brush_arf()), feel_brush, ely_brush.knob_mods, vec![delay::passthrough]);
    let kick_mel = kick_melody();
    let group_kick = kick::grouping(&kick_mel, &kick_arf());

    use Renderable::*;
    let renderables:Vec<Renderable> = vec![
        stem_vibe,
        stem_hats,
        stem_chords,
        Instance(stem_sine),
        Instance(stem_brush),
        Group(group_kick)
    ];

    // let group_reverbs:Vec<reverb::convolution::ReverbParams> = vec![];
    use crate::Distance;
    use crate::types::timbre::{Enclosure};

    let complexity:f32 = rng.gen::<f32>();
    let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Vast, complexity);
    let keep_stems = Some(path.as_str());

    let mix = render::combiner(cps, root, &renderables, &group_reverbs, keep_stems);
    let filename = format!("{}/{}.wav",location(demo_name), demo_name);
    render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate() {
    demonstrate()
}