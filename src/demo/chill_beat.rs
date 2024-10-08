use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name:&str = "chill-beat";

use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Stem, Melody, Feel};
use crate::presets::Instrument;
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::analysis::volume::db_to_amp;

use presets::ambien::{ perc, kick, hats};


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


fn perc_melody() -> Melody<Note> {
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


fn perc_arf() -> Arf {
    Arf {
        mode: Mode::Enharmonic,
        role: Role::Perc,
        register: 7,
        visibility: Visibility::Visible,
        energy: Energy::Low,
        presence: Presence::Staccatto,
    }
}
fn hats_arf() -> Arf {
    Arf {
        mode: Mode::Enharmonic,
        role: Role::Hats,
        register: 12,
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

    let cps:f32 = 1.2;
    let cps:f32 = 2.1;
    let root:f32 = 1.9;
    let labels:Vec<&str> = vec!["vibe", "sine", "brush"];

    let delays:Vec<DelayParams> = vec![delay::passthrough];

    let hats_melody = hats_melody();
    let perc_melody = perc_melody();
    let kick_mel = kick_melody();

    let stem_hats = hats::renderable(&hats_melody, &hats_arf());
    let stem_perc = perc::renderable(&perc_melody, &perc_arf());
    let stem_kick = kick::renderable(&kick_mel, &kick_arf());

    use Renderable::{Instance,Group};
    let renderables:Vec<Renderable> = vec![
        stem_kick,
        stem_perc,
        stem_hats,
    ];

    use crate::Distance;
    use crate::types::timbre::Enclosure;

    let complexity:f32 = rng.gen::<f32>();
    let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Vast, complexity);
    let keep_stems = Some(path.as_str());

    let mix = render::combiner(cps, root, &renderables, &group_reverbs, keep_stems);
    let filename = format!("{}/{}.wav",location(demo_name), demo_name);
    render::engrave::samples(SR, &mix, &filename);
}



fn samp(cps:f32, root:f32) -> SampleBuffer {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    

    let delays:Vec<DelayParams> = vec![delay::passthrough];

    let hats_melody = hats_melody();
    let perc_melody = perc_melody();
    let kick_mel = kick_melody();

    let stem_hats = hats::renderable(&hats_melody, &hats_arf());
    let stem_perc = perc::renderable(&perc_melody, &perc_arf());
    let stem_kick = kick::renderable(&kick_mel, &kick_arf());

    use Renderable::{Instance,Group};
    let renderables:Vec<Renderable> = vec![
        stem_kick,
        stem_perc,
        stem_hats,
    ];

    use crate::Distance;
    use crate::types::timbre::Enclosure;

    let complexity:f32 = rng.gen::<f32>();
    let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Vast, complexity);

    render::combiner(cps, root, &renderables, &group_reverbs, None)
}

#[test]
fn test_demonstrate() {
    demonstrate()
}

#[test]
fn test_hypnosis() {
    let path:String = location(demo_name);
    files::with_dir(&path);

    let mut track:SampleBuffer = vec![];
    let mut ring:SampleBuffer = vec![];

    let n_versions = 8;
    let n_loops = 4;

    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut root:f32 = rng.gen::<f32>();
    let base_cps:f32 = 1.2f32 + rng.gen::<f32>();
    let mut cps:f32 = base_cps;

    for i in 0..n_versions {
        ring = samp(cps, root);
        for j in 0..n_loops {
            track.extend(&ring)
        }

        root *= 1.5f32;
        if root > 2f32 {
            root /= 2f32;
        };

        cps *= 1.5f32;
        if cps > base_cps * 3f32 {
            cps /= 3f32;
        };
    }

    let filename = format!("{}/hypnoloop_{}.wav",location(demo_name), demo_name);
    render::engrave::samples(SR, &track, &filename);
}


#[test]
fn test_render_playbook() {
    crate::render_playbook("/media/naltroc/engraver 2/music-gen/demo/chill-beat-extra-square/test_ambien_playbook", "src/demo/playbook-demo-ambien.json", "test-preset-ambien")
}