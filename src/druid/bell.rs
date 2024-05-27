use super::*;
pub type BellPartial = (f32, f32);
use rand;
use rand::Rng;


fn gen_float(min:f32, max:f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

/// Generates a soft but present sub x2 octave weight
fn gen_sub_weight() -> f32 {
    gen_float(0.005, 0.01)
}

/// Generates a soft but present sub octave weight
fn gen_bass_weight() -> f32 {
    gen_float(0.05, 0.1)
}

/// Generates a wide variety of amplitude presence weight
fn gen_fundamental_weight() -> f32 {
    gen_float(0.001, 0.01)
}

/// Generates a strike weight
fn gen_strike_weight() -> f32 {
    gen_float(0.05, 0.1)
}

/// Generates a tierce weight
fn gen_tierce_weight() -> f32 {
    gen_float(0.001, 0.01)
}

/// Generates a quint weight
fn gen_quint_weight() -> f32 {
    gen_float(0.0005, 0.002)
}

/// Generates a nominal weight
fn gen_nominal_weight() -> f32 {
    gen_float(0.00001, 0.0001)
}


fn generate_multipliers(fundamental: f32, num_multipliers: usize) -> Vec<f32> {
    let max_k = NFf/fundamental;
    (1..=num_multipliers).map(|k| {
        match k {
            1 => 1f32 / 4.0, // Sub x2 octave
            2 => 1f32 / 2.0, // Sub octave
            3 => 1f32,       // Fundamental
            4 => gen_float(1.98, 2.10), // Strike
            5 => gen_float(2.5, 2.8),   // Tierce
            6 => gen_float(3.95, 4.56), // Quint
            7 => gen_float(5.0, 12.0),  // Nominal
            _ => gen_float(9.0, max_k),
        }
    }).collect()
}


fn amp_bell(k:usize, x:f32, d:f32) -> f32 {
    let n = k - 1;
    match n {
        0 => gen_sub_weight(),
        // 1 => gen_bass_weight(),
        // 2 => gen_fundamental_weight(),
        // 3 => gen_strike_weight(),
        // 4 => gen_tierce_weight(),
        // 5 => gen_quint_weight(),
        // _ => gen_nominal_weight(), // Default case
        _ => 0f32, // Default case
    }
}

fn modders_bell() -> Modders {
    [
        Some(vec![(1f32, amp_bell)]), 
        None,
        None,
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    static cps:f32 = 1.7;
    static test_dir:&str = "dev-audio/druid";
    use crate::files;
    use crate::render::engrave;
    use crate::synth::{SR};

    fn nearly_none_bell(fund:f32) -> Element {
        Element {
            mode: Mode::Bell,
            muls: generate_multipliers(fund, 6),
            modders: modders_bell(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    #[test]
    fn test_blend_single_element_bell() {
        let test_name:&str = "bell-default";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let druid:Elementor = vec![
            (1f32, nearly_none_bell)
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &druid));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

}