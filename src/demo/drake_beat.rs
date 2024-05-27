use super::out_dir;
use std::iter::FromIterator;

use crate::files;
use crate::synth::{MF, NF, SR, SampleBuffer, pi, pi2};
use crate::types::synthesis::{Ampl, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::{Melody};
use crate::types::timbre::{Visibility, Mode, Role, Contrib, FilterMode, Sound, Sound2, Energy, Presence, Timeframe, Phrasing,AmpLifespan, AmpContour};
use crate::{presets, render};
use crate::time;
use presets::{PartialModulators, DModulators, kick, snare};

static demo_name:&str = "drakebeat";


/// helper for making a test line of specific length with arbitrary pitch.
fn make_line(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, overs:bool) -> Vec<Note> {
    let len = durations.len();
    if len != registers.len() || len != amps.len() {
        panic!("Must provide the same number of components per contributor. Got actual lengths for duration {} register {} amp {}", len, registers.len(), amps.len());
    }

    let mut line:Vec<Note> = Vec::with_capacity(len);
    for (i, duration) in durations.iter().enumerate() {
        let register = registers[i];
        let amp = amps[i];
        line.push(test_note(*duration, register, amp, overs))
    }
    line
}

/// helper for making a test monophonic melody of specific length with arbitrary pitch.
fn make_melody(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, overs:bool) -> Melody<Note> {
    vec![
        make_line(durations, registers, amps, overs)
    ]
}

/// given a length, duration, ampltidue, and space selection, 
/// create a note in the register.
fn test_note(duration:Duration, register:i8, amp:f32, overs:bool) -> Note {
    let monic:i8 = 1;
    let rotation:i8 = 0;
    
    let q:i8 = if overs { 0 } else { 1 };
    let monic = 1;
    let monae:Monae = (rotation,q, monic);
    (duration, (register, monae), amp)
}

/// Produces the (kick, perc, hats) tuple of melodies for a popluar percussive rhythm in range of 60-84BPM
fn make_melodies() -> [Melody<Note>; 3] {
    let tala_hats:Vec<Duration> = vec![(1i32, 4i32); 16];
    let tala_perc:Vec<Duration> = vec![
        (1i32,1i32), // rest
        (3i32,4i32),
        (5i32,4i32),
        (1i32,1i32)
    ];
    let tala_kick:Vec<Duration> = vec![
        (1i32, 2i32),
        (1i32, 2i32),
        (1i32, 1i32), // rest
        (2i32, 1i32)
    ];

    let amp_hats:Vec<Ampl> = vec![
        0.5f32, 0.5f32, 1f32, 0.5f32,
        0.5f32, 0.5f32, 0.66f32, 0.5f32,
        0.5f32, 0.5f32, 0.75f32, 0.5f32,
        0.5f32, 0.5f32, 0.66f32, 0.5f32,
    ];

    let amp_perc:Vec<Ampl> = vec![
        0f32, 1f32, 0.66f32, 0.75f32
    ];

    let amp_kick:Vec<Ampl> = vec![
        1f32, 0.66f32, 0f32, 1f32
    ];

    let register_hats:Vec<i8> = vec![10i8; amp_hats.len()];
    let register_perc:Vec<i8> = vec![8i8; amp_perc.len()];
    let register_kick:Vec<i8> = vec![5i8; amp_kick.len()];

    [
        make_melody(tala_kick, register_kick, amp_kick, true),
        make_melody(tala_perc, register_perc, amp_perc, true),
        make_melody(tala_hats, register_hats, amp_hats, true),
    ]
}

fn make_sounds() -> [Sound2; 3] {
    let allpass = (FilterMode::Linear, FilterPoint::Constant, (MF as f32, NF as f32));

    let sound_hats:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };

    let sound_perc:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };

    let sound_kick:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };

    [   
        sound_kick,
        sound_perc, 
        sound_hats
    ]
}

fn make_synth(sound:&Sound2) {

}

fn get_contribs() -> [Contrib;3] {
    let kick:Contrib = Contrib {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Low,
        presence: Presence::Legato,
    };

    let snare:Contrib = Contrib {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Low,
        presence: Presence::Legato,
    };

    let hats:Contrib = Contrib {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Low,
        presence: Presence::Legato,
    };

    [kick, snare, hats]
}


fn demonstrate() {
    let cps:f32 = 1.15;
    let labels:Vec<&str> = vec!["kick", "perc", "hat"];
    let melodies = make_melodies();
    let sounds = make_sounds();
    // let synths = make_synths(&sounds);

    // let mut buffs:Vec<SampleBuffer> = Vec::with_capacity(melodies.len());

    // files::with_dir(out_dir);
    // for (i, label) in labels.iter().enumerate() {
    //     let filename = format!("{}/{}-{}.wav", out_dir, demo_name, label);
        
        
    //     render::engrave::samples(SR, &signal, &filename);
    //     buffs.push(signal);
    //     println!("Rendered stem {}", filename);
    // }

    // match render::realize::mix_buffers(&mut buffs) {
    //     Ok(mixdown) => {
    //         let filename = format!("{}/{}.wav", out_dir, demo_name);
    //         render::engrave::samples(SR, &mixdown, &filename);
    //         println!("Rendered mixdown {}", filename);
    //     },
    //     Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    // }
}

#[test]
fn test() {
    demonstrate()
}
