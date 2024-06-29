use super::out_dir;
use std::iter::FromIterator;

use crate::files;
use crate::synth::{MF, NF, SR, SampleBuffer, pi, pi2};
use crate::types::synthesis::{Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::{Melody};
use crate::types::timbre::{Visibility, Mode, Role, Arf, FilterMode, Sound, Sound2, Energy, Presence, Timeframe, Phrasing,AmpLifespan, AmpContour};
use crate::{presets, render};
use crate::time;
use presets::{kick, kick_hard, snare, snare_hard, hats, hats_hard};

use crate::phrasing::lifespan;
use crate::druid::{Elementor, Element, ApplyAt, melody_frexer, inflect};

static demo_name:&str = "beat";

fn make_synths(arfs:&[Arf;3]) -> [Elementor; 3] {
    [kick_hard::synth(&arfs[0]), snare_hard::synth(&arfs[1]), hats_hard::synth(&arfs[2])]
}

/// helper for making a test line of specific length with arbitrary pitch.
fn make_line(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, muls:bool) -> Vec<Note> {
    let len = durations.len();
    if len != registers.len() || len != amps.len() {
        panic!("Must provide the same number of components per arfutor. Got actual lengths for duration {} register {} amp {}", len, registers.len(), amps.len());
    }

    let mut line:Vec<Note> = Vec::with_capacity(len);
    for (i, duration) in durations.iter().enumerate() {
        let register = registers[i];
        let amp = amps[i];
        line.push(test_note(*duration, register, amp, muls))
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

    let register_hats:Vec<i8> = vec![11i8; amp_hats.len()];
    let register_perc:Vec<i8> = vec![8i8; amp_perc.len()];
    let register_kick:Vec<i8> = vec![6i8; amp_kick.len()];

    [
        make_melody(tala_kick, register_kick, amp_kick, true),
        make_melody(tala_perc, register_perc, amp_perc, true),
        make_melody(tala_hats, register_hats, amp_hats, true),
    ]
}

struct Coverage<'hyper> {
    label: &'hyper str,
    mode: Vec<Mode>,
    role: Vec<Role>,
    register: Vec<Register>,
    visibility: Vec<Visibility>,
    energy: Vec<&'hyper Energy>,
    presence: Vec<&'hyper Presence>
}

fn make_specs<'hyper>() -> [Coverage<'hyper>; 3] {
    use Mode::*;
    use Role::*;
    use Visibility::*;
    use Energy::*;
    use Presence::*;

    let kick = Coverage {
        label: "kick",
        mode: vec![Melodic, Enharmonic],
        // mode: vec![],
        role: vec![Kick],
        // register: vec![5,6,7],
        register: vec![5],
        visibility: vec![Visibility::Visible],
        energy: vec![&Energy::Low,&Energy::High],
        presence: vec![&Presence::Staccatto, &Presence::Tenuto, &Presence::Legato],
    };

    let snare = Coverage {
        label: "snare",
        mode: vec![Noise, Enharmonic],
        // mode: vec![],
        role: vec![Perc],
        // register: vec![7,8,9],
        register: vec![8],
        visibility: vec![Visibility::Visible,Visibility::Hidden],
        energy: vec![&Energy::Low, &Energy::High],
        presence: vec![&Presence::Staccatto, &Presence::Tenuto, &Presence::Legato],
    };

    let hats = Coverage {
        label: "hats",
        mode: vec![Bell],
        role: vec![Hats],
        // register: vec![10,11,12],
        register: vec![10],
        visibility: vec![Visibility::Visible, Visibility::Hidden],
        energy: vec![&Energy::Low, &Energy::High],
        presence: vec![&Presence::Staccatto, &Presence::Tenuto, &Presence::Legato],
    };

    [kick, snare, hats]
}

/// Iterate the available VEP parameters to produce a beat trio
fn gen_arfs(spec:&Coverage) -> Vec<(String, Arf)> {
    let mut arfs: Vec<(String, Arf)> = Vec::new();

    for mode in &spec.mode {
        for role in &spec.role {
            for register in &spec.register {
                for visibility in &spec.visibility {
                    for energy in &spec.energy {
                        for presence in &spec.presence {
                            let variant_name = format!(
                                "mode-{:?}-role-{:?}-register-{:?}-visibility-{:?}-energy-{:?}-presence-{:?}",
                                mode, role, register, visibility, energy, presence
                            );

                            let arf = Arf {
                                mode: *mode,
                                role: *role,
                                register: register.clone(),
                                visibility: (*visibility).clone(),
                                energy: **energy,
                                presence: **presence,
                            };

                            arfs.push((variant_name, arf))
                        }
                    }
                }
            }
        }
    }

    arfs
}


fn get_arfs() -> [Arf;3] {
    let kick:Arf = Arf {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Low,
        presence: Presence::Legato,
    };

    let snare:Arf = Arf {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Medium,
        presence: Presence::Legato,
    };

    let hats:Arf = Arf {
        mode: Mode::Melodic,
        role: Role::Kick,
        register: 5,
        visibility: Visibility::Foreground,
        energy: Energy::Low,
        presence: Presence::Legato,
    };

    [kick, snare, hats]
}


fn enumerate() {
    let cps:f32 = 1.15;
    let labels:Vec<&str> = vec!["kick", "perc", "hat"];
    let melodies = make_melodies();
    let arfs = get_arfs();
    let synths = make_synths(&arfs);
    let specs = make_specs();
    files::with_dir(out_dir);

    for (i, spec) in specs.iter().enumerate() {
        
        let labelled_arfs = gen_arfs(&spec);
        for (label, arf) in labelled_arfs {
            let mut channels:Vec<SampleBuffer> = render_arf(cps, &melodies[i], &synths[i], arf);
            match render::realize::mix_buffers(&mut channels.clone()) {
                Ok(mixdown) => {
                    let filename = format!("{}/{}-{}.wav", out_dir, demo_name, label);
                    render::engrave::samples(SR, &mixdown, &filename);
                    println!("Rendered stem {}", filename);
                },
                Err(msg) => panic!("Error while preparing mixdown: {}", msg)
            }
        }
    }
}

fn render_arf(cps:f32, melody:&Melody<Note>, synth:&Elementor, arf:Arf) -> Vec<SampleBuffer> {
    let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
    let mut channels:Vec<SampleBuffer> = Vec::with_capacity(melody.len());
        
    for (index, line_frexd) in melody_frexd.iter().enumerate() {
        let mut line_buff:SampleBuffer = Vec::new();
        let line = &melody[index];

        for (jindex, frex) in line_frexd.iter().enumerate() {
            let dur = time::duration_to_cycles(line[jindex].0);
            let amp = line[jindex].2;
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            let applied:Elementor = synth.iter().map(|(w, r)| (*w * amp, *r)).collect();
            line_buff.append(&mut inflect(
                &frex, 
                &at, 
                &applied, 
                &arf.visibility,
                &arf.energy,
                &arf.presence
            ));
        }
        channels.push(line_buff)
    }
    channels
}


fn demonstrate(selection:Option<usize>) {
    let cps:f32 = 1.15;
    let labels:Vec<&str> = vec!["kick", "perc", "hat"];
    let melodies = make_melodies();
    let arfs = get_arfs();
    let synths = make_synths(&arfs);

    files::with_dir(out_dir);

    let mut stems:Vec<SampleBuffer> = Vec::with_capacity(melodies.len());


    for (i, label) in labels.iter().enumerate() {
        if selection.is_some() && selection.unwrap()!= i {
            println!("Skipping test perc {}",i);
            continue
        }
        let melody = &melodies[i];
        let synth = &synths[i];
        let arf = &arfs[i];
        let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
        let filename = format!("{}/{}-{}.wav", out_dir, demo_name, label);
        let mut channels:Vec<SampleBuffer> = Vec::with_capacity(melody.len());

        for (index, line_frexd) in melody_frexd.iter().enumerate() {
            let mut line_buff:SampleBuffer = Vec::new();
            let line = &melody[index];

            for (jindex, frex) in line_frexd.iter().enumerate() {
                let dur = time::duration_to_cycles(line[jindex].0);
                let at = ApplyAt { frex: *frex, span: (cps, dur) };
                line_buff.append(&mut inflect(
                    &frex, 
                    &at, 
                    &synth, 
                    &arf.visibility,
                    &arf.energy,
                    &arf.presence
                ));
            }
            channels.push(line_buff)
        }

        match render::realize::mix_buffers(&mut channels.clone()) {
            Ok(mixdown) => {
                let filename = format!("{}/{}-{}.wav", out_dir, demo_name, label);
                render::engrave::samples(SR, &mixdown, &filename);
                println!("Rendered stem {}", filename);
                stems.push(mixdown)
            },
            Err(msg) => panic!("Error while preparing mixdown: {}", msg)
        }
    }

    match render::realize::mix_buffers(&mut stems) {
        Ok(mixdown) => {
            let filename = format!("{}/{}.wav", out_dir, demo_name);
            render::engrave::samples(SR, &mixdown, &filename);
            println!("Rendered mixdown {}", filename);
        },
        Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        demonstrate(None);
        // enumerate();
    }
}
