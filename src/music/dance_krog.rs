use rand::thread_rng;

use crate::synth::{MF, NF,SampleBuffer,SR};
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};

use crate::engrave; 
use crate::phrasing::contour; 
use crate::phrasing::lifespan; 
use crate::preset; 
use crate::presets; 
use crate::render; 
use crate::files;
use rand;
use rand::Rng;
use rand::seq::SliceRandom;
static out_dir:&str = "audio/music/amb";

fn make_pad(cps:f32, fundamental:f32, n_cycles:f32, amp:f32) -> SampleBuffer {
    let mut rng = thread_rng();

    let mut ls = &lifespan::lifespans;
    let mut lifespan_opts:Vec<usize> = (0..ls.len()).collect();
    lifespan_opts.shuffle(&mut rng);


    let n_buckets = 2usize;
    let lifespans:Vec<&AmpLifespan> = lifespan_opts.iter().take(n_buckets).map(|i| &ls[*i]).collect();

    

    let o = (rng.gen::<f32>() * 4f32) as i32 - 2;
    let d = (rng.gen::<f32>() * 2f32) as i32 - 1;

    let mut pad_a = presets::melodic::gen_pad(cps, amp, 1.5f32, 2f32.powi(o)*fundamental, 2, 5, n_cycles/4f32);
    let mut pad_b = presets::melodic::gen_pad(cps,amp, 1.5f32, 2f32.powi(o+d)*fundamental, 2, 5, n_cycles/2f32);

    let l1 = lifespan::mod_lifespan(pad_a.len(), n_cycles/4f32, lifespans[0], 1, 0f32);
    let l2 = lifespan::mod_lifespan(pad_b.len(), n_cycles/2f32, lifespans[1], 1, 0f32);

    contour::apply_contour(&mut pad_a, &l1);
    contour::apply_contour(&mut pad_b, &l2);

    let mut pad:Vec<f32> = Vec::with_capacity(pad_b.len()*2);
    pad.extend(pad_a.iter());
    pad.append(&mut pad_b);
    pad.extend(pad_a.iter());

    render::amp_scale(&mut pad, amp);

    pad
}

fn gen_kick(cps:f32, fundamental:f32, n_cycles:f32) -> SampleBuffer {
    use crate::presets;

    let mbs:preset::SomeModulators = preset::SomeModulators {
        amp: Some(presets::kick::amod),
        freq: Some(presets::kick::fmod),
        phase: Some(presets::kick::pmod),
    };

    let mut buffs:Vec<Vec<SampleBuffer>> = Vec::new();
    let dir = Direction::Constant;
    let osc = &BaseOsc::Sine;

    // this test has one arc containing one line
    // so use the same duration for each of form/arc/line
    let mut phr = Phrasing {
        cps, 
        form: Timeframe {
            cycles: n_cycles,
            p: 0f32,
            instance: 0
        },
        arc: Timeframe {
            cycles: n_cycles,
            p: 0f32,
            instance: 0
        },
        line: Timeframe {
            cycles: n_cycles,
            p: 0f32,
            instance: 0
        },
        note: Timeframe {
            cycles: 0f32,
            p: 0f32,
            instance: 0
        }
    };
    

    let sound = Sound {
        bandpass: (FilterMode::Logarithmic, FilterPoint::Constant, (MF as f32, NF as f32)),
        energy: Energy::Medium,
        presence : Presence::Staccatto,
        pan: 0f32,
    };

    let register= 6i8;
    let melody = crate::engrave::test_tone(1i32, register, n_cycles as usize);
    let notebufs = crate::engrave::render_line(cps, &melody, &osc, &sound, &mut phr, &mbs);

    buffs.push(notebufs);

    let mixers:Vec<SampleBuffer> = buffs.into_iter().map(|buff|
        buff.into_iter().flatten().collect()
    ).collect();

    files::with_dir(out_dir);
    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => signal,
        Err(err) => {
            panic!("Problem while mixing buffers. Message: {}", err)
        }
    }
}


#[test]
fn main() {
    let cps = 2.1f32;
    let fundamental = 100f32.log2();
    let n_cycles = 64f32;

    files::with_dir(&out_dir);
    let filename = format!("{}/amb-dance.wav",out_dir);

    let f_kick = engrave::fit(64f32, fundamental);
    let f_pad = engrave::fit(200f32, fundamental);
    let a_pad = 0.25;
    let pad_a = make_pad(cps, f_pad, n_cycles, a_pad);
    let kick_a = gen_kick(cps, f_kick, n_cycles);
    let buffs = vec![
        pad_a,
        kick_a
    ];

    let sig = render::pad_and_mix_buffers(buffs).unwrap();
    let ok = render::samples_f32(SR, &sig, &filename);
    println!("Completed writing track to {}", filename);
}

#[test]
fn many() {

    let mut rng = rand::thread_rng();
    let mut prev_fundamental = 1f32;

    let harmonic_basis = 1.5f32;
    for i in 0..127 {
        let cps = 1f32 + rng.gen::<f32>() * 5f32;
        prev_fundamental = engrave::fit(1f32, prev_fundamental * harmonic_basis);
        let cycles = 3f32 + rng.gen::<f32>() * 4f32;
        let n_cycles = 2f32.powi(cycles as i32);
    
        files::with_dir(&out_dir);
        let filename = format!("{}/nother-dance-{}.wav",out_dir, i);
    
        let f_kick = engrave::fit(64f32, prev_fundamental);
        let f_pad = engrave::fit(200f32, prev_fundamental);
        let a_pad = 0.05;
        let pad_a = make_pad(cps, f_pad, n_cycles, a_pad);
        let kick_a = gen_kick(cps, f_kick, n_cycles);
        let buffs = vec![
            pad_a,
            kick_a
        ];
    
        let sig = render::pad_and_mix_buffers(buffs).unwrap();
        let mut buf = sig.clone();
        for i in  0..4 {
            buf.extend(&sig);
        }
        let ok = render::samples_f32(SR, &buf, &filename);
        println!("Completed writing track to {}", filename);
    }
   
}
