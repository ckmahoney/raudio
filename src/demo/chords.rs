use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name:&str = "just-chords";

use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Stem, Melody, Feel};
use crate::presets::Instrument;
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::analysis::volume::db_to_amp;

use presets::hop;
use presets::hill;

fn chords_melody() -> Melody<Note> {
    let tala:Vec<Duration> = vec![
        (2i32, 1i32), 
        (3i32, 1i32),
        (3i32, 1i32), 
        (4i32, 1i32), 
        (4i32, 1i32), 
        (8i32, 1i32),
        (8i32, 1i32), 
        (4i32, 1i32), 
    ];

    let amps:Vec<Ampl> = vec![1f32; tala.len()];

    let line_1:Vec<Tone> = vec![
        (5, (0i8, 0i8, 1i8)),
        (5, (-1i8, 0i8, 1i8)),
        (6, (0i8, 0i8, 1i8)),
        (6, (1i8, 0i8, 1i8)),
        (7, (0i8, 0i8, 1i8)),
        (7, (-1i8, 0i8, 1i8)),
        (8, (0i8, 0i8, 1i8)),
        (8, (1i8, 0i8, 1i8)),
    ];

    let line_2:Vec<Tone> = vec![
        (5, (0i8, 0i8, 3i8)),
        (5, (-1i8, 0i8, 3i8)),
        (6, (0i8, 0i8, 3i8)),
        (6, (1i8, 0i8, 3i8)),
        (7, (0i8, 0i8, 3i8)),
        (7, (-1i8, 0i8, 3i8)),
        (8, (0i8, 0i8, 3i8)),
        (8, (1i8, 0i8, 3i8)),
    ];

    let line_3:Vec<Tone> = vec![
        (5, (0i8, 0i8, 5i8)),
        (5, (-1i8, 0i8, 5i8)),
        (6, (0i8, 0i8, 5i8)),
        (6, (1i8, 0i8, 5i8)),
        (7, (0i8, 0i8, 5i8)),
        (7, (-1i8, 0i8, 5i8)),
        (8, (0i8, 0i8, 5i8)),
        (8, (1i8, 0i8, 5i8)),
    ];

    vec![
        zip_line(tala.clone(), line_1, amps.clone()),
        zip_line(tala.clone(), line_2, amps.clone()),
        zip_line(tala.clone(), line_3, amps.clone()),
    ]
}


fn chords_arf(visibility:Visibility, energy:Energy, presence:Presence) -> Arf {
    Arf {
        mode: Mode::Melodic,
        role: Role::Chords,
        register: 6,
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

    let cps:f32 = 2f32;
    let root:f32 = 1.02;

    let delays:Vec<DelayParams> = vec![delay::passthrough];

    let chords_melody = chords_melody();
    
    let stem_chords2 = hill::chords::renderable(cps, &chords_melody, &chords_arf(Visibility::Background, Energy::Low,  Presence::Tenuto));
    let stem_chords1 = hill::chords::renderable(cps, &chords_melody, &chords_arf(Visibility::Foreground, Energy::Medium, Presence::Tenuto));
    let stem_chords3 = hill::chords::renderable(cps, &chords_melody, &chords_arf(Visibility::Hidden, Energy::High,  Presence::Tenuto));
    

    use Renderable2::{Instance,Group};
    let renderables:Vec<Renderable2> = vec![
        stem_chords2,
        stem_chords1,
        stem_chords3,
    ];

    use crate::Distance;
    use crate::types::timbre::Enclosure;

    let complexity:f32 = rng.gen::<f32>().min(0.01);
    // let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Room, complexity);
    let group_reverbs: Vec<crate::reverb::convolution::ReverbParams> = vec![];
    let keep_stems = Some(path.as_str());
    let group_reverbs = vec![];
    let mix = render::combiner_with_reso(cps, root, &renderables, &group_reverbs, keep_stems);
    let filename = format!("{}/{}.wav",location(demo_name), demo_name);
    render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate () {
    demonstrate()
}

