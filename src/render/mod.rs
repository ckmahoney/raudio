pub mod blend;
pub mod engrave;
pub mod ifft;
pub mod ninja;
pub mod realize;

use crate::analysis::in_range;
use crate::analysis::delay::{DelayParams, StereoField};
use crate::analysis::tools::{compressor, expander, rescale_amplitude, CompressorParams, ExpanderParams};
use crate::analysis::volume::db_to_amp;
use crate::analysis::{delay, freq::apply_filter, freq::apply_resonance, freq::slice_signal, xform_freq};
use crate::presets::get_rescale_target;
use crate::druid::applied_modulation::{self, update_mods};
use crate::druid::{inflect, melody_frexer, ApplyAt, Element, Elementor};
use crate::fm::{render_operators, Operator};
use crate::monic_theory::{note_to_freq, tone_to_freq};
use crate::phrasing::contour::{apply_contour, sample, Expr, Position};
use crate::phrasing::lifespan::{self};
use crate::phrasing::ranger::{Knob, KnobMacro, KnobMods, KnobMods2, KnobbedRanger, Ranger};
use crate::presets::DB_HEADROOM;
use crate::render;
use crate::reverb::convolution::{self, ReverbParams};
use crate::synth::{pi, pi2, MFf, NFf, SRf, SampleBuffer, MF, NF, SR};
use crate::time::{self, samples_per_cycle};
use crate::types::render::{Conf, Feel, Melody, Span, Stem, Stem2, DrumSample, StemFM};
use crate::types::synthesis::{
  BoostGroup, BoostGroupMacro, Bp, Bp2, Clippers, GlideLen, MacroMotion, Modifiers, ModifiersHolder, Note, Range, Soids,
};
use crate::types::timbre::{AmpContour, AmpLifespan, Arf};
use crate::{Energy, Mode, Presence, Role, Visibility};
use rand;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;


#[derive(Clone, Debug)]
pub enum Renderable<'render> {
  Instance(Stem<'render>),
  Group(Vec<Stem<'render>>),
  Tacet(Stem<'render>),
}

#[derive(Clone, Debug)]
pub enum Renderable2<'render> {
  Instance(Stem2<'render>),
  Tacet(Stem2<'render>),
  Group(Vec<Stem2<'render>>),
  Sample(DrumSample<'render>),
  Mix(Vec<(f32, Renderable2<'render>)>),
  FMOp(StemFM<'render>),
}

#[inline]
pub fn normalize(buffer: &mut Vec<f32>) {
  if buffer.is_empty() {
    return;
  }

  let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

  if max_amplitude != 0.0 {
    buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
  }
}

pub fn norm_scale(buffer: &mut Vec<f32>, scale: f32) {
  if buffer.is_empty() {
    return;
  }

  let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

  if max_amplitude != 0.0 {
    buffer.iter_mut().for_each(|sample| *sample /= scale * max_amplitude);
  }
}

pub fn amp_scale(buffer: &mut Vec<f32>, amp: f32) {
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
  let padded_buffers = buffers
    .into_iter()
    .map(|buffer| {
      let mut padded = buffer;
      padded.resize(max_buffer_length, 0.0);
      padded
    })
    .collect();

  mix_and_normalize_buffers(padded_buffers)
}

pub fn arf(cps: f32, contour: &AmpContour, melody: &Melody<Note>, synth: &Elementor, arf: Arf) -> SampleBuffer {
  let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
  let mut channels: Vec<SampleBuffer> = Vec::with_capacity(melody.len());
  let mut seed: ThreadRng = thread_rng();

  // @art-choice: apply a visible dynamic amp mix for the synth as a whole
  let mod_amp_synth: f32 = 0.5f32 + 0.5 * seed.gen::<f32>();

  for (index, line_frexd) in melody_frexd.iter().enumerate() {
    let mut line_buff: SampleBuffer = Vec::new();
    let line = &melody[index];
    // @art-choice: apply a background dynamic amp for the melodies within as well
    let mod_amp_melody: f32 = 0.8f32 + 0.2 * seed.gen::<f32>();

    for (jindex, frex) in line_frexd.iter().enumerate() {
      let mod_amp_dither: f32 = 0.99 + 0.01 * seed.gen::<f32>();

      let dur = time::duration_to_cycles(line[jindex].0);
      let amp = line[jindex].2;
      let at = ApplyAt {
        frex: *frex,
        span: (cps, dur),
      };
      let weight_modulation: f32 = mod_amp_dither * mod_amp_melody * mod_amp_synth;
      let applied: Elementor = synth.iter().map(|(w, r)| (weight_modulation * *w * amp, *r)).collect();
      line_buff.append(&mut inflect(
        &frex,
        &at,
        &applied,
        &arf.visibility,
        &arf.energy,
        &arf.presence,
      ));
    }
    channels.push(line_buff)
  }

  match realize::mix_buffers(&mut channels) {
    Ok(mut mixdown) => {
      let cont = lifespan::mod_lifespan(
        mixdown.len() / 2,
        1f32,
        &lifespan::select_lifespan(&contour),
        1usize,
        1f32,
      );
      apply_contour(&mut mixdown, &cont);
      mixdown
    }
    Err(msg) => panic!("Error while preparing mixdown: {}", msg),
  }
}

fn longest_delay_length(ds: &Vec<DelayParams>) -> f32 {
  ds.iter().fold(0f32, |max, params| {
    (params.len_seconds * params.n_echoes as f32).max(max)
  })
}

fn longest_reverb_length(rs: &Vec<convolution::ReverbParams>) -> f32 {
  rs.iter().fold(0f32, |max, params| (params.dur).max(max))
}

fn tacet(cps: f32, (melody, soids, expr, feel, knob_mods, delays): &Stem) -> SampleBuffer {
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let signal_len = time::samples_of_cycles(cps, len_cycles);
  vec![0f32; signal_len]
}

fn tacet2(cps: f32, stem: &Stem2) -> SampleBuffer {
  let len_cycles: f32 = time::count_cycles(&stem.0[0]);
  let signal_len = time::samples_of_cycles(cps, len_cycles);
  vec![0f32; signal_len]
}

#[inline]
/// Render an audio sample for an applied polyphonic or monophonic melody.
fn channel(cps: f32, root: f32, (melody, soids, expr, feel, knob_mods, delays): &Stem) -> SampleBuffer {
  let line_buffs: Vec<SampleBuffer> = melody
    .iter()
    .map(|line| {
      let mut channel_samples: Vec<SampleBuffer> = Vec::new();

      let len_cycles: f32 = time::count_cycles(line);
      let rounding_offset = 0; // since usize rounding might cutoff some sample
      let append_delay = rounding_offset + time::samples_of_dur(1f32, longest_delay_length(&delays));

      let signal_len = time::samples_of_cycles(cps, len_cycles) + append_delay;
      let durs: Vec<f32> = line.iter().map(|(d, _, _)| time::duration_to_cycles(*d)).collect();
      let mut p: f32 = 0f32;
      line.iter().enumerate().for_each(|(i, (_, tone, amp))| {
        let freq = root * tone_to_freq(tone);
        let moment = summer(
          p, len_cycles, cps, root, *amp, freq, durs[i], soids, &expr, feel, knob_mods, &delays,
        );
        channel_samples.push(moment);
        p += durs[i] / len_cycles;
      });
      let mut mixed = overlapping(signal_len, cps, durs, &channel_samples);
      // trim_zeros(&mut mixed);
      mixed
    })
    .collect();

  match pad_and_mix_buffers(line_buffs) {
    Ok(sig) => sig,
    Err(msg) => panic!("Failed to mix and render channel: {}", msg),
  }
}

///Generate a value based on the motion type
fn generate_value(motion: MacroMotion, a: f32, b: f32, p: f32, rng: &mut ThreadRng) -> f32 {
  let min = a.min(b);
  let max = a.max(b);
  match motion {
    MacroMotion::Min => min,
    MacroMotion::Max => max,
    MacroMotion::Mean => (min + max) / 2.0,
    MacroMotion::Constant => min + (max - min) * rng.gen::<f32>(), // Random constant value
    MacroMotion::Forward => min + (max - min) * p,                 // Linear interpolation from min to max
    MacroMotion::Reverse => max - (max - min) * p,                 // Linear interpolation from max to min
    MacroMotion::Random => min + (max - min) * rng.gen::<f32>(),   // Random selection within range
  }
}

/// Create a knob value from a KnobMacro
pub fn get_knob(kmac: &KnobMacro, p: f32, rng: &mut ThreadRng) -> Knob {
  Knob {
    a: generate_value(kmac.ma, kmac.a[0], kmac.a[1], p, rng),
    b: generate_value(kmac.mb, kmac.b[0], kmac.b[1], p, rng),
    c: generate_value(kmac.mc, kmac.c[0], kmac.c[1], p, rng),
  }
}

/// Pre-compute knob values for Constant motion, or None if not needed
fn pre_compute_knobs(
  macros: &KnobMods2, rng: &mut ThreadRng,
) -> (Vec<Option<Knob>>, Vec<Option<Knob>>, Vec<Option<Knob>>) {
  let constant_knobs_amp = macros
    .0
    .iter()
    .map(|kmac| {
      if let MacroMotion::Constant = kmac.0.ma {
        Some(get_knob(&kmac.0, 0.0, rng))
      } else {
        None
      }
    })
    .collect();

  let constant_knobs_freq = macros
    .1
    .iter()
    .map(|kmac| {
      if let MacroMotion::Constant = kmac.0.mb {
        Some(get_knob(&kmac.0, 0.0, rng))
      } else {
        None
      }
    })
    .collect();

  let constant_knobs_phase = macros
    .2
    .iter()
    .map(|kmac| {
      if let MacroMotion::Constant = kmac.0.mc {
        Some(get_knob(&kmac.0, 0.0, rng))
      } else {
        None
      }
    })
    .collect();

  (constant_knobs_amp, constant_knobs_freq, constant_knobs_phase)
}

/// Get knob values based on whether they are pre-computed or need to be generated
fn get_knob_mods(
  macros: &KnobMods2, p: f32, rng: &mut ThreadRng, constant_knobs_amp: &Vec<Option<Knob>>,
  constant_knobs_freq: &Vec<Option<Knob>>, constant_knobs_phase: &Vec<Option<Knob>>,
) -> KnobMods {
  let mut amp_knobs: Vec<(Knob, Ranger)> = Vec::with_capacity(macros.0.len());
  let mut freq_knobs: Vec<(Knob, Ranger)> = Vec::with_capacity(macros.1.len());
  let mut phase_knobs: Vec<(Knob, Ranger)> = Vec::with_capacity(macros.2.len());

  for (i, a_macro) in macros.0.iter().enumerate() {
    let knob = match &constant_knobs_amp[i] {
      Some(knob) => *knob,
      None => get_knob(&a_macro.0, p, rng),
    };
    amp_knobs.push((knob, a_macro.1));
  }

  for (i, f_macro) in macros.1.iter().enumerate() {
    let knob = match &constant_knobs_freq[i] {
      Some(knob) => *knob,
      None => get_knob(&f_macro.0, p, rng),
    };
    freq_knobs.push((knob, f_macro.1));
  }

  for (i, p_macro) in macros.2.iter().enumerate() {
    let knob = match &constant_knobs_phase[i] {
      Some(knob) => *knob,
      None => get_knob(&p_macro.0, p, rng),
    };
    phase_knobs.push((knob, p_macro.1));
  }

  KnobMods(amp_knobs, freq_knobs, phase_knobs)
}

#[inline]
fn channel_with_reso(
  conf: &Conf,
  arf: &Arf,
  (melody, soids, expr, bp, knob_macros, delays1, delays2, reverbs1, reverbs2): &Stem2,
) -> SampleBuffer {
  let mut rng = thread_rng();
  let Conf { cps, root } = *conf;
  let soids = crate::analysis::trig::process_soids(soids.clone());

  // Pre-compute knobs for Constant motion
  // only evaluate moving knob macros per note-event.
  let (constant_knobs_amp, constant_knobs_freq, constant_knobs_phase) = pre_compute_knobs(knob_macros, &mut rng);

  let line_buffs: Vec<SampleBuffer> = melody
    .iter()
    .map(|line| {
      let mut channel_samples: Vec<SampleBuffer> = Vec::new();

      let len_cycles: f32 = time::count_cycles(line);
      let append_delay = time::samples_of_dur(1f32, longest_delay_length(&delays1));
      let append_reverb = time::samples_of_dur(1f32, longest_reverb_length(&reverbs1));

      let pad_samples_time_effects = append_delay + append_reverb;
      let signal_len = if pad_samples_time_effects > 0 {
        // since usize rounding might cutoff some sample
        // give it some room to render off the edge
        let pad_samples_error_margin = 2 * (delays1.len() + reverbs1.len());
        time::samples_of_cycles(cps, len_cycles) + pad_samples_time_effects + pad_samples_error_margin
      } else {
        time::samples_of_cycles(cps, len_cycles)
      };

      let durs: Vec<f32> = line.iter().map(|(d, _, _)| time::duration_to_cycles(*d)).collect();
      let mut p: f32 = 0f32;

      line.iter().enumerate().for_each(|(i, (_, tone, amp))| {
        let freq = root * tone_to_freq(tone);
        let knob_mods = get_knob_mods(
          knob_macros,
          p,
          &mut rng,
          &constant_knobs_amp,
          &constant_knobs_freq,
          &constant_knobs_phase,
        );

        // delay1 is applied per note-event in summer_with_reso
        let mut moment = summer_with_reso(
          p, len_cycles, cps, root, *amp, freq, durs[i], &soids, &expr, bp, knob_mods, &delays1,
        );

        let target_rms = get_rescale_target(&mut rng, arf.visibility);
        let expander_params = gen_inst_expander(&mut rng, &arf);
        let compressor_params = gen_inst_compressor(&mut rng, &arf);

        let normalized = rescale_amplitude(0.5f32, &moment);
        moment = compressor(
          &expander(&normalized, expander_params, None).unwrap(),
          compressor_params,
          None,
        )
        .unwrap();

        // Apply reverbs per note-event
        let moment_wet = if reverbs1.is_empty() {
          moment
        } else {
          reverbs1.iter().fold(moment, |sig, params| {
            let mut sig = convolution::of(&sig, params);
            trim_zeros(&mut sig);
            sig
          })
        };

        channel_samples.push(moment_wet);
        p += durs[i] / len_cycles;
      });

      overlapping(signal_len, cps, durs, &channel_samples)
    })
    .collect();

  match pad_and_mix_buffers(line_buffs) {
    Ok(mixed) => {
      let chan_wet_delays = if delays2.is_empty() {
        mixed
      } else {
        delays2.iter().fold(mixed, |mut sig, delay_params| {
          let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
          for replica_n in 0..=delay_params.n_echoes.max(1) {
            for j in 0..sig.len() {
              let v = sig[j];
              let offset_j = samples_per_echo * replica_n;
              let gain = delay::gain(j, replica_n, delay_params);
              let clip_thresh = 1f32;
              let gate_thresh = 0f32;

              // Apply gating and clipping
              if gain * v.abs() > clip_thresh {
                sig[j + offset_j] += gain * v.signum() * clip_thresh;
              } else if gain * v.abs() >= gate_thresh {
                sig[j + offset_j] += gain * v;
              }
            }
          }
          sig
        })
      };

      let mut chan_wet_reverbs = if reverbs2.is_empty() {
        // leave the wet delay channel as is
        chan_wet_delays
      } else {
        reverbs2.iter().fold(chan_wet_delays, |sig, params| {
          let mut sig = convolution::of(&sig, params);
          trim_zeros(&mut sig);
          sig
        })
      };
      chan_wet_reverbs
    }
    Err(msg) => panic!("Failed to mix and render channel: {}", msg),
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

fn finalize_signal(
  mut signal: SampleBuffer, delays: &[DelayParams], reverbs: &[ReverbParams], lowpass_cutoff_freq: Option<f32>,
) -> SampleBuffer {
  // Apply delays
  if !delays.is_empty() {
    signal = delays.iter().fold(signal, |mut sig, delay_params| {
      let samples_per_echo: usize = time::samples_from_dur(1.0, delay_params.len_seconds);
      for replica_n in 0..=delay_params.n_echoes.max(1) {
        for j in 0..sig.len() {
          let v = sig[j];
          let offset_j = j + samples_per_echo * replica_n;
          if offset_j < sig.len() {
            let gain = delay::gain(j, replica_n, delay_params);
            sig[offset_j] += gain * v;
          }
        }
      }
      sig
    });
  }

  // Apply reverbs
  if !reverbs.is_empty() {
    signal = reverbs.iter().fold(signal, |mut sig, params| {
      let mut sig = convolution::of(&sig, params);
      sig
    });
  }

  // Apply Butterworth filter (if cutoff frequency is specified)
  if let Some(cutoff_freq) = lowpass_cutoff_freq {
    signal = crate::analysis::freq::butterworth_lowpass_filter(&signal, SR as u32, NFf);
  }

  signal
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_finalize_signal_has_passthrough() {
    let original_signal: SampleBuffer = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let delays: Vec<DelayParams> = vec![];
    let reverbs: Vec<ReverbParams> = vec![];
    let lowpass_cutoff_freq: Option<f32> = None;

    let result = finalize_signal(original_signal.clone(), &delays, &reverbs, lowpass_cutoff_freq);
    assert_eq!(
      result, original_signal,
      "Must have a passthrough effect when no settings are applied (None)"
    );

    // Nearly equal so just comment out the test
    // let lowpass_cutoff_freq: Option<f32> = Some(NFf);
    // let result = finalize_signal(original_signal.clone(), &delays, &reverbs, lowpass_cutoff_freq);
    // assert_eq!(result, original_signal, "Must have a passthrough effect when no settings are applied (NFf)");
  }
}

/// Given a list of signals whose tails may intend to overlap with the head of the next signal
/// (e.g. long delay or release times)
/// Create a sample representing their overlapped mixing.
pub fn overlapping(base_len: usize, cps: f32, durs: Vec<f32>, samples: &Vec<SampleBuffer>) -> SampleBuffer {
  let mut signal: SampleBuffer = vec![0f32; base_len];
  durs.iter().enumerate().fold(0, |cue, (i, dur)| {
    // Make sure there's enough room for us to add reverb/delay artifacts
    if signal.len() < cue + samples[i].len() {
      let mut adds  = vec![0f32; samples[i].len()];
      eprintln!("Warning! 'overlapping' method needed to append {} samples to the input duration to meet the wet samples requirements.", adds.len());
      signal.append(&mut adds);
    }

    // apply the wet samples to the running buffer
    for (j, s) in samples[i].iter().enumerate() {
      signal[cue + j] += s
    }

    // advance the cue not by the wet samples length, but by the defacto note duration length
    cue + time::samples_of_dur(cps, *dur)
  });
  signal
}

// seems that we want DB_PER_OCTAVE and DB_DISTANCE to have a product of 48.
// (for lowpass filter)
const DB_PER_OCTAVE: f32 = 48f32; // vertical compression of energy (instantaneous  compression)
const DB_DISTANCE: f32 = 1f32; // temporal spread of energy (spread over time)

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
  curr_progress: f32,
  len_cycles: f32,
  cps: f32,
  root: f32,
  vel: f32, // call it vel for velicty (name amp is taken)
  fundamental: f32,
  n_cycles: f32,
  soids: &Soids,
  expr: &Expr,
  Feel {
    bp,
    modifiers,
    clippers,
  }: &'render Feel,
  KnobMods(knobsAmp, knobsFreq, knobsPhase): &KnobMods,
  delays: &Vec<DelayParams>,
) -> Vec<f32> {
  let headroom_factor: f32 = db_to_amp(DB_HEADROOM); // would be good to lazy::static this
  let rounding_offset: usize = 10;
  let rounding_offset: usize = 0;
  let append_delay = rounding_offset + time::samples_of_dur(1f32, longest_delay_length(delays));
  let sig_samples = time::samples_of_cycles(cps, n_cycles);
  let mut sig = vec![0f32; sig_samples + append_delay];
  let (gate_thresh, clip_thresh) = clippers;

  if n_cycles.signum() == -1f32 || vel <= *gate_thresh {
    // skip rests, fill an empty vec
    return sig;
  }
  let (modsAmp, modsFreq, modsPhase, modTime) = modifiers;

  // slice the overall bandpass filter for this note's cutoff range
  let end_p: f32 = curr_progress + (n_cycles / len_cycles);
  let bp_slice_highpass: Vec<f32> = slice_signal(&bp.0, curr_progress, end_p, sig_samples);
  let bp_slice_lowpass: Vec<f32> = slice_signal(&bp.1, curr_progress, end_p, sig_samples);

  // Use exact-length buffers to prevent index interpolation during render
  let resampled_aenv = slice_signal(&expr.0, curr_progress, end_p, sig_samples);
  let resampled_fenv = slice_signal(&expr.1, 0f32, 1f32, sig_samples);
  let resampled_penv = slice_signal(&expr.2, 0f32, 1f32, sig_samples);

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

      let t0: f32 = (j as f32) / SRf;
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
          continue;
        }
        let a1: f32 = knobsAmp.iter().fold(a0, |acc, (knob, func)| {
          acc * func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });
        let mut amp = vel * modsAmp.iter().fold(a1, |acc, ma| ma.apply(pos_cycles, acc));

        // pre-filter attenuation. if the local amp scale is below thresh, before filter/boost fx, we can't use this sample.
        if amp < *gate_thresh {
          continue;
        }

        let f0: f32 = fm * m * fundamental;
        let f1: f32 = knobsFreq.iter().fold(f0, |acc, (knob, func)| {
          acc * func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });

        let frequency = modsFreq.iter().fold(f1, |acc, mf| mf.apply(pos_cycles, acc));

        // pre-filter fast check. These are application-wide hard limits.
        if frequency > NFf || frequency < MFf {
          continue;
        }

        amp *= apply_filter(frequency, hp, lp, DB_PER_OCTAVE, DB_DISTANCE);

        let p0 = pm + frequency * pi2 * pos_cycles;
        let p1: f32 = knobsPhase.iter().fold(p0, |acc, (knob, func)| {
          acc + func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });
        let phase = modsPhase.iter().fold(p1, |acc, mp| mp.apply(pos_cycles, acc));

        v += amp * phase.sin();
      }

      for replica_n in 0..=(delay_params.n_echoes.max(1)) {
        let offset_j = samples_per_echo * replica_n;
        let gain = delay::gain(j, replica_n, delay_params);
        if gain < *gate_thresh {
          continue;
        }

        // apply gating and clipping
        if gain * v.abs() > *clip_thresh {
          sig[j + offset_j] += gain * v.signum() * (*clip_thresh);
        } else if gain * v.abs() >= *gate_thresh {
          sig[j + offset_j] += gain * v;
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

fn apply_delay(buffer: Vec<f32>, delay_params: &DelayParams) -> Vec<f32> {
  let samples_per_echo = time::samples_from_dur(1.0, delay_params.len_seconds);
  let mut output = buffer.clone();
  for replica_n in 0..=delay_params.n_echoes.max(1) {
    for j in 0..buffer.len() {
      let offset_j = j + replica_n * samples_per_echo;
      if offset_j < output.len() {
        let gain = delay::gain(j, replica_n, delay_params);
        let sample = buffer[j];
        output[offset_j] += sample * gain;
      }
    }
  }
  output
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
pub fn summer_with_reso<'render>(
  curr_progress: f32,
  len_cycles: f32,
  cps: f32,
  root: f32,
  vel: f32, // call it vel for velicty (name amp is taken)
  fundamental: f32,
  n_cycles: f32,
  soids: &Soids,
  expr: &Expr,
  bp: &'render Bp2,
  KnobMods(knobsAmp, knobsFreq, knobsPhase): KnobMods,
  line_delays: &Vec<DelayParams>,
) -> Vec<f32> {
  let headroom_factor: f32 = db_to_amp(DB_HEADROOM); // would be good to lazy::static this
  let rounding_offset: usize = 10;
  let rounding_offset: usize = 0;

  let mut ds: Vec<DelayParams> = vec![];

  // current implementation requires entries in the delays, even if no delay is desired.
  // that is a bad implementation but this is a easy work around to enable empty delay argument
  if line_delays.is_empty() {
    ds.push(delay::passthrough)
  } else {
    for d in line_delays {
      ds.push(d.clone())
    }
  };

  let delays = &ds;
  let append_delay = rounding_offset + time::samples_of_dur(1f32, longest_delay_length(delays));
  let sig_samples = time::samples_of_cycles(cps, n_cycles);
  let mut sig = vec![0f32; sig_samples + append_delay];

  if n_cycles.signum() == -1f32 || vel <= 0f32 {
    // skip rests, fill an empty vec
    return sig;
  }

  // slice the overall bandpass filter for this note's cutoff range
  let end_p: f32 = curr_progress + (n_cycles / len_cycles);
  let bp_slice_highpass: Vec<f32> = slice_signal(&bp.0, curr_progress, end_p, sig_samples);
  let bp_slice_lowpass: Vec<f32> = slice_signal(&bp.1, curr_progress, end_p, sig_samples);
  let mut boostgroups: Vec<BoostGroup> = bp.2.iter().map(|boostgroup_macro| boostgroup_macro.gen()).collect();
  let reso_slices: Vec<(Vec<f32>, Vec<f32>)> = (&boostgroups)
    .iter()
    .map(|boostgroup| {
      let boost_slice_highpass: Vec<f32> = slice_signal(&boostgroup.bp.0, curr_progress, end_p, sig_samples);
      let boost_slice_lowpass: Vec<f32> = slice_signal(&boostgroup.bp.1, curr_progress, end_p, sig_samples);
      (boost_slice_highpass, boost_slice_lowpass)
    })
    .collect();

  // Use exact-length buffers to prevent index interpolation during render
  let resampled_aenv = slice_signal(&expr.0, curr_progress, end_p, sig_samples);
  let resampled_fenv = slice_signal(&expr.1, 0f32, 1f32, sig_samples);
  let resampled_penv = slice_signal(&expr.2, 0f32, 1f32, sig_samples);

  // seems that we want DB_PER_OCTAVE and DB_DISTANCE to have a product of 48.
  // (for lowpass filter)
  const DB_PER_OCTAVE: f32 = 48f32; // vertical compression of energy (instantaneous  compression)
  const DB_DISTANCE: f32 = 1f32; // temporal spread of energy (spread over time)

  for delay_params in delays {
    // setup position and progress
    // let inner_p: f32 = j as f32 / sig_samples as f32;
    let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
    for j in 0..sig_samples {
      // sample the amp, freq, and phase offset envelopes
      let mut am = resampled_aenv[j];
      let mut fm = resampled_fenv[j];
      let mut pm = resampled_penv[j];
      let hp = bp_slice_highpass[j];
      let lp = bp_slice_lowpass[j];

      let t0: f32 = (j as f32) / SRf;
      let pos_cycles: f32 = t0;
      let mut v: f32 = 0f32;

      let amplifiers = &soids.0;
      let multipliers = &soids.1;
      let phases = &soids.2;

      if vel < 0f32 {
        continue;
      }

      // let pp = p + (inner_p * len_cycles);
      for (i, &m) in multipliers.iter().enumerate() {
        let a0 = am * amplifiers[i];

        let a1: f32 = knobsAmp.iter().fold(a0, |acc, (knob, func)| {
          acc * func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });
        let mut amp = vel * a1;

        let f0: f32 = fm * m * fundamental;
        let f1: f32 = knobsFreq.iter().fold(f0, |acc, (knob, func)| {
          acc * func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });

        let frequency = f1;

        // pre-filter fast check. These are application-wide hard limits.
        if frequency > NFf || frequency < MFf {
          continue;
        }

        for (i, (boost_min, boost_max)) in (&reso_slices).iter().enumerate() {
          let b_min = boost_min[j];
          let b_max = boost_max[j];
          // rolloff and q get applied here. rolloff is given as octaves to fade to attenuation level (gain)
          let d_octave = (frequency.log2() - b_min.log2()).max(0.0) - (b_max.log2() - frequency.log2()).max(0.0);

          let boostgroup: &BoostGroup = &boostgroups[i];
          let y = d_octave / boostgroup.rolloff;
          let gain = db_to_amp(y * -boostgroup.att.abs()) * boostgroup.q.powf(y);
          amp *= gain
        }

        let p0 = pm + frequency * pi2 * pos_cycles;
        let p1: f32 = knobsPhase.iter().fold(p0, |acc, (knob, func)| {
          acc + func(knob, cps, fundamental, m, n_cycles, pos_cycles)
        });
        let phase = p1;

        let final_freq = phase.sin();

        amp *= apply_filter(frequency, hp, lp, DB_PER_OCTAVE, DB_DISTANCE);

        v += amp * final_freq;
      }

      for replica_n in 0..=(delay_params.n_echoes.max(1)) {
        let offset_j = samples_per_echo * replica_n;
        let gain = delay::gain(j, replica_n, delay_params);
        if gain < 0f32 {
          continue;
        }

        // apply gating and clipping
        if gain * v.abs() > 1f32 {
          sig[j + offset_j] += gain * v.signum();
        } else {
          sig[j + offset_j] += gain * v;
        }
      }

      // post-gen filter:
      sig[j] *= headroom_factor;
      // apply global headroom scaling
    }
  }
  sig
}

/// Given a list of renderables (either instances or groups) and how to represent them in space,
/// Generate the signals and apply reverberation. Return the new signal.
/// Accepts an optional parameter `keep_stems`. When provided, it is the directory for placing the stems.
pub fn combiner<'render>(
  cps: f32, root: f32, renderables: &Vec<Renderable<'render>>, reverbs: &Vec<convolution::ReverbParams>,
  keep_stems: Option<&str>,
) -> SampleBuffer {
  // Collect channels by processing each renderable
  let mut channels: Vec<SampleBuffer> = renderables
    .iter()
    .enumerate()
    .map(|(j, renderable)| {
      let ch = match renderable {
        Renderable::Instance(stem) => {
          // Process a single stem
          vec![channel(cps, root, stem)]
        }
        Renderable::Group(stems) => {
          // Process each stem in the group
          stems.iter().map(|stem| channel(cps, root, stem)).collect::<Vec<_>>()
        }
        Renderable::Tacet(stem) => {
          vec![tacet(cps, stem)]
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
        Err(msg) => panic!("Unexpected error while mixing channels {}", msg),
      }
    })
    .collect();

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
    }
    Err(msg) => {
      panic!("Failed to mix and render audio: {}", msg)
    }
  }
}



pub fn gen_inst_compressor(rng: &mut ThreadRng, arf: &Arf) -> CompressorParams {
  let ratio: f32 = match arf.energy {
    Energy::High => in_range(rng, 6.0, 8.0),
    Energy::Medium => in_range(rng, 3.0, 6.0),
    Energy::Low => in_range(rng, 2.0, 3.0),
  };
  let threshold: f32 = match arf.role {
    Role::Bass => match arf.presence {
      // higher thresholds here tend to create the most open / sustained sound
      Presence::Tenuto => in_range(rng, -6.0, -9.0),
      Presence::Legato => in_range(rng, -9.0, -15.0),
      // low thresholds for producing a closed / muted sound
      Presence::Staccatto => in_range(rng, -15.0, -24.0),
    },
    Role::Chords => match arf.presence {
      // higher thresholds here tend to create the most open / sustained sound
      Presence::Tenuto => in_range(rng, -6.0, -9.0),
      Presence::Legato => in_range(rng, -12.0, -18.0),
      // low thresholds for producing a closed / muted sound
      Presence::Staccatto => in_range(rng, -21.0, -33.0),
    },
    Role::Lead => match arf.presence {
      // higher thresholds here tend to create the most open / sustained sound
      Presence::Tenuto => in_range(rng, -9.0, -12.0),
      Presence::Legato => in_range(rng, -15.0, -21.0),
      // lower thresholds for producing a closed / muted sound
      Presence::Staccatto => in_range(rng, -24.0, -36.0),
    },
    _ => panic!("Not implemented for melodic arfs"),
  };

  CompressorParams {
    ratio,
    threshold,
    attack_time: 0.1,
    release_time: 0.2,
    ..Default::default()
  }
}

pub fn gen_inst_expander(rng: &mut ThreadRng, arf: &Arf) -> ExpanderParams {

  let threshold: f32 = match arf.role {
    Role::Bass => in_range(rng, -39.0, -27.0),
    Role::Chords => in_range(rng, -24.0, -15.0),
    Role::Lead => in_range(rng, -42.0, -30.0),
    _ => panic!("Not implemented for melodic arfs"),
  };
  ExpanderParams {
    threshold,
    attack_time: 0.1,
    release_time: 0.2,
    ..Default::default()
  }
}


// #[inline]
// fn fm_channel_with_reso(
//     conf: &Conf,
//     operator: &Operator,
//     delays1: &Vec<DelayParams>,
//     delays2: &Vec<DelayParams>,
//     reverbs1: &Vec<ReverbParams>,
//     reverbs2: &Vec<ReverbParams>,
// ) -> SampleBuffer {
//     let Conf { cps, .. } = *conf;

//     let n_cycles = 1.0; // Define the length of the render in cycles
//     let signal_len = time::samples_of_cycles(cps, n_cycles);

//     // Render the FM signal for the operator
//     let mut signal = operator.render(n_cycles, cps, SR);

//     // Apply primary reverbs
//     if !reverbs1.is_empty() {
//         signal = reverbs1.iter().fold(signal, |sig, params| {
//             let mut processed = convolution::of(&sig, params);
//             trim_zeros(&mut processed);
//             processed
//         });
//     }

//     // Apply primary delays
//     if !delays1.is_empty() {
//         signal = delays1.iter().fold(signal, |mut sig, params| {
//             let delay_samples = time::samples_of_dur(1.0, params.len_seconds);
//             for replica in 0..params.n_echoes.max(1) {
//                 for j in 0..sig.len() {
//                     let offset = delay_samples * replica;
//                     if j + offset < sig.len() {
//                         sig[j + offset] += sig[j] * delay::gain(j, replica, params);
//                     }
//                 }
//             }
//             sig
//         });
//     }

//     // Apply secondary delays
//     let delayed_signal = delays2.iter().fold(signal.clone(), |mut sig, params| {
//         let delay_samples = time::samples_of_dur(1.0, params.len_seconds);
//         for replica in 0..params.n_echoes.max(1) {
//             for j in 0..sig.len() {
//                 let offset = delay_samples * replica;
//                 if j + offset < sig.len() {
//                     sig[j + offset] += sig[j] * delay::gain(j, replica, params);
//                 }
//             }
//         }
//         sig
//     });

//     // Apply secondary reverbs
//     let reverberated_signal = reverbs2.iter().fold(delayed_signal, |sig, params| {
//         let mut processed = convolution::of(&sig, params);
//         trim_zeros(&mut processed);
//         processed
//     });

//     reverberated_signal
// }

pub fn fm_combiner_with_reso<'render>(
  conf: &Conf, stem: StemFM<'render>, reverbs: &Vec<ReverbParams>, keep_stems: Option<&str>,
) -> SampleBuffer {
  let (melody, arf, fm_fn, delay1, delay2, reverb1, reverb2) = stem;

  let append_delay = time::samples_of_dur(1f32, longest_delay_length(&delay1));
  let append_reverb = time::samples_of_dur(1f32, longest_reverb_length(&reverb1));

  // Precompute signal lengths
  let line_length_cycles = time::count_cycles(&melody[0]);
  let total_signal_samples = time::samples_of_cycles(conf.cps, line_length_cycles) + append_delay + append_reverb;
  let polyphony_attenuation = 1f32 / melody.len() as f32;

  // Process each note in the melody
  let line_buffs: Vec<SampleBuffer> = melody
    .iter()
    .map(|line| {
      let mut channel_samples: Vec<SampleBuffer> = Vec::new();

      let len_cycles = time::count_cycles(line);
      let append_delay = time::samples_of_dur(1f32, longest_delay_length(&delay1));
      let append_reverb = time::samples_of_dur(1f32, longest_reverb_length(&reverb1));

      let pad_samples_time_effects = append_delay + append_reverb;
      let signal_len = if pad_samples_time_effects > 0 {
        let pad_samples_error_margin = 2 * (delay1.len() + reverb1.len());
        time::samples_of_cycles(conf.cps, len_cycles) + pad_samples_time_effects + pad_samples_error_margin
      } else {
        time::samples_of_cycles(conf.cps, len_cycles)
      };

      let durs: Vec<f32> = line.iter().map(time::note_to_cycles).collect();
      let mut curr_pos_cycles = 0f32;

      line.iter().enumerate().for_each(|(idx, note)| {
        let ((step_num, step_denom), (register, monae), amp) = *note;
        let n_cycles = durs[idx];

        let moment: Vec<f32> = if step_num.signum() < 0 || step_denom.signum() < 0 || amp == 0f32 {
          // this is a rest
          vec![0f32; time::samples_of_cycles(conf.cps, n_cycles)]
        } else {
          let freq = note_to_freq(note);
          let high_freq_attenuation: f32 = match freq.log2().floor() as i32 {
            10 => 0.8f32,
            11 => 0.5f32,
            12 => 0.4f32,
            13 => 0.4f32,
            14 => 0.6f32,
            15 => 0.6f32,
            _ => 1f32,
          };

          let velocity = amp
            * apply_filter(
              crate::monic_theory::note_to_freq(note),
              MFf,
              NFf,
              DB_PER_OCTAVE,
              DB_DISTANCE,
            );
          let gain = polyphony_attenuation * high_freq_attenuation;
          let operators = fm_fn(conf, &arf, note, conf.cps, n_cycles, curr_pos_cycles, gain * velocity);

          let mut rng = thread_rng();
          // Render the operators to a single channel
          let mut moment = render_operators(operators, n_cycles, conf.cps, SR);

          let target_rms = get_rescale_target(&mut rng, arf.visibility);
          let expander_params = gen_inst_expander(&mut rng, &arf);
          let compressor_params = gen_inst_compressor(&mut rng, &arf);

          let normalized = rescale_amplitude(0.5f32, &moment);
          moment = compressor(
            &expander(&normalized, expander_params, None).unwrap(),
            compressor_params,
            None,
          )
          .unwrap();

          // Apply per-note reverb effects
          if !reverb1.is_empty() {
            moment = reverb1.iter().fold(moment, |sig, params| {
              let mut processed = convolution::of(&sig, params);
              trim_zeros(&mut processed);
              processed
            });
          }
          moment
        };
        channel_samples.push(moment);
        curr_pos_cycles += n_cycles;
      });

      overlapping(signal_len, conf.cps, durs, &channel_samples)
    })
    .collect();

  // Pad and mix the note-level channels
  let mixed_signal = match pad_and_mix_buffers(line_buffs) {
    Ok(signal) => signal,
    Err(msg) => panic!("Failed to mix FM channels: {}", msg),
  };

  // Apply line-level delay effects
  let chan_wet_delays = if delay2.is_empty() {
    mixed_signal
  } else {
    delay2.iter().fold(mixed_signal, |mut sig, params| {
      let samples_per_echo = time::samples_from_dur(1f32, params.len_seconds);
      for replica_n in 0..=params.n_echoes.max(1) {
        for j in 0..sig.len() {
          let v = sig[j];
          let offset_j = samples_per_echo * replica_n;
          let gain = delay::gain(j, replica_n, params);
          let clip_thresh = 1f32;
          let gate_thresh = 0f32;

          // Apply gating and clipping
          if gain * v.abs() > clip_thresh {
            sig[j + offset_j] += gain * v.signum() * clip_thresh;
          } else if gain * v.abs() >= gate_thresh {
            sig[j + offset_j] += gain * v;
          }
        }
      }
      sig
    })
  };

  // Apply line-level reverb effects
  let mut final_signal = if reverb2.is_empty() {
    chan_wet_delays
  } else {
    reverb2.iter().fold(chan_wet_delays, |sig, params| {
      let mut processed = convolution::of(&sig, params);
      trim_zeros(&mut processed);
      processed
    })
  };

  // Save the final mixed signal if stems are requested
  if let Some(stem_dir) = keep_stems {
    let filename = format!("{}/final-fm-stem.wav", stem_dir);
    render::engrave::samples(SR, &final_signal, &filename);
  }

  // Apply global reverbs
  if !reverbs.is_empty() {
    final_signal = reverbs.iter().fold(final_signal, |sig, params| {
      let mut processed = convolution::of(&sig, params);
      trim_zeros(&mut processed);
      processed
    });
  }

  final_signal
}

#[cfg(test)]
mod test_fm_render {
  use crate::Visibility;

  use super::*;

  // #[test]
  // fn test_fm_channel_generation() {
  //     let conf = Conf { cps: 440.0, root: 1.0 };
  //     let operator = Operator::carrier(440.0);

  //     let signal = fm_channel_with_reso(&conf, &operator, &vec![], &vec![], &vec![], &vec![]);

  //     assert!(!signal.is_empty(), "FM channel signal should not be empty");
  // }

  #[test]
  fn test_fm_combiner_generation() {
    let conf = Conf { cps: 1.5, root: 1.23 };
    let melody: Melody<Note> = vec![vec![
      ((3, 2), (6, (1, 0, 3)), 1.0),
      ((3, 2), (6, (1, 0, 1)), 1.0),
      ((2, 2), (6, (1, 0, 5)), 1.0),
      ((3, 2), (7, (1, 0, 3)), 1.0),
      ((3, 2), (7, (1, 0, 1)), 1.0),
      ((2, 2), (7, (1, 0, 5)), 1.0),
      ((3, 2), (8, (1, 0, 3)), 1.0),
      ((3, 2), (8, (1, 0, 1)), 1.0),
      ((2, 2), (8, (1, 0, 5)), 1.0),
      ((2, 2), (9, (1, 0, 3)), 1.0),
      ((2, 2), (8, (1, 0, 5)), 1.0),
      ((2, 2), (7, (1, 0, 1)), 1.0),
      ((2, 2), (6, (1, 0, 3)), 1.0),
    ]];

    let stem_fm: StemFM = (
      &melody,
      Arf {
        mode: Mode::Melodic,
        role: Role::Lead,
        register: 7,
        visibility: Visibility::Foreground,
        energy: Energy::Medium,
        presence: Presence::Legato,
      },
      crate::presets::fum::lead::dexed_brass, // Use the correct FM function
      vec![],                                 // Delay1
      vec![],                                 // Delay2
      vec![],                                 // Reverb1
      vec![],                                 // Reverb2
    );

    // Add empty reverbs if not needed
    let reverbs = vec![];

    let signal = fm_combiner_with_reso(&conf, stem_fm, &reverbs, None);

    assert!(!signal.is_empty(), "FM combined signal should not be empty");
    let filename = format!("dev-audio/test-fm_render-stemfm");
    crate::render::engrave::samples(SR, &signal, &filename);
  }
}

/// Given a list of renderables (either instances or groups) and how to represent them in space,
/// Generate the signals and apply reverberation. Return the new signal.
/// Accepts an optional parameter `keep_stems`. When provided, it is the directory for placing the stems.
pub fn combiner_with_reso2<'render>(
  conf: &Conf, renderables: &Vec<(Arf, Renderable2<'render>)>, stem_reverbs: &Vec<convolution::ReverbParams>,
  group_reverbs: &Vec<convolution::ReverbParams>, keep_stems: Option<&str>,
) -> SampleBuffer {
  // Initialize a global Rayon thread pool with a max of 4 threads
  let _ = ThreadPoolBuilder::new().num_threads(4).build_global();

  // Collect channels by processing each renderable in parallel
  let mut channels: Vec<SampleBuffer> = renderables
    .par_iter()
    .enumerate()
    .map(|(j, (arf, renderable))| {
      let ch = match renderable {
        Renderable2::Instance(stem) => {
          // Process a single stem
          vec![channel_with_reso(conf, arf, stem)]
        }
        Renderable2::Group(stems) => {
          // Process each stem in the group
          stems.par_iter().map(|stem| channel_with_reso(conf, arf, stem)).collect::<Vec<_>>()
        }
        Renderable2::Sample(stem) => {
          vec![channel_with_samples(conf, stem)]
        }

        Renderable2::Mix(weighted_stems) => weighted_stems
          .iter()
          .map(|(gain, renderable2)| {
            combiner_with_reso2(&conf, &vec![(*arf, renderable2.to_owned())], &vec![], &vec![], keep_stems)
              .iter()
              .map(|v| gain * v)
              .collect()
          })
          .collect(),
        Renderable2::Tacet(stem) => {
          vec![tacet2(conf.cps, stem)]
        }

        Renderable2::FMOp(fm_stem) => {
          vec![fm_combiner_with_reso(conf, fm_stem.clone(), &vec![], keep_stems)]
        }
      };

      if let Some(stem_dir) = keep_stems {
        // Keep the substems
        ch.iter().enumerate().for_each(|(stem_num, channel_samples)| {
          let filename = format!("{}/part-{}-twig-{}.wav", stem_dir, j, stem_num);
          if channel_samples.is_empty() {
            eprintln!("Warning: Channel samples are empty for stem {}-{}", j, stem_num);
          }
          render::engrave::samples(SR, &channel_samples, &filename);
        });
      }

      let rendered_channel = pad_and_mix_buffers(ch);

      match rendered_channel {
        Ok(signal) => {
          if stem_reverbs.is_empty() {
            signal
          } else {
            convolution::of(&signal, &stem_reverbs[j])
          }
        }
        Err(msg) => panic!("Unexpected error while mixing channels: {}", msg),
      }
    })
    .collect();

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
      if group_reverbs.is_empty() {
        signal
      } else {
        group_reverbs.iter().fold(signal, |sig, params| {
          let mut sig = convolution::of(&sig, &params);
          trim_zeros(&mut sig);
          sig
        })
      }
    }
    Err(msg) => panic!("Failed to mix and render audio: {}", msg),
  }
}

/// Render a channel from sample-based input, applying the necessary effects
#[inline]
fn channel_with_samples(
  conf: &Conf, (melody, ref_samples, amp_expr, lowpass_cutoff_freq, delays1, delays2, reverbs1, reverbs2): &DrumSample,
) -> SampleBuffer {
  let Conf { cps, root } = *conf;

  let line_buffs: Vec<SampleBuffer> = melody
    .iter()
    .map(|line| {
      let len_cycles = time::count_cycles(line);
      let append_delay = time::samples_of_dur(1.0, longest_delay_length(delays1));
      let append_reverb = time::samples_of_dur(1.0, longest_reverb_length(reverbs1));

      let signal_len = time::samples_of_cycles(cps, len_cycles) + append_delay + append_reverb;
      let durs: Vec<f32> = line.iter().map(|(d, _, _)| time::duration_to_cycles(*d)).collect();

      let mut p: f32 = 0.0;
      let mut line_signal = vec![0.0; signal_len];
      let mut accumulated_offset: usize = 0; // Track the accumulated offset

      // Process each note in the line
      line.iter().enumerate().for_each(|(i, (_, tone, amp))| {
        let freq = root * tone_to_freq(tone);

        // Render the sample for the current note
        let moment = render_sample(
          p,
          len_cycles,
          cps,
          root,
          *amp,
          freq,
          durs[i],
          ref_samples,
          amp_expr,
          *lowpass_cutoff_freq,
        );

        // Apply effects (delays, reverbs) to the sample
        let mut wet = finalize_signal(moment, delays1, reverbs1, Some(NFf));

        // Add the processed sample to the line buffer
        add_to_buffer(&mut line_signal, wet, accumulated_offset);

        // Update the accumulated offset for the next note
        accumulated_offset += time::samples_of_dur(cps, durs[i]);

        // Update position in the line
        p += durs[i] / len_cycles;
      });

      line_signal
    })
    .collect();

  match pad_and_mix_buffers(line_buffs) {
    Ok(mixed) => finalize_signal(mixed, delays2, reverbs2, Some(*lowpass_cutoff_freq)),
    Err(msg) => panic!("Failed to render and mix line buffers: {}", msg),
  }
}

/// Render a single sample using the given parameters and reference samples
/// ## Arguments
///     `p` Position in the phrase in [0, 1] as defined by render context
///     `len_cycles` The duration of the phrase this note event lives in
///     `cps` Cycles Per Second, The sample rate scaling factor (aka playback rate or BPM)
///     `root` The fundamental frequency of the composition in [1, 2]
///     `vel` Velocity, a constant scalar for output amplitude
///     `fundamental` The fundamental frequency of the note event in [MFf, NFf]
///     `n_cycles` The length in cycles of the note event
///     `amp_expr` Note-length tuple of ADSR envelopes to apply to amplitude
///     `delays` Stack of delay effects to apply to output sample
#[inline]
fn render_sample(
  p: f32, len_cycles: f32, cps: f32, root: f32, vel: f32, fundamental: f32, n_cycles: f32, ref_samples: &SampleBuffer,
  amp_expr: &Vec<Range>, lowpass_cutoff_freq: f32,
) -> SampleBuffer {
  // Calculate the duration of the note in seconds
  let duration = n_cycles / cps;

  // Calculate the desired length of the signal in samples
  let signal_len = time::samples_of_cycles(cps, n_cycles);
  let mut signal = vec![0.0; signal_len];

  // Calculate the playback rate for pitch modulation
  let playback_rate = crate::analysis::fit(0.66f32, root);

  // Resample the amplitude envelope to match the signal length
  let end_p: f32 = p + (n_cycles / len_cycles);
  let resampled_aenv = slice_signal(amp_expr, p, end_p, signal_len);
  let headroom_factor: f32 = db_to_amp(DB_HEADROOM); // would be good to lazy::static this

  // Iterate through the output signal
  for i in 0..signal_len {
    // Calculate the corresponding index in the reference sample buffer
    let sample_index = ((i as f32 * playback_rate) as usize).min(ref_samples.len() - 1);

    // Apply the resampled amplitude envelope
    signal[i] = ref_samples[sample_index] * resampled_aenv[i] * headroom_factor;
  }

  // If the signal length is less than requested duration, pad with zeros
  if ref_samples.len() < signal_len {
    signal.resize(signal_len, 0.0);
  }

  // Apply a lowpass filter to the signal
  crate::analysis::freq::butterworth_lowpass_filter(&mut signal, SR as u32, lowpass_cutoff_freq);

  signal
}

/// Add a buffer into another, starting at a specified offset
#[inline]
fn add_to_buffer(target: &mut SampleBuffer, source: SampleBuffer, offset: usize) {
  for (i, &sample) in source.iter().enumerate() {
    if i + offset < target.len() {
      target[i + offset] += sample;
    }
  }
}

#[cfg(test)]
mod test {
  use convolution::ReverbParams;

  use super::*;
  use crate::druid::melodic;
  use crate::music::lib::{happy_birthday, x_files};
  use crate::{files, phrasing};
  static TEST_DIR: &str = "dev-audio/render";

  fn write_test_asset(signal: &SampleBuffer, test_name: &str) {
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
      vec![],
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
      vec![],
    )
  }

  fn feeling_lead() -> Feel {
    Feel {
      // expr: (lifespan::mod_lifespan(SR, 1f32, &AmpLifespan::Snap, 1, 1f32),vec![1f32],vec![0f32]),
      bp: (vec![MFf], vec![NFf]),
      modifiers: modifiers_lead(),
      clippers: (0f32, 1f32),
    }
  }

  fn feeling_chords() -> Feel {
    Feel {
      // expr: (vec![1f32],vec![1f32],vec![0f32]),
      bp: (vec![MFf], vec![NFf]),
      modifiers: modifiers_chords(),
      clippers: (0f32, 1f32),
    }
  }

  #[test]
  fn test_combine() {
    let mods_chords: ModifiersHolder = modifiers_chords();
    let mods_lead: ModifiersHolder = modifiers_lead();

    let delays_lead = vec![DelayParams {
      mix: 0.25f32,
      gain: 0.66,
      len_seconds: 0.33f32,
      n_echoes: 5,
      pan: StereoField::Mono,
    }];

    let delays_chords = vec![DelayParams {
      mix: 0f32,
      gain: 0f32,
      len_seconds: 0.15f32,
      n_echoes: 5,
      pan: StereoField::Mono,
    }];
    let mel = happy_birthday::lead_melody();
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let stems: Vec<Renderable> = vec![
      Renderable::Instance((
        &mel,
        melodic::soids_square(MFf),
        expr.clone(),
        feeling_lead(),
        KnobMods::unit(),
        delays_lead,
      )),
      Renderable::Instance((
        &mel,
        melodic::soids_sawtooth(MFf),
        expr,
        feeling_chords(),
        KnobMods::unit(),
        delays_chords,
      )),
    ];

    let reverbs: Vec<convolution::ReverbParams> = vec![ReverbParams {
      mix: 0.005,
      amp: 0.2,
      dur: 3f32,
      rate: 0.1,
    }];

    let result = combiner(happy_birthday::cps, happy_birthday::root, &stems, &reverbs, None);
    write_test_asset(&result, "combine");
  }
  use crate::types::timbre::{Distance, Echo, Enclosure, Positioning, SpaceEffects};
  use rand::seq::SliceRandom;
  use rand::thread_rng;

  fn gen_enclosure() -> Enclosure {
    let mut rng = thread_rng();

    *[Enclosure::Spring, Enclosure::Room, Enclosure::Hall, Enclosure::Vast].choose(&mut rng).unwrap()
  }

  fn gen_positioning() -> Positioning {
    let mut rng = thread_rng();

    Positioning {
      distance: *[Distance::Far, Distance::Near, Distance::Adjacent].choose(&mut rng).unwrap(),
      echo: *[Echo::Slapback, Echo::Trailing, Echo::Bouncy, Echo::None].choose(&mut rng).unwrap(),
      complexity: if rng.gen::<f32>() < 0.25 { 0f32 } else { rng.gen() },
    }
  }

  use crate::inp::arg_xform;

  #[test]
  fn test_space_effects() {
    for i in 0..3 {
      let mods_chords: ModifiersHolder = modifiers_chords();
      let mods_lead: ModifiersHolder = modifiers_lead();
      let expr = (vec![1f32], vec![1f32], vec![0f32]);

      let enclosure = gen_enclosure();
      let se_lead: SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
      let se_chords: SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
      let mel = happy_birthday::lead_melody();
      let stems: Vec<Renderable> = vec![
        Renderable::Instance((
          &mel,
          melodic::soids_square(MFf),
          expr,
          feeling_lead(),
          KnobMods::unit(),
          se_lead.delays,
        )),
        // (happy_birthday::lead_melody(), melodic::dress_sawtooth as fn(f32) -> Dressing, feeling_chords(), mods_chords, &se_chords.delays)
      ];

      let result = combiner(
        happy_birthday::cps,
        happy_birthday::root,
        &stems,
        &se_lead.reverbs,
        None,
      );
      write_test_asset(&result, &format!("combine_with_space_{}", i));
    }
  }
  #[test]
  fn test_mixing_soids() {
      use crate::analysis::trig;
      use std::time::Instant;
  
      let mods_chords: ModifiersHolder = modifiers_chords();
      let mods_lead: ModifiersHolder = modifiers_lead();
  
      let enclosure = gen_enclosure();
      let se_lead: SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
      let se_chords: SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
      let mel = happy_birthday::lead_melody();
      let soids_raw: Vec<Soids> = vec![
          melodic::soids_triangle(MFf),
          melodic::soids_sawtooth(MFf),
          melodic::soids_sawtooth(MFf)
      ];
      let expr = (vec![1f32], vec![1f32], vec![0f32]);
      let soid_processed: Soids = trig::process_soids(trig::prepare_soids_input(soids_raw.clone()));
  
      let stems_raw: Vec<Renderable> = vec![
          Renderable::Instance((&mel, trig::flatten_soids(soids_raw.clone()), expr.clone(), feeling_lead(), KnobMods::unit(), se_lead.delays.clone())),
      ];
      let stems_processed: Vec<Renderable> = vec![
          Renderable::Instance((&mel, soid_processed, expr, feeling_lead(), KnobMods::unit(), se_lead.delays)),
      ];
  
      // Record timing for the raw render
      let start_raw = Instant::now();
      let result_raw = combiner(
          happy_birthday::cps,
          happy_birthday::root,
          &stems_raw,
          &se_lead.reverbs,
          None,
      );
      let duration_raw = start_raw.elapsed();
      write_test_asset(&result_raw, &format!("test_mixing_soids_raw_soids"));
  
      // Record timing for the processed render
      let start_processed = Instant::now();
      let result_processed = combiner(
          happy_birthday::cps,
          happy_birthday::root,
          &stems_processed,
          &se_lead.reverbs,
          None,
      );
      let duration_processed = start_processed.elapsed();
      write_test_asset(&result_processed, &format!("test_mixing_soids_processed_soids"));
  
      // Print the durations to the console
      println!("Raw render took {:?}", duration_raw);
      println!("Processed render took {:?}", duration_processed);
  }
  

  #[test]
  fn test_longest_delay_length() {
    let params: Vec<DelayParams> = vec![
      DelayParams {
        len_seconds: 1f32,
        n_echoes: 5,
        gain: 1f32,
        mix: 1f32,
        pan: StereoField::Mono,
      },
      DelayParams {
        len_seconds: 3f32,
        n_echoes: 5,
        gain: 1f32,
        mix: 1f32,
        pan: StereoField::Mono,
      },
    ];

    let expected = 15f32;
    let actual = longest_delay_length(&params);
    assert_eq!(expected, actual, "Invalid longest delay time")
  }
}
