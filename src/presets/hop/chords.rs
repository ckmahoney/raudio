use hound::Sample;

use super::super::*;
use crate::types::synthesis::{BoostGroupMacro,BoostGroup,ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};
use crate::time;
use crate::analysis::melody::{ODR, ODRMacro, Levels,LevelMacro, mask_wah, mask_sighwah,find_reach};


fn amp_knob_detune(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let detune_rate = match energy {
        Energy::Low => rng.gen::<f32>()/6f32,
        Energy::Medium => rng.gen::<f32>()/3f32,
        Energy::High => rng.gen::<f32>(),
    };
    let detune_mix = match visibility {
        Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
        Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
        Visibility::Background => 0.1 + 0.1 * rng.gen::<f32>(),
        Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
    };

    return (Knob { a: detune_rate, b: detune_mix, c: 0.0 }, ranger::amod_detune);
}

fn freq_knob_tonal(v:Visibility, e:Energy, p:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let modulation_amount = match e {
        Energy::Low => 0.005f32 + 0.003 * rng.gen::<f32>(),
        Energy::Medium => 0.008f32 + 0.012f32 * rng.gen::<f32>(),
        Energy::High => 0.1f32 + 0.2f32 * rng.gen::<f32>()
    };
    (Knob { a: modulation_amount, b: 0f32, c: 0.0}, ranger::fmod_warble)
}

fn pmod_chorus(v:Visibility, e:Energy, p:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let modulation_depth:f32 = match v {
        Visibility::Hidden => 0.33f32,
        Visibility::Background => 0.5,
        Visibility::Foreground => 0.75,
        Visibility::Visible => 1f32,
    };

    let chorus_visibility:f32 = match v {
        Visibility::Hidden => 0f32,
        Visibility::Background => 0.1f32 + 0.5f32 * rng.gen::<f32>(),
        Visibility::Foreground => 0.6f32 + 0.2f32 * rng.gen::<f32>(),
        Visibility::Visible => 0.8f32 + 0.1f32 * rng.gen::<f32>(),
    };

    (Knob { a: modulation_depth, b: chorus_visibility, c: 0.0}, ranger::pmod_chorus)
}


/// Generate a phrase length filter contour for the given melody and arf.
pub fn filter_contour_linear_rise<'render>(melody:&'render Melody<Note>, arf:&Arf, n_samples:usize) -> (SampleBuffer, SampleBuffer) {
    let len_cycles:f32 = time::count_cycles(&melody[0]);

    let mut highpass_contour:SampleBuffer = vec![MFf; n_samples];
    let mut lowpass_contour:SampleBuffer = Vec::with_capacity(n_samples);

    // the default position of the lowpass filter. 
    let start_cap:f32 = 2.1f32;
    let final_cap:f32 = MAX_REGISTER as f32 - arf.register as f32 - start_cap;

    let min_f:f32 = 2f32.powf(arf.register as f32 + start_cap);
    let max_f:f32 = 2f32.powf(arf.register as f32 + start_cap + final_cap);
    let n:f32 = n_samples as f32;
    let df:f32 = (max_f - min_f).log2();

    for i in 0..n_samples {
        let x:f32 = i as f32 / n;
        lowpass_contour.push(min_f + 2f32.powf(df * x));
    }
    (highpass_contour, lowpass_contour)
}




fn dynamics(arf:&Arf, n_samples:usize, k:f32) -> SampleBuffer {
    let min_db = -30f32;
    let max_db = 0f32;
    let gain:f32 = visibility_gain(arf.visibility);

    let n = n_samples as f32;

    let mut dynamp_contour:Vec<f32> = Vec::with_capacity(n_samples);
    for i in 0..n_samples {
        let x: f32 = i as f32 / n;

        let x_adjusted = (k * x).fract();
        let triangle_wave = if x_adjusted <= 0.5 {
            2.0 * x_adjusted
        } else {
            2.0 * (1.0 - x_adjusted)
        };

        let y = db_to_amp(min_db + (max_db-min_db)*triangle_wave);

        // Calculate the lowpass frequency based on the triangle wave
        dynamp_contour.push(y);
    }

    dynamp_contour
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_cresc<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> (SampleBuffer, SampleBuffer) {
    let size = len_cycles.log2()-1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = (len_cycles/2f32) as usize * SR;

    let (highpass, lowpass):(Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
        match arf.energy {
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size), vec![NFf]),
            _ => (vec![MFf],filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size*rate_per_size))
        }
    } else {
        (vec![MFf], vec![NFf])
    };

    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
    let odr = ODR {
        onset: 60.0,
        decay: 1330.0,    
        release: 110.0,  
    };

    (highpass, lowpass)
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_wah<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> (SampleBuffer, SampleBuffer) {
    let size = len_cycles.log2()-1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = (len_cycles/2f32) as usize * SR;

    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
    let odr = ODR {
        onset: 60.0,
        decay: 1330.0,    
        release: 110.0,  
    };
    let (highpass, lowpass):(Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
        match arf.energy {
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size), vec![NFf]),
            _ => (vec![MFf], mask_wah(cps, &mel[low_index], &levels, &odr))
        }
    } else {
        (vec![MFf], vec![NFf])
    };

    (highpass, lowpass)
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_sighpad<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> Bp2 {
    let size = len_cycles.log2()-1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = (len_cycles/2f32) as usize * SR;
    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
    let level_macro:LevelMacro = LevelMacro {
        stable: [1f32, 1f32],
        peak: [2.5f32, 4f32],
        sustain: [0.4f32, 0.8f32],
    };
    let odr_macro = ODRMacro {
        onset: [1260.0, 2120f32],
        decay: [2330.0, 8500f32],    
        release: [1510.0, 2000f32],  
    };

    let boost_macro = BoostGroupMacro {
        bandpass: [300f32, 350f32],
        bandwidth: [0.25f32, 0.77f32],
        att: [8f32, 12f32],
        rolloff: [21f32, 2.3f32],
        q: [1f32, 2f32]
    };

    let (highpass, mut lowpass):(Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
        match arf.energy {
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size), vec![NFf]),
            _ => (vec![MFf], mask_sighwah(cps, &mel[low_index], &level_macro, &odr_macro))
        }
    } else {
        (vec![MFf], vec![NFf])
    };


    let resos = vec![
        boost_macro
        // BoostGroup::static_width(1350f32, 2420f32, 24f32, 2f32, 2f32),
    ];

    (highpass, lowpass, resos)
}

pub fn renderable<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf) -> Renderable2<'render> {
    // spent 30mins testing these values. need to record elsewhere
    // 8 is the optimal value for high energy because using 7 has the same appearance but costs 2x more
    // 10 is clearly different than 8 
    // 12 is clearly different than 10 
    // also noting that 8 and 9 not so different, 10 and 11 somewhat different 
    let mullet = match arf.energy {
        Energy::Low => 2f32.powi(12i32),
        Energy::Medium => 2f32.powi(10i32),
        Energy::High => 2f32.powi(8i32),
    }; 
    let len_cycles:f32 = time::count_cycles(&melody[0]);

    let soids = druidic_soids::overs_sawtooth(mullet);
    // let soids = soid_fx::amod::reece(&soids, 2);

    let bp:Bp2 = bp_sighpad(cps, melody, arf, len_cycles);

    let mut knob_mods_tonal:KnobMods = KnobMods::unit();
    knob_mods_tonal.0.push(amp_microtransient(arf.visibility, arf.energy, arf.presence));
    knob_mods_tonal.2.push(pmod_chorus(arf.visibility, arf.energy, arf.presence));
    let n_samples=(SRf*len_cycles/2f32) as usize; 

    let dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);

    let expr = (
        // dynamics(arf, n_samples, 4f32), 
        // vec![1f32], 
        dynamics,
        vec![1f32], 
        vec![0f32]
    );

    let stem_tonal = (melody, soids, expr, bp, knob_mods_tonal, vec![delay::passthrough]);
    Renderable2::Group(vec![
        stem_tonal
    ])
}
