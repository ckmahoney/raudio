use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name:&str = "bass-bird";

use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Stem, Melody, Feel};
use crate::presets::Instrument;
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::analysis::volume::db_to_amp;

use presets::hop::bass;

fn bass_melody() -> Melody<Note> {
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
        (4i32, 1i32), 
        (4i32, 1i32),
        (2i32, 1i32), 
        (6i32, 1i32), 
    ];

    let amps:Vec<Ampl> = vec![1f32; tala.len()];

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

fn bass_arf(visibility:Visibility, energy:Energy, presence:Presence) -> Arf {
    Arf {
        mode: Mode::Melodic,
        role: Role::Bass,
        register: 5,
        visibility,
        energy,
        presence,
    }
}



fn demonstrate() {
    let path:String = location(demo_name);
    files::with_dir(&path);

    use rand::Rng;
    let mut rng = rand::thread_rng();

    let cps:f32 = 2.12;
    let root:f32 = 1.09f32;

    let delays:Vec<DelayParams> = vec![delay::passthrough];

    let bass_melody = bass_melody();

    let stem_bass1 = bass::renderable(&bass_melody, &bass_arf(Visibility::Foreground, Energy::Low, Presence::Legato));
    let stem_bass2 = bass::renderable(&bass_melody, &bass_arf(Visibility::Foreground, Energy::Medium, Presence::Legato));
    let stem_bass3 = bass::renderable(&bass_melody, &bass_arf(Visibility::Visible, Energy::High, Presence::Legato));

    use Renderable::{Instance,Group};
    let renderables:Vec<Renderable> = vec![
        stem_bass1,
        stem_bass2,
        stem_bass3,
    ];

    use crate::Distance;
    use crate::types::timbre::Enclosure;

    let complexity:f32 = rng.gen::<f32>().min(0.01);
    // let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Room, complexity);
    let group_reverbs: Vec<crate::reverb::convolution::ReverbParams> = vec![];
    let keep_stems = Some(path.as_str());
    let group_reverbs = vec![];
    let mix = render::combiner(cps, root, &renderables, &group_reverbs, keep_stems);
    let filename = format!("{}/{}.wav",location(demo_name), demo_name);
    render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate () {
    demonstrate()
}

