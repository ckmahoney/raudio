use super::*;
use crate::analysis::in_range;

/// Adapted from "FM Theory and Applications: By Musicians for Musicians" by John Chowning and David Bristow
/// Page 166
const TL_VALUES: [u8; 100] = [
    127, 122, 118, 114, 110, 107, 104, 102, 100, 98, 
    96, 94, 92, 90, 88, 86, 85, 84, 82, 81, 
    79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 
    69, 68, 67, 66, 65, 64, 63, 62, 61, 60, 
    59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 
    49, 48, 47, 46, 45, 44, 43, 42, 41, 40, 
    39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 
    29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 
    19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 
    9,  8,  7,  6,  5,  4,  3,  2,  1,  0
];



/// Calculates the modulation index for an FM synthesis system, 
/// inspired by the Yamaha DX7. This function combines a normalized input (0–1),
/// and a Total Level (TL) mapping to emulate the DX7 behavior.
///
/// # Parameters
/// - `normalized_input`: A value in the range [0, 1] representing the user-adjusted input.
///   Typically sourced from a slider or similar UI element.
///
/// # Returns
/// - A floating-point value representing the modulation index, bounded in [0, 13).
fn calculate_modulation_index(normalized_input: f32) -> f32 {
    // Clamp the normalized input to the range [0, 1]
    let input = normalized_input.clamp(0.0, 1.0);

    // Map normalized input (0.0–1.0) to the DX7 Output range (0–99)
    let output_level = (input * 99.0).round() as usize;

    // Look up Total Level (TL) from the array
    let total_level = TL_VALUES[output_level] as f32;

    // Calculate x = (33/16) - (TL / 8)
    let x = (33.0 / 16.0) - (total_level / 8.0);

    // Calculate modulation index I = PI * 2^x, as specified on page 166
    pi * 2f32.powf(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulation_index_monotonic_increase() {
        // Ensure the modulation index increases monotonically with input
        let step = 0.05;
        let mut previous_value = 0.0;
        for i in 0..=20 {
            let input = i as f32 * step; // Input in the range [0, 1] at intervals of 0.05
            let result = calculate_modulation_index(input);
            println!("For input {:.2}, modulation index is {:.4}", input, result);
            assert!(
                result >= previous_value,
                "Modulation index did not increase for input {:.2}: got {:.4}, previous {:.4}",
                input,
                result,
                previous_value
            );
            previous_value = result;
        }
    }

    #[test]
    fn test_modulation_index_bounds() {
        // Test bounds of modulation index
        assert!((calculate_modulation_index(0.0) - 0.0).abs() < 1e-4, "Failed at input 0.0");
        assert!(
            calculate_modulation_index(1.0) < 13.0,
            "Modulation index exceeds upper bound at input 1.0"
        );
    }
}


/// Render and mix signals for multiple operators into a single output signal.
/// 
/// # Parameters
/// - `operators`: A vector of `Operator` structs to be rendered and mixed.
/// - `n_cycles`: Number of cycles to render.
/// - `cps`: Cycles per second (frequency scaling).
/// - `sample_rate`: The sample rate for rendering.
///
/// # Returns
/// - A single vector of mixed samples.
pub fn render_operators(operators: Vec<Operator>, n_cycles: f32, cps: f32, sample_rate: usize) -> Vec<f32> {
    let n_samples = crate::time::samples_of_cycles(cps, n_cycles); // Total number of samples
    let operator_signals:Vec<Vec<f32>> = operators.iter().map(|operator|  operator.render(n_cycles, cps, sample_rate)).collect();
    let max_len = operator_signals.iter().map(|x|x.len()).max().unwrap();
    let mut mixed_signal = vec![0.0; max_len]; 

    for signal in operator_signals {
        for (i, sample) in signal.iter().enumerate() {
            mixed_signal[i] += sample; 
        }
    }

    mixed_signal
}

fn dx_to_mod_index(dx_level: f32) -> f32 {
    calculate_modulation_index(dx_level / 99.0) // Normalize DX level to [0, 1]
}

fn single_modulator(op:Operator) -> Vec<ModulationSource> {
    vec![ModulationSource::Operator(op)]
}

fn dx7_simplified_5_8(base_frequency: f32) -> Vec<Operator> {
    

    // Carrier 1 (op1) and its modulator (op2)
    let op2 = Operator::modulator(base_frequency * 1.0, dx_to_mod_index(84.0)); // Ratio: 1.0, Mod Index: 84
    let op1 = Operator {
        frequency: base_frequency * 1.0, // Carrier: 1.0 Hz
        modulators: single_modulator(op2),
        ..Operator::carrier(base_frequency * 1.0)
    };

    // Carrier 2 (op3) and its modulator (op4)
    let op4 = Operator::modulator(base_frequency * 2.0, dx_to_mod_index(84.0)); // Ratio: 2.0, Mod Index: 84
    let op3 = Operator {
        frequency: base_frequency * 1.288, // Carrier: 1.288 Hz
        modulators:single_modulator(op4),
        ..Operator::carrier(base_frequency * 1.288)
    };

    // Carrier 3 (op5) and its modulator (op6)
    let op6 = Operator::modulator(base_frequency * 3.0, dx_to_mod_index(84.0)); // Ratio: 3.0, Mod Index: 84
    let op5 = Operator {
        frequency: base_frequency * 2.042, // Carrier: 2.042 Hz
        modulators: single_modulator(op6),
        ..Operator::carrier(base_frequency * 2.042)
    };

    // Return all carriers as independent patches
    vec![op1, op3, op5]
}

/// Represents Algorithm 2 in DX-7 operator configuration.
/// First carrier has a static LFO and is modulated by a feedback loop.
/// Second carrier is octave with rich modulation.
/// [2 -> 2] -> 1
/// 6 -> 5 -> 4 -> 3
fn dexed_mushstring(n_cycles: f32, cps:f32, base_frequency:f32, const_a:f32) -> Vec<Operator> {
    let mut rng = thread_rng();

    let op2_detune_cents = -2f32 * base_frequency.log2() / 15f32;
    let op3_detune_cents = 2f32 * base_frequency.log2() / 15f32;
    let op4_detune_cents = - 3f32 * base_frequency.log2() / 15f32;
    let op6_detune_cents = -2.5f32 * base_frequency.log2() / 15f32;

    let base_frequency = base_frequency / 8f32;

    // Carrier 1 (op1) and its modulator (op2)

    let knob:Knob = Knob {
        a: in_range(&mut rng, 0f32, 0.125f32),
        b: 0.75f32,
        c: 1.0f32,
    };
    let onset1 =  ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_1_4, 
        &knob, cps, 1f32, 4f32
    );
    let onset2 =  ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_1_4, 
        &Knob {a: in_range(&mut rng, 0f32, 0.0125f32), ..knob}, cps, 1f32, 4f32
    );

    let env_1 = mul_envelopes(
        onset1, 
        ranger::eval_knob_mod(
            ranger::amod_fadeout, 
            &Knob {a: in_range(&mut rng, 0.7f32, 1f32), b:1f32, c: 0f32}, cps, 1f32, 4f32
        )
    , true);

    let op2 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_burp, 
                &Knob {a: 1f32, b:1f32, c: 0f32}, cps, 1f32, 4f32
            )
        },
        modulators: vec![ModulationSource::Feedback(0.5f32)],
        ..Operator::modulator(2.02f32 + op2_detune_cents, dx_to_mod_index(50.0))
    }; 
    let op1 = Operator {
        modulators:single_modulator(op2),
        mod_gain_env_mul: Envelope::SampleBased { samples: (env_1) },
        ..Operator::carrier(const_a)
    };

    // Carrier 3 (op1) and its cascde of modulators (op2)
    let op6 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_burp, 
                &Knob {a: 1f32, b:1f32, c: 0f32}, cps, 1f32, 4f32
            )
        },
        ..Operator::modulator(op6_detune_cents + base_frequency * 4f32, dx_to_mod_index(in_range(&mut rng, 30.0, 80.0)))
    }; 
    let op5 = Operator {
        modulators:single_modulator(op6),
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_burp, 
                &Knob {a: 1f32, b:1f32, c: 0f32}, cps, 1f32, 4f32
            )
        },
        ..Operator::modulator(base_frequency * 8f32, dx_to_mod_index(in_range(&mut rng, 30.0, 80.0)))
    };
    let op4 = Operator {
        modulators:single_modulator(op5),
        mod_gain_env_mul: Envelope::SampleBased {
            samples: gen_organic_amplitude(10, ((n_cycles*SRf) as f32*cps) as usize, Visibility::Visible)
        },
        ..Operator::modulator(op4_detune_cents + base_frequency * 2f32, dx_to_mod_index(in_range(&mut rng, 70.0, 80.0)))
    };
    let onset3 = mul_envelopes(
        onset2, 
        ranger::eval_knob_mod(
            ranger::amod_fadeout, 
            &Knob {a: in_range(&mut rng, 0.7f32, 1f32), b:1f32, c: 0f32}, cps, 1f32, 4f32
        )
    , true);

    let op3 = Operator {
        modulators:single_modulator(op4),
        mod_gain_env_mul: Envelope::SampleBased {
            samples: onset3
        },
        ..Operator::carrier(op3_detune_cents + base_frequency * 2.0)
    };
    vec![op1, op3]
}

/// applies what I know about the tune knob
fn get_dexed_detune(base_frequency:f32, tune:i32) -> f32 {
    let v = tune as f32/7i32 as f32; // range of knob in dexed is -7 to 7
    v * base_frequency.log2() / pi
}



/// A rich bass synth evoking distorted bassoon. 
/// 
/// Uses Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + 6 > 5 > 4 } > 1 }
/// op2 provides the basis of the bass woodwind timbre
/// op3 offers the upper body of the woodwind timbre
/// op4 gives it the bite on attack
fn dexed_bassoon(p:f32, n_cycles: f32, cps:f32, freq:f32, mod_gain:f32) -> Vec<Operator> {
    let mut rng = thread_rng();

    let op3_detune_cents = get_dexed_detune(freq, 2);
    let op5_detune_cents = get_dexed_detune(freq, -2);
    let op6_detune_cents = get_dexed_detune(freq, 3);
    let max_fmod_mul = 2f32;
    
    let op_frequency = freq / max_fmod_mul;


    let mut op2 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_burp,
                &Knob {a: 1f32, b:1f32, c: 0f32}, cps, freq, 4f32
            )
        },
        ..Operator::modulator(op_frequency * 1.0f32, mod_gain * dx_to_mod_index(93.0))
    }; 

    let op3 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: eval_odr_level(cps, n_cycles, 
                &LevelMacro {
                    stable: [0.8, 0.9f32],
                    peak:[0.98, 1f32],
                    sustain: [0.95, 0.9f32]
                },
                &ODRMacro {
                    onset: [20.0, 30.0],
                    decay: [150.0, 200.0],
                    release: [1500.0, 2100.0],
                    mo: vec![MacroMotion::Constant],
                    md: vec![MacroMotion::Constant],
                    mr: vec![MacroMotion::Constant],
                }
            )
        },
        modulators: vec![ModulationSource::Feedback(0.5)],
        ..Operator::modulator(op_frequency * 1.0f32 + op3_detune_cents, mod_gain * dx_to_mod_index(81.0))
    }; 


    let op6 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: eval_odr_level(cps, n_cycles, 
                &LevelMacro {
                    stable: [0.1, 0.1f32],
                    peak:[0.98, 1f32],
                    sustain: [0.95, 0.995f32]
                },
                &ODRMacro {
                    onset: [20.0, 50.0],
                    decay: [500.0, 1200.0],
                    release: [30.0, 50.0],
                    mo: vec![MacroMotion::Constant],
                    md: vec![MacroMotion::Constant],
                    mr: vec![MacroMotion::Constant],
                }
            )
        },
        ..Operator::modulator(op_frequency * 2.0f32 + op6_detune_cents, dx_to_mod_index(99.0))
    }; 


    let op5 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: eval_odr_level(cps, n_cycles, 
                &LevelMacro {
                    stable: [0.3, 0.4f32],
                    peak:[0.98, 1f32],
                    sustain: [0.95, 0.995f32]
                },
                &ODRMacro {
                    onset: [10.0, 30.0],
                    decay: [1500.0, 2200.0],
                    release: [30.0, 50.0],
                    mo: vec![MacroMotion::Constant],
                    md: vec![MacroMotion::Constant],
                    mr: vec![MacroMotion::Constant],
                }
            )
        },
        modulators: single_modulator(op6),
        ..Operator::modulator(op_frequency * 2.0f32 + op5_detune_cents, dx_to_mod_index(57.0))
    }; 

    // burst of harmonics on note entry
    let op4 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: eval_odr_level(cps, n_cycles, 
                &LevelMacro {
                    stable: [1.0, 1f32],
                    peak:[1.0, 1f32],
                    sustain: [0.65, 0.75f32]
                },
                &ODRMacro {
                    onset: [20.0, 50.0],
                    decay: [1500.0, 3200.0],
                    release: [30.0, 100.0],
                    mo: vec![MacroMotion::Constant],
                    md: vec![MacroMotion::Constant],
                    mr: vec![MacroMotion::Constant],
                }
            )
        },
        modulators: single_modulator(op5),
        ..Operator::modulator(op_frequency * 1.0f32, mod_gain * dx_to_mod_index(75.0))
    }; 

    let op1 = Operator {
        modulators: vec![
            ModulationSource::Operator(op2), 
            ModulationSource::Operator(op3), 
            ModulationSource::Operator(op4)
        ],
        ..Operator::carrier(freq)
    };

    vec![op1]
}




/// Represents Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + { 6 > 5 > 4 } } > 1 }
/// mod op2 
/// mod op3 
/// mod op4 
fn dexed_brass(p:f32, n_cycles: f32, cps:f32, freq:f32, mod_gain:f32) -> Vec<Operator> {
    let mut rng = thread_rng();

    let op2_detune_cents = get_dexed_detune(freq, -6);
    let op3_detune_cents = get_dexed_detune(freq, 7);
    let op4_detune_cents = get_dexed_detune(freq, 2);
    let op5_detune_cents = get_dexed_detune(freq, -2);

    let max_fmod_mul = 2f32;
    
    let op_frequency = freq / max_fmod_mul;
    
    let mut op2 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,  
                &Knob {a: 0.25f32, b:0.5f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.25f32, b:0.15f32, c: 0f32}, cps, freq, n_cycles/2f32
            )
        },
        ..Operator::modulator(op_frequency * 2f32.powi(0) + op2_detune_cents, mod_gain * dx_to_mod_index(93.0))
    }; 

    let fadein = ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_4_16,
        &Knob {a: 0.95f32, b:0.5f32, c: 0.66f32}, cps, freq, 1f32
    );

    let op3 = Operator {
        mod_index_env_sum: Envelope::SampleBased{samples: fadein.clone()},
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.25f32, b:0.15f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: vec![ModulationSource::Feedback(0.95)],
        ..Operator::modulator(op_frequency* 2f32.powi(0) + op3_detune_cents, mod_gain * dx_to_mod_index(81.0))
    }; 
   
    let op6 = Operator {
        mod_index_env_sum: Envelope::SampleBased{samples: fadein},
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.19f32, b:0.75f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        ..Operator::modulator(op_frequency* 2f32.powi(0), mod_gain *  dx_to_mod_index(99.0))
    }; 

    let op5 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.5f32, b:0f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.5f32, b:0.85f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: single_modulator(op6),
        ..Operator::modulator(op_frequency* 2f32.powi(-1) + op5_detune_cents, mod_gain * dx_to_mod_index(57.0))
    }; 

    // burst of harmonics on note entry
    let op4 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: eval_odr_level(cps, n_cycles, 
                &LevelMacro {
                    stable: [1.0, 1f32],
                    peak:[1.0, 1f32],
                    sustain: [0.65, 0.75f32]
                },
                &ODRMacro {
                    onset: [20.0, 50.0],
                    decay: [500.0, 1200.0],
                    release: [30.0, 100.0],
                    mo: vec![MacroMotion::Constant],
                    md: vec![MacroMotion::Constant],
                    mr: vec![MacroMotion::Constant],
                }
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.7f32, b:0.75f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: single_modulator(op5),
        ..Operator::modulator(op_frequency * 2f32.powi(0) + op4_detune_cents, mod_gain * dx_to_mod_index(75.0))
    }; 
    
    let op1 = Operator {
        modulators: vec![
            ModulationSource::Operator(op2), 
            ModulationSource::Operator(op3), 
            ModulationSource::Operator(op4)
        ],
        mod_gain_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.25f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_gain_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.75f32, b:0.75f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        ..Operator::carrier(freq)
    };

    vec![op1]
}





/// Represents Algorithm 9 in DX-7 operator configuration for a synth pad
/// A9 { [2 > 2] > 1 + { 4 + 6 > 5 } > 3 }
fn dexed_pad(p:f32, n_cycles: f32, cps:f32, freq:f32, mod_gain:f32) -> Vec<Operator> {
    let mut rng = thread_rng();

    let op1_detune_cents = get_dexed_detune(freq, 7);
    let op2_detune_cents = get_dexed_detune(freq, 5);
    let op3_detune_cents = get_dexed_detune(freq, 1);
    let op6_detune_cents = get_dexed_detune(freq, 7);

    let max_fmod_mul = 17.38f32;
    
    let op_frequency = freq / max_fmod_mul;
    let op_frequency = freq/2f32;
    
    let fadein = ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_4_16,
        &Knob {a: 0.95f32, b:0.5f32, c: 0.66f32}, cps, freq, 1f32
    );

    let op2 = Operator {
        mod_index_env_sum: Envelope::SampleBased{samples: fadein.clone()},
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.77f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: vec![ModulationSource::Feedback(0.95)],
        ..Operator::modulator(op_frequency* 2f32.powi(-1) + op2_detune_cents, mod_gain * dx_to_mod_index(77.0))
    }; 


    let mut op1 = Operator {
        mod_gain_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,  
                &Knob {a: 0.99f32, b:0.95f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_gain_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.95f32, b:0.15f32, c: 0f32}, cps, freq, n_cycles/2f32
            )
        },
        modulators: single_modulator(op2),
        ..Operator::carrier(op_frequency * 2f32.powi(0) + op1_detune_cents)
    }; 

    let op6 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.95f32, b:0.5f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        ..Operator::modulator(op_frequency* 10f32 + op6_detune_cents, mod_gain *  dx_to_mod_index(86.0))
    }; 

    let op5 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.88f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: 
            Envelope::SampleBased {
                samples: ranger::eval_knob_mod(
                ranger::amod_cycle_fadein_1_4, 
                &Knob {a: 1f32, b:0.5f32, c: 1f32}, cps, 1f32, n_cycles
            )
        },
        modulators: single_modulator(op6),
        ..Operator::modulator(op_frequency* 3f32, mod_gain * dx_to_mod_index(69.0))
    }; 

    // burst of harmonics on note entry
    let op4 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.05f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples:  ranger::eval_knob_mod(
                ranger::amod_cycle_fadein_1_4, 
                &Knob {a: 0.25f32, b:0.5f32, c: 1f32}, cps, 1f32, n_cycles
            )
        },
        ..Operator::modulator(op_frequency * 17.38, mod_gain * dx_to_mod_index(75.0))
    }; 
    
    let op3 = Operator {
        mod_gain_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.75f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: vec![
            ModulationSource::Operator(op4),
            ModulationSource::Operator(op5),
        ],
        ..Operator::carrier(op_frequency* 2f32.powi(0))
    }; 
   

    vec![
        op1, 
        op3
    ]
}

fn compute_bandwidth(operator: &Operator, offset_frequency: f32, t:f32) -> (f32, f32) {
    // Calculate the effective frequency of the operator
    let f = operator.frequency + offset_frequency;

    // Calculate the base modulation index
    let mut base_mod_index = operator.modulation_index;

    // Apply envelopes for modulation index
    base_mod_index += operator.mod_index_env_sum.get_at(t, SR);
    if let Some(callback) = &operator.mod_index_sum {
        base_mod_index += callback.evaluate(t);
    }
    base_mod_index *= operator.mod_index_env_mul.get_at(t, SR);

    // Apply callbacks for modulation index
    if let Some(callback) = &operator.mod_index_mul {
        base_mod_index *= callback.evaluate(t); // Evaluate callback at t = 0.0
    }
    // Handle the case where there are no modulators
    if operator.modulators.is_empty() {
        if base_mod_index > 0.0 {
            // Positive modulation index
            return (f, 2.0 * base_mod_index * f);
        } else {
            // Zero modulation index
            return (f, 1.0);
        }
    }

    // Recursive computation for modulators
    let mut total_bandwidth = 0.0;
    for modulator in &operator.modulators {
        if let ModulationSource::Operator(mod_op) = modulator {
            // Compute the bandwidth contribution of this modulator
            let (_mod_freq, mod_bandwidth) = compute_bandwidth(&mod_op, 0.0, t);
            total_bandwidth += mod_bandwidth;
        }
    }

    // Combine the operator's own bandwidth with the total modulator bandwidth
    let operator_bandwidth = 2.0 * base_mod_index * f;

    // Return the effective center frequency and total bandwidth
    (f, total_bandwidth + operator_bandwidth)
}

fn scale_volume(operator: &Operator, gain: f32) -> Operator {
    // Clone the operator to create a new one with the updated values
    let mut updated_operator = operator.clone();

    // If the operator is a carrier (modulation_index == 0), scale the gain directly
    if updated_operator.modulation_index == 0.0 {
        updated_operator.mod_gain_env_sum = scale_envelope(&updated_operator.mod_gain_env_sum, gain);
    } else {
        // Otherwise, scale the modulation index sum envelope
        updated_operator.mod_index_env_mul = scale_envelope(&updated_operator.mod_index_env_mul, gain);
    }

    updated_operator
}

// Helper function to scale an envelope by a given gain factor
fn scale_envelope(envelope: &Envelope, gain: f32) -> Envelope {
    match envelope {
        Envelope::Parametric {
            attack,
            decay,
            sustain,
            release,
            min,
            max,
            mean,
        } => Envelope::Parametric {
            attack: *attack,
            decay: *decay,
            sustain: sustain * gain,
            release: *release,
            min: min * gain,
            max: max * gain,
            mean: mean * gain,
        },
        Envelope::SampleBased { samples } => Envelope::SampleBased {
            samples: samples.iter().map(|&value| value * gain).collect(),
        },
    }
}

/// Computes the remaining bandwidth available for modulation.
/// 
/// # Parameters
/// - `operator`: The operator whose bandwidth is being evaluated.
/// - `max_bandwidth`: The maximum allowable bandwidth, typically the Nyquist frequency.
/// - `t`: The current time parameter, used for dynamic envelope evaluation.
/// 
/// # Returns
/// The remaining bandwidth (in Hz) available for modulation.
pub fn get_remaining_bandwidth(operator: &Operator, max_bandwidth: f32, t: f32) -> f32 {
    let constrained_bandwidth = max_bandwidth.min(NFf); // Ensure bandwidth does not exceed NFf
    fn compute_total_bandwidth(operator: &Operator, t: f32) -> f32 {
        let f = operator.frequency;

        let mut base_mod_index = operator.modulation_index;
        base_mod_index += operator.mod_index_env_sum.get_at(t, SR);
        if let Some(callback) = &operator.mod_index_sum {
            base_mod_index += callback.evaluate(t);
        }
        base_mod_index *= operator.mod_index_env_mul.get_at(t, SR);
        if let Some(callback) = &operator.mod_index_mul {
            base_mod_index *= callback.evaluate(t);
        }

        let mut total_bandwidth = 2.0 * base_mod_index * f;

        for modulator in &operator.modulators {
            if let ModulationSource::Operator(mod_op) = modulator {
                total_bandwidth += compute_total_bandwidth(mod_op, t);
            }
        }

        total_bandwidth
    }

    let consumed_bandwidth = compute_total_bandwidth(operator, t);
    (constrained_bandwidth - consumed_bandwidth).max(0.0) // Ensure no negative bandwidth
}



/// Determines the maximum modulation frequency for a given remaining bandwidth and modulation index.
///
/// # Parameters
/// - `remaining_bandwidth`: The bandwidth left available for modulation (in Hz).
/// - `mod_index`: The modulation index to be used for the calculation.
/// 
/// # Returns
/// The maximum allowable modulation frequency (in Hz).
///
/// # Panics
/// Panics if `mod_index` is less than or equal to 0.
fn determine_mod_freq(remaining_bandwidth: f32, mod_index: f32) -> f32 {
    // Ensure modulation index is positive and reasonable
    if mod_index <= 0.0 {
        panic!("Modulation index must be greater than zero");
    }   

    // Calculate the maximum allowable modulation frequency
    let max_mod_freq = remaining_bandwidth / (2.0 * (mod_index + 1.0));

    // Clamp the frequency to an audible range (optional)
    max_mod_freq.clamp(20.0, 20000.0)
}

/// Determines the maximum modulation index for a given remaining bandwidth and modulation frequency.
///
/// # Parameters
/// - `remaining_bandwidth`: The bandwidth left available for modulation (in Hz).
/// - `mod_freq`: The modulation frequency to be used for the calculation.
/// 
/// # Returns
/// The maximum allowable modulation index.
/// 
/// # Panics
/// Panics if `mod_freq` is less than or equal to 0.
fn determine_mod_index(remaining_bandwidth: f32, mod_freq: f32) -> f32 {
    if mod_freq <= 0.0 {
        panic!("Modulation frequency must be greater than zero");
    }

    // Calculate the maximum allowable modulation index
    let max_mod_index = (remaining_bandwidth / (2.0 * mod_freq)) - 1.0;

    // Ensure index is non-negative
    max_mod_index.max(0.0)
}

pub fn generate_serial_modulation_chain(operator: &Operator, lowpass_filter: f32) -> Option<Operator> {
    let mut current_operator = operator.clone();
    let max_bandwidth = NFf.min(lowpass_filter);

    loop {
        let bandwidth_remaining = get_remaining_bandwidth(&current_operator, max_bandwidth, 0.0).max(0.0);
        println!("v bandwidth_remaining {}", bandwidth_remaining);
        if bandwidth_remaining <= 0.0 {
            break; // No remaining bandwidth
        }

        // Generate candidate modulators
        let candidates: Vec<Operator> = vec![
            extend_harmonic_range(&current_operator, 1.5),
            thicken_harmonic_density(&current_operator, 2),
            subtle_enhancement(&current_operator),
        ]
        .into_iter()
        .filter_map(|candidate| {
            if let Some(operator) = candidate {
                // Check if the candidate's bandwidth exceeds the remaining bandwidth
                let (_, candidate_bandwidth) = compute_bandwidth(&operator, 0.0, 0.0);
                if candidate_bandwidth <= bandwidth_remaining {
                    Some(operator) // Include valid candidate
                } else {
                    None // Discard candidate exceeding the bandwidth
                }
            } else {
                None // Discard invalid candidates
            }
        })
        .collect();

        // If no valid candidates remain, stop
        if candidates.is_empty() {
            break;
        }

        // Randomly select a valid candidate and add it as a modulator
        use rand::seq::SliceRandom;
        let mut new_operator = current_operator.clone();
        let new_modulator = candidates
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone();
        new_operator
            .modulators
            .push(ModulationSource::Operator(new_modulator));
        current_operator = new_operator;
    }

    // Return the operator if modulators were added, or None otherwise
    if current_operator.modulators.is_empty() {
        None
    } else {
        Some(current_operator)
    }
}




fn extend_harmonic_range(operator: &Operator, amount: f32) -> Option<Operator> {
    let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);

    // Determine the maximum modulation frequency for the desired extension
    let new_mod_freq = determine_mod_freq(remaining_bandwidth, 1.0) * amount;
    if new_mod_freq > 20.0 && new_mod_freq < NFf {
        let mut new_operator = operator.clone();
        let new_modulator = Operator::modulator(new_mod_freq, 1.0);
        new_operator.modulators.push(ModulationSource::Operator(new_modulator));
        return Some(new_operator);
    }
    None
}


fn thicken_harmonic_density(operator: &Operator, density_factor: usize) -> Option<Operator> {
    let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);
    let base_frequency = operator.frequency;

    let mut new_operator = operator.clone();
    let mut added = false;

    for i in 1..=density_factor {
        let new_mod_freq = determine_mod_freq(remaining_bandwidth / density_factor as f32, 0.5);
        if new_mod_freq > 20.0 && new_mod_freq < NFf {
            let new_modulator = Operator::modulator(base_frequency + new_mod_freq / i as f32, 0.5);
            new_operator.modulators.push(ModulationSource::Operator(new_modulator));
            added = true;
        }
    }

    if added {
        Some(new_operator)
    } else {
        None
    }
}


fn subtle_enhancement(operator: &Operator) -> Option<Operator> {
    let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);

    // Choose a moderate frequency and modulation index
    let mod_freq = determine_mod_freq(remaining_bandwidth, 0.8);
    if mod_freq > 20.0 && mod_freq < NFf {
        let mut new_operator = operator.clone();
        let new_modulator = Operator::modulator(mod_freq, 0.8);
        new_operator.modulators.push(ModulationSource::Operator(new_modulator));
        return Some(new_operator);
    }
    None
}



#[cfg(test)]
mod test_bandwidth {
    use super::*;

    #[test]
    fn test_compute_bandwidth_sine() {
        let operator:Operator = Operator::carrier(440.0);
        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        assert_eq!(result_bandwidth, 1f32, "For simple carriers (no modulation) must return 1 for its bandwidth");

    }

    #[test]
    fn test_compute_bandwidth_simple_modulation() {
        let modulator = Operator::modulator(10f32, 1f32);
        let operator:Operator = Operator{
            modulators: vec![
                ModulationSource::Operator(modulator.clone())
            ],
            ..Operator::carrier(440.0)
        };
        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        assert_eq!(result_bandwidth, modulator.frequency * 2f32, "For simple carriers (single modulator), the bandwidth must be twice the modulation frequency");
    }

    #[test]
    fn test_compute_bandwidth_no_modulation_index() {
        let operator: Operator = Operator {
            modulation_index: 0.0,
            ..Operator::carrier(440.0)
        };
        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        assert_eq!(result_bandwidth, 1.0, "For carriers with zero modulation index, bandwidth must be 1");
    }

    #[test]
    fn test_compute_bandwidth_nested_modulators() {
        let inner_modulator = Operator::modulator(20.0, 1.0);
        let modulator = Operator {
            modulators: vec![
                ModulationSource::Operator(inner_modulator.clone())
            ],
            ..Operator::modulator(10.0, 1.0)
        };
        let operator: Operator = Operator {
            modulators: vec![
                ModulationSource::Operator(modulator.clone())
            ],
            ..Operator::carrier(440.0)
        };

        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        let expected_bandwidth = modulator.frequency * 2.0 + inner_modulator.frequency * 2.0;
        assert_eq!(result_bandwidth, expected_bandwidth, "Bandwidth must account for nested modulation chains");
    }

    #[test]
    fn test_compute_bandwidth_multiple_modulators() {
        let modulator1 = Operator::modulator(10.0, 1.0);
        let modulator2 = Operator::modulator(15.0, 1.0);
        let operator: Operator = Operator {
            modulators: vec![
                ModulationSource::Operator(modulator1.clone()),
                ModulationSource::Operator(modulator2.clone()),
            ],
            ..Operator::carrier(440.0)
        };

        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        let expected_bandwidth = modulator1.frequency * 2.0 + modulator2.frequency * 2.0;
        assert_eq!(result_bandwidth, expected_bandwidth, "Bandwidth must account for multiple modulators");
    }

    #[test]
    fn test_compute_bandwidth_dynamic_modulation_index() {
        let operator: Operator = Operator {
            modulation_index: 1.0,
            mod_index_env_mul: Envelope::unit_mul(),
            mod_index_env_sum: Envelope::unit_sum(),
            ..Operator::carrier(440.0)
        };
        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        let expected_bandwidth = 2.0 * (1.0 * 1.0 + 1.0) * operator.frequency; // Includes unit envelopes
        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        assert_eq!(result_bandwidth, expected_bandwidth, "Bandwidth must consider dynamic envelopes for modulation index");
    }

    #[test]
    fn test_compute_bandwidth_high_modulation_index() {
        let operator: Operator = Operator {
            modulation_index: 5.0,
            ..Operator::carrier(440.0)
        };
        let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

        let expected_bandwidth = 2.0 * 5.0 * operator.frequency;
        assert_eq!(result_center, operator.frequency, "Must have the same center frequency as fundamental operator");
        assert_eq!(result_bandwidth, expected_bandwidth, "Bandwidth must scale with high modulation index");
    }

    #[test]
    fn test_generate_modulator_with_maximum_frequency() {
        let carrier = Operator {
            modulation_index: 1.0,
            frequency: 440.0,
            ..Operator::default()
        };

        let max_bandwidth = 20000.0; // Example Nyquist frequency
        let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
        let mod_index = 1.0;

        let max_mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);
        assert!(max_mod_freq <= 20000.0, "Modulation frequency must not exceed Nyquist frequency");

        let new_modulator = Operator::modulator(max_mod_freq, mod_index);
        let (new_center, new_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);

        assert!(new_bandwidth <= remaining_bandwidth, "Generated modulator must fit within remaining bandwidth");
        assert_eq!(new_center, new_modulator.frequency, "New modulator center frequency must match its defined frequency");
    }

    #[test]
    fn test_generate_modulator_with_maximum_modulation_index() {
        let carrier = Operator {
            modulation_index: 1.0,
            frequency: 440.0,
            ..Operator::default()
        };

        let max_bandwidth = 20000.0; // Example Nyquist frequency
        let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
        let mod_freq = 1000.0;

        let max_mod_index = determine_mod_index(remaining_bandwidth, mod_freq);
        assert!(max_mod_index > 0.0, "Modulation index must be positive");

        let new_modulator = Operator::modulator(mod_freq, max_mod_index);
        let (new_center, new_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);

        assert!(new_bandwidth <= remaining_bandwidth, "Generated modulator must fit within remaining bandwidth");
        assert_eq!(new_center, new_modulator.frequency, "New modulator center frequency must match its defined frequency");
    }

    #[test]
    fn test_generate_nested_modulators() {
        let inner_modulator = Operator::modulator(500.0, 1.0);
        let middle_modulator = Operator {
            modulators: vec![ModulationSource::Operator(inner_modulator.clone())],
            frequency: 200.0,
            modulation_index: 2.0,
            ..Operator::default()
        };

        let carrier = Operator {
            modulators: vec![ModulationSource::Operator(middle_modulator.clone())],
            frequency: 440.0,
            modulation_index: 1.0,
            ..Operator::default()
        };

        let max_bandwidth = 20000.0; // Example Nyquist frequency
        let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
        let mod_index = 1.5;

        let max_mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);
        assert!(max_mod_freq > 0.0 && max_mod_freq <= 20000.0, "Nested modulator frequency must fit within available range");

        let new_nested_modulator = Operator::modulator(max_mod_freq, mod_index);
        let (new_center, new_bandwidth) = compute_bandwidth(&new_nested_modulator, 0.0, 0.0);

        assert!(new_bandwidth <= remaining_bandwidth, "Generated nested modulator must fit within remaining bandwidth");
        assert_eq!(new_center, new_nested_modulator.frequency, "Nested modulator center frequency must match its defined frequency");
    }

    #[test]
    fn test_generate_modulator_with_scaling() {
        let carrier = Operator {
            modulation_index: 1.0,
            frequency: 440.0,
            ..Operator::default()
        };

        let max_bandwidth = 20000.0; // Example Nyquist frequency
        let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
        let mod_index = 2.0;
        let mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);

        let new_modulator = Operator::modulator(mod_freq, mod_index);
        let scaled_modulator = scale_volume(&new_modulator, 0.8); // Scale the modulator volume down

        let (original_center, original_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);
        let (scaled_center, scaled_bandwidth) = compute_bandwidth(&scaled_modulator, 0.0, 0.0);

        assert_eq!(original_center, scaled_center, "Scaling volume must not affect center frequency");
        assert!(scaled_bandwidth <= original_bandwidth, "Scaled modulator must have reduced bandwidth");
    }

    #[test]
    fn test_generate_serial_modulation_chain_with_filter() {
        let operator = Operator::carrier(440.0);
    
        if let Some(chain_operator) = generate_serial_modulation_chain(&operator, NFf) {
            let (_center, total_bandwidth) = compute_bandwidth(&chain_operator, 0.0, 0.0);
    
            // Ensure the bandwidth is within limits (10,000 Hz)
            assert!(
                total_bandwidth <= NFf,
                "Total bandwidth ({}) exceeds lowpass filter limit",
                total_bandwidth
            );
    
            // Ensure at least one modulator was added
            assert!(!chain_operator.modulators.is_empty(), "No modulators were added to the chain");
        } else {
            panic!("Failed to generate serial modulation chain with lowpass filter");
        }
    }

}




/// Represents Zalgorithm 1 in SYX-9 operator configuration for a synth lead
/// Z1 { { 3 > 2 + 6 > 5 > 4 } > 1, { 3 > 2 } > [7, 8, 9]  }
fn dexed_lead(p:f32, n_cycles: f32, cps:f32, freq:f32, mod_gain:f32) -> Vec<Operator> {
    let mut rng = thread_rng();

    let op7_detune_cents = get_dexed_detune(freq, 5);
    let op8_detune_cents = get_dexed_detune(freq, -4);
    let op9_detune_cents = get_dexed_detune(freq, 7);

    let op_frequency = freq/2f32;
    
    let fadein = ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_4_16,
        &Knob {a: 0.95f32, b:0.5f32, c: 0.66f32}, cps, freq, 1f32
    );

    let op2 = Operator {
        mod_index_env_sum: Envelope::SampleBased{samples: fadein.clone()},
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.77f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: vec![ModulationSource::Feedback(0.95)],
        ..Operator::modulator(op_frequency* 2f32.powi(-1), mod_gain * dx_to_mod_index(77.0))
    }; 


    let mut op1 = Operator {
        mod_gain_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,  
                &Knob {a: 0.99f32, b:0.95f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_gain_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.95f32, b:0.15f32, c: 0f32}, cps, freq, n_cycles/2f32
            )
        },
        modulators: single_modulator(op2),
        ..Operator::carrier(op_frequency * 2f32.powi(0) )
    }; 

    let op6 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.95f32, b:0.5f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        ..Operator::modulator(op_frequency* 10f32 , mod_gain *  dx_to_mod_index(86.0))
    }; 

    let op5 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.88f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: 
            Envelope::SampleBased {
                samples: ranger::eval_knob_mod(
                ranger::amod_cycle_fadein_1_4, 
                &Knob {a: 1f32, b:0.5f32, c: 1f32}, cps, 1f32, n_cycles
            )
        },
        modulators: single_modulator(op6),
        ..Operator::modulator(op_frequency* 3f32, mod_gain * dx_to_mod_index(69.0))
    }; 

    // burst of harmonics on note entry
    let op4 = Operator {
        mod_index_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.85f32, b:0.05f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        mod_index_env_mul: Envelope::SampleBased {
            samples:  ranger::eval_knob_mod(
                ranger::amod_cycle_fadein_1_4, 
                &Knob {a: 0.25f32, b:0.5f32, c: 1f32}, cps, 1f32, n_cycles
            )
        },
        ..Operator::modulator(op_frequency * 17.38, mod_gain * dx_to_mod_index(75.0))
    }; 
    
    let op3 = Operator {
        mod_gain_env_sum: Envelope::SampleBased {
            samples: ranger::eval_knob_mod(
                ranger::amod_unit,
                &Knob {a: 0.75f32, b:0.2f32, c: 0f32}, cps, freq, n_cycles
            )
        },
        modulators: vec![
            ModulationSource::Operator(op4),
            ModulationSource::Operator(op5),
        ],
        ..Operator::carrier(op_frequency* 2f32.powi(0))
    }; 
   

    vec![
        op1, 
        op3
    ]
}


#[test]
fn test_dx_string() {
    let freqs: Vec<f32> = (0..12).map(|i| 330f32 + 330f32 * 2f32.powf(i as f32/12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c:f32 = 1.62181;
    let n_cycles =  3f32;
    let cps:f32 =1.5f32;
    for carrier_frequency in &freqs {
        let modulator_playback_rate = 1.0;

        let operators = dexed_mushstring(n_cycles, cps, *carrier_frequency, c);
        let mut signal = render_operators(operators,n_cycles, cps, SR);

        assert!(!signal.is_empty());
        melody.extend(signal)
    }

    let filename = format!(
    "dev-audio/test-dx-strings"
    );
    engrave::samples(SR, &melody, &filename);
}


#[test]
fn test_dx_bassoon() {
    let label = "dx-bass";
    let start_freq = 333f32/8f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32/12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c:f32 = 1.62181;
    let n_cycles =  3f32;
    let cps:f32 =1.5f32;

    let l = freqs.len() as f32;
    for (i, carrier_frequency) in freqs.iter().enumerate() {
        let p = i as f32 * l/ l;
        let modulator_playback_rate = 1.0;
        let operators = dexed_bassoon(p, n_cycles, cps, *carrier_frequency, 0.15f32);
        let mut signal = render_operators(operators,n_cycles, cps, SR);

        assert!(!signal.is_empty());
        melody.extend(signal)
    }

    let filename = format!(
    "dev-audio/{}", label
    );
    engrave::samples(SR, &melody, &filename);
}

#[test]
fn test_dx_brass() {
    let label = "dx-brass";
    let start_freq = 333f32/1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32/12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c:f32 = 1.62181;
    let n_cycles =  3f32;
    let cps:f32 =1.5f32;

    let l = freqs.len() as f32;
    for (i, carrier_frequency) in freqs.iter().enumerate() {
        let p = i as f32 * l/ l;
        let modulator_playback_rate = 1.0;
        let operators = dexed_brass(p, n_cycles, cps, *carrier_frequency, 1f32);
        let mut signal = render_operators(operators,n_cycles, cps, SR);

        assert!(!signal.is_empty());
        melody.extend(signal)
    }

    let filename = format!(
    "dev-audio/{}", label
    );
    engrave::samples(SR, &melody, &filename);
}



#[test]
fn test_dx_pad() {
    let label = "dx-pad";
    let start_freq = 333f32/1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32/12f32)).collect();
    let c:f32 = 1.62181;
    let n_cycles =  12f32;
    let cps:f32 =1.5f32;

    let l = freqs.len() as f32;

    for mod_i in (2..=10).step_by(2) {
        let mod_gain = mod_i as f32 / 10f32;
        let mut melody: Vec<f32> = vec![];

        for (i, carrier_frequency) in freqs.iter().enumerate() {
            let p = i as f32 * l/ l;
            let modulator_playback_rate = 1.0;
            let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency, mod_gain);
            let mut chords_signal = render_operators(operators,n_cycles, cps, SR);

            for mul in vec![1.2f32, 1.5f32] {
                let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency * mul, mod_gain);
                let mut add_signal = render_operators(operators,n_cycles, cps, SR);
                for (ii,y) in add_signal.iter().enumerate() {
                    chords_signal[ii] += y;
                }
            }
            chords_signal.iter_mut().for_each(|v| *v /= 3f32);
            melody.extend(chords_signal)
        }

        let filename = format!(
        "dev-audio/{}-mod-gain-{}", label,mod_gain
        );
        engrave::samples(SR, &melody, &filename);
    }
}


#[test]
fn test_dx_lead() {
    let label = "dx-lead";
    let start_freq = 333f32/1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32/12f32)).collect();
    let n_cycles =  12f32;
    let cps:f32 =1.5f32;

    let l = freqs.len() as f32;

    for mod_i in (2..=10).step_by(2) {
        let mod_gain = mod_i as f32 / 10f32;
        let mut melody: Vec<f32> = vec![];

        for (i, carrier_frequency) in freqs.iter().enumerate() {
            let p = i as f32 * l/ l;
            let modulator_playback_rate = 1.0;
            let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency, mod_gain);
            let mut chords_signal = render_operators(operators,n_cycles, cps, SR);

            for mul in vec![1.2f32, 1.5f32] {
                let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency * mul, mod_gain);
                let mut add_signal = render_operators(operators,n_cycles, cps, SR);
                for (ii,y) in add_signal.iter().enumerate() {
                    chords_signal[ii] += y;
                }
            }
            chords_signal.iter_mut().for_each(|v| *v /= 3f32);
            melody.extend(chords_signal)
        }

        let filename = format!(
        "dev-audio/{}-mod-gain-{}", label,mod_gain
        );
        engrave::samples(SR, &melody, &filename);
    }
}




#[test]
fn test_dx_clone() {
    let freqs: Vec<f32> = (1..11).map(|i| i as f32 * 110f32).collect();
    let mut melody: Vec<f32> = vec![];
    for carrier_frequency in &freqs {
        let modulator_playback_rate = 1.0;

        let operators = dx7_simplified_5_8(*carrier_frequency);
        let mut signal = render_operators(operators, 12f32, 1.5, SR);

        assert!(!signal.is_empty());
        melody.extend(signal)
    }

    let filename = format!(
    "dev-audio/test-strings-clone"
    );
    engrave::samples(SR, &melody, &filename);
}


/// Creates a carrier and attaches multiple random modulators with envelopes.
fn random_carrier_with_modulators(base_freq: f32, cps: f32, n_cycles: f32, num_modulators: usize, depth: usize) -> Operator {
    let modulators: Vec<ModulationSource> = (0..num_modulators)
        .map(|_| ModulationSource::Operator(random_modulator_with_envelope(cps, n_cycles, base_freq, depth)))
        .collect();

    let mut carrier = Operator::carrier(base_freq);
    carrier.modulators.extend(modulators);

    carrier
}

/// Renders all carriers with dynamic modulation applied via their envelopes.
pub fn render_operators_with_envelopes(operators: Vec<Operator>, n_cycles: f32, cps: f32, sample_rate: usize) -> Vec<f32> {
    let mut mixed_signal = vec![];
    for operator in operators {
        let signal = operator.render(n_cycles, cps, sample_rate);
        mixed_signal.extend(signal);
    }
    mixed_signal
}

#[test]
fn animated_fm_synthesis_demo() {
    let base_freqs: Vec<f32> = vec![110.0, 220.0, 330.0]; // Example base frequencies
    let cps = 1.0; // Cycles per second
    let n_cycles = 4.0 * cps;

    let mut final_signal = vec![];

    // Create carriers and render them with their modulators
    for base_freq in base_freqs {
        let carrier = random_carrier_with_modulators(base_freq, cps, n_cycles, 3, 2); // 3 modulators, depth 2
        let rendered_signal = carrier.render(n_cycles, cps, SR);
        final_signal.extend(rendered_signal);
    }

    // Save the rendered signal to a WAV file
    engrave::samples(SR, &final_signal, "animated_fm_synthesis_demo.wav");
}
