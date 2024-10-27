pub mod blend; 
pub mod engrave;
pub mod ifft;
pub mod ninja;
pub mod realize; 

use crate::analysis::volume::db_to_amp;
use crate::presets::DB_HEADROOM;
use crate::synth::{SR, SRf, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::analysis::{delay, xform_freq, freq::slice_signal, freq::apply_filter, freq::apply_resonance};
use crate::druid::{Elementor, Element, ApplyAt, melody_frexer, inflect};
use crate::druid::applied_modulation::{self, update_mods};
use crate::monic_theory::tone_to_freq;
use crate::phrasing::lifespan::{self};
use crate::phrasing::contour::{Expr, Position, sample, apply_contour};
use crate::phrasing::ranger::{Knob, Ranger, KnobbedRanger, KnobMods};
use crate::render;
use crate::reverb::convolution;
use crate::time::{self, samples_per_cycle};
use crate::types::timbre::{AmpContour, Arf, AmpLifespan};
use crate::types::synthesis::{GlideLen, Modifiers, ModifiersHolder, Note, Range, Bp, Clippers, Soids};
use crate::types::render::{Melody,Span, Stem, Feel};
use rand;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

#[derive(Clone,Debug)]
pub enum Renderable<'render> {
    Instance(Stem<'render>),
    Group(Vec<Stem<'render>>),
}


pub fn rescale(samples: &[f32], original_freq: f32, target_freq: f32) -> Vec<f32> {
    let ratio = original_freq / target_freq;
    let new_length = (samples.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_length);

    for i in 0..new_length {
        let float_idx = i as f32 / ratio;
        let idx = float_idx as usize;
        let next_idx = if idx + 1 < samples.len() { idx + 1 } else { idx };
        
        // Linear interpolation
        let sample = if idx != next_idx {
            let fraction = float_idx.fract();
            samples[idx] * (1.0 - fraction) + samples[next_idx] * fraction
        } else {
            samples[idx]
        };

        resampled.push(sample);
    }

    resampled
}

pub fn normalize(buffer: &mut Vec<f32>) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}

pub fn norm_scale(buffer: &mut Vec<f32>, scale:f32) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= scale * max_amplitude);
    }
}


pub fn amp_scale(buffer:&mut Vec<f32>, amp: f32) {
    buffer.iter_mut().for_each(|sample| *sample *= amp)
}

pub fn mix_and_normalize_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    realize::mix_buffers(&mut buffers.clone())
}

pub fn pad_and_mix_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffers.is_empty() {
        return Ok(Vec::new());
    }

    let max_buffer_length = buffers.iter().map(|b| b.len()).max().unwrap_or(0);
    let padded_buffers = buffers.into_iter()
    .map(|buffer| {
        let mut padded = buffer;
        padded.resize(max_buffer_length, 0.0);
        padded
    })
    .collect();

    mix_and_normalize_buffers(padded_buffers)
}


pub fn arf(cps:f32, contour:&AmpContour, melody:&Melody<Note>, synth:&Elementor, arf:Arf) -> SampleBuffer {
    let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
    let mut channels:Vec<SampleBuffer> = Vec::with_capacity(melody.len());
    let mut seed:ThreadRng = thread_rng();
    
    // @art-choice: apply a visible dynamic amp mix for the synth as a whole
    let mod_amp_synth:f32 = 0.5f32 + 0.5 * seed.gen::<f32>();

    for (index, line_frexd) in melody_frexd.iter().enumerate() {
        let mut line_buff:SampleBuffer = Vec::new();
        let line = &melody[index];
        // @art-choice: apply a background dynamic amp for the melodies within as well
        let mod_amp_melody:f32 = 0.8f32 + 0.2 * seed.gen::<f32>();

        for (jindex, frex) in line_frexd.iter().enumerate() {
            let mod_amp_dither:f32 = 0.99 + 0.01 * seed.gen::<f32>();

            let dur = time::duration_to_cycles(line[jindex].0);
            let amp = line[jindex].2;
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            let weight_modulation:f32 = mod_amp_dither * mod_amp_melody * mod_amp_synth;
            let applied:Elementor = synth.iter().map(|(w, r)| (weight_modulation* *w * amp, *r)).collect();
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

    match realize::mix_buffers(&mut channels) {
        Ok(mut mixdown) => {
            let cont = lifespan::mod_lifespan(
                mixdown.len()/2, 
                1f32, 
                &lifespan::select_lifespan(&contour), 
                1usize, 
                1f32
            );
            apply_contour(&mut mixdown, &cont);
            mixdown
        },
        Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    }

}


fn longest_delay_length(ds:&Vec<delay::DelayParams>) -> f32 {
    ds.iter().fold(0f32, |max, params| (params.len_seconds * params.n_echoes as f32).max(max))
}

#[inline]
/// Render an audio sample for an applied polyphonic or monophonic melody.
fn channel(cps:f32, root:f32, (melody, soids, expr, feel, knob_mods, delays):&Stem) -> SampleBuffer {
    let line_buffs:Vec<SampleBuffer> = melody.iter().map(|line| {
        let mut channel_samples:Vec<SampleBuffer> = Vec::new();

        let len_cycles:f32 = time::count_cycles(line);
        let rounding_offset = 0; // since usize rounding might cutoff some sample
        let append_delay = rounding_offset + time::samples_of_dur(1f32, longest_delay_length(&delays));

        let signal_len = time::samples_of_cycles(cps, len_cycles) + append_delay;
        let durs:Vec<f32> = line.iter().map(|(d,_,_)| time::duration_to_cycles(*d)).collect();
        let mut p:f32 =0f32;
        line.iter().enumerate().for_each(|(i, (_, tone, amp))| {
            let freq = root * tone_to_freq(tone);
            let moment = summer(p, len_cycles, cps, root, *amp, freq, durs[i], soids, &expr, feel, knob_mods, &delays);
            channel_samples.push(moment);
            p += durs[i]/len_cycles;
        });
        let mut mixed = overlapping(signal_len, cps, durs, &channel_samples);
        // trim_zeros(&mut mixed);
        mixed
    }).collect();

    match pad_and_mix_buffers(line_buffs) {
        Ok(sig) => sig,
        Err(msg) => panic!("Failed to mix and render channel: {}", msg)
    }
}

/// Convolution and delay effects may produce a long tail of empty signal.
/// Remove it.
pub fn trim_zeros(signal: &mut Vec<f32>) {
    if let Some(last_sound) = find_last_audible_index(signal, 0.001) {
        signal.truncate(last_sound + 1);
    }
}

fn find_last_audible_index(vec: &Vec<f32>, thresh: f32) -> Option<usize> {
    for (i, &value) in vec.iter().enumerate().rev() {
        if value.abs() > thresh {
            return Some(i);
        }
    }
    None
}


/// Given a list of signals whose tails may intend to overlap with the head of the next signal 
/// (e.g. long delay or release times)
/// Create a sample representing their overlapped mixing.
pub fn overlapping(base_len:usize, cps:f32, durs:Vec<f32>, samples:&Vec<SampleBuffer>) -> SampleBuffer {
    let mut signal:SampleBuffer = vec![0f32; base_len];
    durs.iter().enumerate().fold(0, |cue, (i, dur)| { 
        // Make sure there's enough room for us to add reverb/delay artifacts
        if signal.len() < cue + samples[i].len() {
            signal.append(&mut vec![0f32; samples[i].len()]);
        } 
        
        for (j,s) in samples[i].iter().enumerate() {
            
            signal[cue + j] += s
        }
        cue + time::samples_of_dur(cps, *dur)
    });
    signal
}

/// Render a signal from contextual and decorative parameters.  
/// Returns a SampleBuffer representing the moment produced from this request.  
/// 
/// This model offers three methods of modulation:  
/// The Feel.expr tuple is the simplest, offering ADSR like envelope contours for (amp, freq, phase).  
/// The next simplest is the Feel.modifiers tuple. It is analogous to a guitar pedal: a static set of parameters that are continuously modulationg the signal.  
/// The most complex option are the rangers. These are functions of (fundamental, k, p, duration) that are applied on a per-multiplier basis. This offers the most granular control at higher  copmute cost as each function is called per-multipler per-sample.  
/// 
/// ## Arguments  
///     `p` Position in the phrase in [0, 1] as defined by render context  
///     `len_cycles` The duration of the phrase this note event lives in
///     `cps` Cycles Per Second, The sample rate scaling factor (aka playback rate or BPM)  
///     `root` The fundamental frequency of the composition  
///     `vel` Velocity, a constant scalar for output amplitude  
///     `fundamental` The fundamental frequency of the note event  
///     `n_cycles` The length in cycles of the note event  
///     `soids` The sinusoidal arguments for a Fourier series  
///     `expr` Note-length tuple of ADSR envelopes to apply to (amplitude, frequency, phase offset)  
///     `Feel` Phrase-length effects to apply to current note event  
///     `KnobMods` Modulation effects to apply to current note event  
///     `delays` Stack of delay effects to apply to output sample  
#[inline]
pub fn summer<'render>(
    curr_progress:f32,
    len_cycles:f32,
    cps:f32, 
    root: f32, 
    vel: f32,  // call it vel for velicty (name amp is taken)
    fundamental:f32,
    n_cycles:f32,
    soids:&Soids,
    expr:&Expr,
    Feel { bp,  modifiers, clippers }: &'render Feel,
    KnobMods (knobsAmp, knobsFreq, knobsPhase):&KnobMods,
    delays: &Vec::<delay::DelayParams>
) -> Vec<f32> {
    let headroom_factor:f32 = db_to_amp(DB_HEADROOM); // would be good to lazy::static this
    let rounding_offset:usize = 10;
    let rounding_offset:usize = 0;
    let append_delay = rounding_offset + time::samples_of_dur(1f32, longest_delay_length(delays));
    let sig_samples = time::samples_of_cycles(cps, n_cycles);
    let mut sig = vec![0f32;  sig_samples + append_delay];
    let (gate_thresh, clip_thresh) = clippers;

    if n_cycles.signum() == -1f32 || vel <= *gate_thresh {
        // skip rests, fill an empty vec
        return sig
    }
    let (modsAmp, modsFreq, modsPhase, modTime) = modifiers;

    // slice the overall bandpass filter for this note's cutoff range
    let end_p:f32 = curr_progress + (n_cycles/len_cycles);
    let bp_slice_highpass:Vec<f32> = slice_signal(&bp.0, curr_progress, end_p, sig_samples);
    let bp_slice_lowpass:Vec<f32> = slice_signal(&bp.1, curr_progress, end_p, sig_samples);

    // Use exact-length buffers to prevent index interpolation during render
    let resampled_aenv = slice_signal(&expr.0, curr_progress, end_p, sig_samples);
    let resampled_fenv = slice_signal(&expr.1, 0f32, 1f32, sig_samples);
    let resampled_penv = slice_signal(&expr.2, 0f32, 1f32, sig_samples);

    // seems that we want DB_PER_OCTAVE and DB_DISTANCE to have a product of 48. 
    // (for lowpass filter)
    const DB_PER_OCTAVE:f32 = 48f32; // vertical compression of energy (instantaneous  compression)
    const DB_DISTANCE:f32 = 1f32; // temporal spread of energy (spread over time)

    // render the sample with the provided effects and context
    for delay_params in delays {
        let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
        for j in 0..sig_samples {
            // setup position and progress 
            // let inner_p: f32 = j as f32 / sig_samples as f32;
            
            // sample the amp, freq, and phase offset envelopes
            let mut am = resampled_aenv[j];
            let mut fm = resampled_fenv[j];
            let mut pm = resampled_penv[j];
            let hp = bp_slice_highpass[j];
            let lp = bp_slice_lowpass[j];

            let t0:f32 = (j as f32 ) / SRf;
            let pos_cycles: f32 = modTime.iter().fold(t0, |acc, mt| mt.apply(t0, acc)); 
            let mut v: f32 = 0f32;

            let amplifiers = &soids.0;
            let multipliers = &soids.1;
            let phases = &soids.2;

            if vel < *gate_thresh {
                continue;
            }
            // let pp = p + (inner_p * len_cycles);
            for (i, &m) in multipliers.iter().enumerate() {
                let a0 = am * amplifiers[i];
                if a0 < *gate_thresh {
                    continue
                }
                let a1:f32 = knobsAmp.iter().fold(a0, |acc, (knob,func)| acc*func(knob, cps, fundamental, m, n_cycles, pos_cycles));
                let mut amp = vel * modsAmp.iter().fold(a1, |acc, ma| ma.apply(pos_cycles, acc)); 

                // pre-filter attenuation. if the local amp scale is below thresh, before filter/boost fx, we can't use this sample.
                if amp < *gate_thresh {
                    continue
                }
                
                let f0:f32 = fm * m * fundamental; 
                let f1:f32 = knobsFreq.iter().fold(f0, |acc, (knob,func)| acc*func(knob, cps, fundamental, m, n_cycles, pos_cycles));
                
                let frequency = modsFreq.iter().fold(f1, |acc, mf| mf.apply(pos_cycles, acc));

                // pre-filter fast check. These are application-wide hard limits.
                if frequency > NFf || frequency < MFf  {
                    continue
                }

                amp *= apply_filter(frequency, hp, lp, DB_PER_OCTAVE, DB_DISTANCE);
                
                let p0 = pm + frequency * pi2 * pos_cycles;
                let p1:f32 = knobsPhase.iter().fold(p0, |acc, (knob,func)| acc+func(knob, cps, fundamental, m, n_cycles, pos_cycles));
                let phase = modsPhase.iter().fold(p1, |acc, mp| mp.apply(pos_cycles, acc)); 
                
                v += amp * phase.sin();
            };

            for replica_n in 0..=(delay_params.n_echoes.max(1)) {
                let offset_j = samples_per_echo * replica_n;
                let gain =  delay::gain(j, replica_n, delay_params);
                if gain < *gate_thresh {
                    continue;
                }

                // apply gating and clipping 
                if gain * v.abs() > *clip_thresh {
                    sig[j+offset_j] += gain *  v.signum() * (*clip_thresh);
                } else if gain * v.abs() >= *gate_thresh {
                    sig[j+offset_j] += gain * v;
                } else {
                    // don't mix this too-quiet sample!
                }
            }

            // post-gen filter: 
            // apply global headroom scaling
            sig[j] *= headroom_factor;
        }
    }
    sig
}


/// Given a list of renderables (either instances or groups) and how to represent them in space,
/// Generate the signals and apply reverberation. Return the new signal.
/// Accepts an optional parameter `keep_stems`. When provided, it is the directory for placing the stems.
pub fn combiner<'render>(
    cps: f32, 
    root: f32, 
    renderables: &Vec<Renderable<'render>>, 
    reverbs: &Vec<convolution::ReverbParams>, 
    keep_stems: Option<&str>
) -> SampleBuffer {
    // Collect channels by processing each renderable
    let mut channels: Vec<SampleBuffer> = renderables.iter().enumerate().map(|(j, renderable)| {
        let ch = match renderable {
            Renderable::Instance(stem) => {
                // Process a single stem
                vec![channel(cps, root, stem)]
            },
            Renderable::Group(stems) => {
                // Process each stem in the group
                stems.iter().map(|stem| channel(cps, root, stem)).collect::<Vec<_>>()
            }
        }; 
        if let Some(stem_dir) = keep_stems {
            // keep the substems
            ch.iter().enumerate().for_each(|(stem_num, channel_samples)| {
                let filename = format!("{}/part-{}-twig-{}.wav", stem_dir, j, stem_num);
                render::engrave::samples(SR, &channel_samples, &filename);
            });
        }
        let rendered_channel = pad_and_mix_buffers(ch);

        match rendered_channel {
            Ok(signal) => signal,
            Err(msg)=> panic!("Unexpected error while mixing channels {}",msg)
        }
    }).collect();
    
    // Optionally save stems if `keep_stems` is provided
    if let Some(stem_dir) = keep_stems {
        channels.iter().enumerate().for_each(|(stem_num, channel_samples)| {
            let filename = format!("{}/stem-{}.wav", stem_dir, stem_num);
            render::engrave::samples(SR, &channel_samples, &filename);
        });
    }

    // Pad and mix the collected channels into a final signal
    match pad_and_mix_buffers(channels) {
        Ok(signal) => {
            // Apply reverbs if provided
            if reverbs.is_empty() {
                signal
            } else {
                reverbs.iter().fold(signal, |sig, params| {
                    let mut sig = convolution::of(&sig, &params);
                    trim_zeros(&mut sig);
                    sig
                })
            }
        },
        Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
        }
    }
}



#[cfg(test)]
mod test {
    use convolution::ReverbParams;

    use super::*;
    use crate::{files, phrasing};
    use crate::music::lib::{happy_birthday, x_files};
    use crate::druid::melodic;
    static TEST_DIR:&str = "dev-audio/render";

    fn write_test_asset(signal:&SampleBuffer, test_name:&str) {
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    } 

    fn modifiers_lead() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let vibrato = applied_modulation::gen_vibrato(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            vec![], 
            vec![], 
            // vec![vibrato], 
            vec![], 
            vec![]
        )
    }

    fn modifiers_chords() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let tremelo = applied_modulation::gen_tremelo(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            // vec![tremelo], 
            vec![], 
            vec![], 
            vec![], 
            vec![]
        )
    }

    fn feeling_lead() -> Feel {
        Feel {
            // expr: (lifespan::mod_lifespan(SR, 1f32, &AmpLifespan::Snap, 1, 1f32),vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            modifiers: modifiers_lead(),
            clippers: (0f32, 1f32)
        }
    }

    fn feeling_chords() -> Feel {
        Feel {
            // expr: (vec![1f32],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            modifiers: modifiers_chords(), 
            clippers: (0f32, 1f32)
        }
    }

    #[test]
    fn test_combine() {
        let mods_chords:ModifiersHolder = modifiers_chords();
        let mods_lead:ModifiersHolder = modifiers_lead();

        let delays_lead = vec![
            delay::DelayParams { mix: 0.25f32, gain: 0.66, len_seconds: 0.33f32, n_echoes: 5}
        ];
        
        let delays_chords = vec![
            delay::DelayParams { mix: 0f32, gain: 0f32, len_seconds: 0.15f32, n_echoes: 5 }
        ];
        let mel = happy_birthday::lead_melody();
        let expr = (vec![1f32],vec![1f32],vec![0f32]);
        let stems:Vec<Renderable> = vec![
            Renderable::Instance((&mel, melodic::soids_square(MFf), expr.clone(), feeling_lead(), KnobMods::unit(),  delays_lead)),
            Renderable::Instance((&mel, melodic::soids_sawtooth(MFf), expr, feeling_chords(), KnobMods::unit(),  delays_chords))
        ];

        let reverbs:Vec<convolution::ReverbParams> = vec![
            ReverbParams { mix: 0.005, amp: 0.2, dur: 3f32, rate: 0.1 }
        ];

        let result = combiner(happy_birthday::cps, happy_birthday::root, &stems, &reverbs, None);
        write_test_asset(&result, "combine");
    }
    use crate::types::timbre::{ SpaceEffects, Positioning, Distance, Echo, Enclosure};
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    fn gen_enclosure() -> Enclosure {
        let mut rng = thread_rng();
        
        *[Enclosure::Spring, Enclosure::Room, Enclosure::Hall, Enclosure::Vast].choose(&mut rng).unwrap()
    }

    fn gen_positioning() -> Positioning {
        let mut rng = thread_rng();

        Positioning {
            distance: *[Distance::Far, Distance::Near,Distance::Adjacent].choose(&mut rng).unwrap(),
            echo: *[Echo::Slapback, Echo::Trailing, Echo::Bouncy, Echo::None].choose(&mut rng).unwrap(),
            complexity: if rng.gen::<f32>() < 0.25 { 0f32 } else { rng.gen() } 
        }
    }

    use crate::inp::arg_xform;

    #[test]
    fn test_space_effects() {
        for i in 0..3 {
            let mods_chords:ModifiersHolder = modifiers_chords();
            let mods_lead:ModifiersHolder = modifiers_lead();
            let expr = (vec![1f32],vec![1f32],vec![0f32]);

            let enclosure = gen_enclosure();
            let se_lead:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
            let se_chords:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
            let mel = happy_birthday::lead_melody();
            let stems:Vec<Renderable> = vec![
                Renderable::Instance((&mel, melodic::soids_square(MFf), expr, feeling_lead(),KnobMods::unit(), se_lead.delays)),
                // (happy_birthday::lead_melody(), melodic::dress_sawtooth as fn(f32) -> Dressing, feeling_chords(), mods_chords, &se_chords.delays)
            ];

            let result = combiner(happy_birthday::cps, happy_birthday::root, &stems, &se_lead.reverbs, None);
            write_test_asset(&result, &format!("combine_with_space_{}", i));
        }
    }

    #[test]
    fn test_mixing_soids() {
        use crate::analysis::trig;

        let mods_chords:ModifiersHolder = modifiers_chords();
        let mods_lead:ModifiersHolder = modifiers_lead();

        let enclosure = gen_enclosure();
        let se_lead:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
        let se_chords:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
        let mel = happy_birthday::lead_melody();
        let soidss:Vec<Soids> = vec![
            melodic::soids_triangle(MFf),
            melodic::soids_sawtooth(MFf)
        ];
        let expr = (vec![1f32],vec![1f32],vec![0f32]);
        let soids:Soids = trig::process_soids(trig::prepare_soids_input(soidss));
        let stems:Vec<Renderable> = vec![
            Renderable::Instance((&mel, soids, expr, feeling_lead(),KnobMods::unit(), se_lead.delays)),
            // (happy_birthday::lead_melody(), melodic::dress_sawtooth as fn(f32) -> Dressing, feeling_chords(), mods_chords, &se_chords.delays)
        ];

        let result = combiner(happy_birthday::cps, happy_birthday::root, &stems, &se_lead.reverbs, None);
        write_test_asset(&result, &format!("combine_with_merged_soids"));
    }

    #[test]
    fn test_longest_delay_length() {
        let params:Vec<delay::DelayParams> = vec![
            delay::DelayParams { len_seconds: 1f32, n_echoes: 5, gain: 1f32, mix: 1f32 },
            delay::DelayParams { len_seconds: 3f32, n_echoes: 5, gain: 1f32, mix: 1f32 },
        ];

        let expected = 15f32;
        let actual = longest_delay_length(&params);
        assert_eq!(expected, actual, "Invalid longest delay time")
    }
}