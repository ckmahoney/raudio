/// Noisegens make great mixers for melodic and bell sounds with low area (noise) contours.
/// They can also be good percussion standalone percussion sound with higher area contours.
///
/// NoiseColor selection is generally decided by the instrument being created.
/// Lead instruments may use a touch of violet noise, 
/// or Bass some pink noise
/// 
/// Energy determines the quality and quantity of noise. 
/// 
/// Low -> shortened
/// Medium -> degraded
/// High -> full spectrum noise

use super::*;
use crate::types::timbre::{Energy};
const MAX_REGISTER:i32 = 15i32;

#[derive(Debug)]
pub enum NoiseColor {
    Violet,
    Blue,
    Equal,
    Pink,
    Red,
}
use rand;
use rand::seq::SliceRandom;
use rand::Rng;

fn select_random_unique(n: usize, min: usize, max: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let range: Vec<usize> = (min..max).collect();
    let mut selected: Vec<usize> = range.choose_multiple(&mut rng, n).cloned().collect();
    selected.into_iter().map(|x| x as f32).collect()
}


/// Produce a list of multipliers for the fundamental which may be higher or lower than the fundamental. 
/// For midrange and high fundamentals, a distinct sound is produced for each of the "shortened", "degraded", and "full spectrum" methods implemented below.
/// For low fundamentals, "Medum" and "High" behvae simliarly (since we can't include 18,000 multipliers per note).
fn multipliers(freq:f32, energy:&Energy) -> Vec<f32> {
    let fund = freq.floor();
    let b = fund.log2();
    
    let (n_freqs, min_freq, max_freq) = match energy {
        Energy::Low => {
            let max_n = 1000f32;
            let df = freq / 3f32; 
            // create one octave of noise centered around the fundamental
            if fund > max_n { (max_n, fund, fund * 2f32) } else { (fund, freq - df, freq + df) }
        },
        Energy::Medium => {
            // sample from all available octaves equally from the fundamental
            let max_n = 3000f32;
            let max_freq = (NFf - fund) as usize;
            // note this should compute from MAX_REGISTER but it's noise and this is good enough for tired eyes
            let n = fund * 3f32;
            if n > max_n { (max_n, fund, NFf) } else { (n, fund, NFf) }
        },
        Energy::High => {
            // sample from all available octaves equally from the fundamental
            let max_n = 10000f32;
            let max_freq = (NFf - fund) as usize;
            // note this should compute from MAX_REGISTER but it's noise and this is good enough for tired eyes
            let n = fund * 7f32;
            if n > max_n { (max_n, fund, NFf) } else { (n, fund, NFf) }
        }
    };
    let noise_components = select_random_unique(n_freqs as usize, min_freq as usize, max_freq as usize);
    noise_components.into_iter().map(|y| y / fund).collect()
}

impl NoiseColor {
    
    pub fn variants() -> Vec<NoiseColor> {
        vec![
            NoiseColor::Equal,
            NoiseColor::Pink,
            NoiseColor::Blue,
            NoiseColor::Red,
            NoiseColor::Violet,
        ]
    }

    #[inline]
    pub fn get_amp_mod(color: &NoiseColor, f:usize) -> f32 {
        match color {
            NoiseColor::Violet => (f as f32).powi(2),
            NoiseColor::
Blue => (f as f32).sqrt(),
            NoiseColor::Equal => 1.0,
            NoiseColor::Pink => 1.0 / (f as f32).sqrt(),
            NoiseColor::Red => 1.0 / (f as f32).powi(2),
        }
    }
}

fn modders_none() -> Modders {
    [
        None,
        None,
        None
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::files;
    use crate::render::engrave;
    static test_dir:&str = "dev-audio/druid";
    static cps:f32 = 1.7;

    fn test_vep() -> (Visibility, Energy, Presence) {
        let energy = Energy::Low;
        let presence = Presence::Staccatto;
        let visibility = Visibility::Visible;
        (visibility,energy,presence)
    }
    
    fn nearly_none_noise(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
        let muls = multipliers(fund, energy);
        println!("Has this many muls for energy {} {:#?}", muls.len(), energy);
        let mut rng = rand::thread_rng();
        let phss = (0..muls.len()).map(|_| rng.gen::<f32>() * pi2).collect();
        Element {
            mode: Mode::Noise,
            // test with equal power noise
            amps: vec![1f32; muls.len()],
            muls,
            phss,
            modders: modders_none(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    #[test]
    fn test_blend_noise_low() {
        let test_name:&str = "noise-low-energy";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_noise)
        ];
        let energy = Energy::Low;

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor, &vis, &en, &pre));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_blend_noise_medium() {
        let test_name:&str = "noise-medium-energy";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_noise)
        ];
        let energy = Energy::Medium;

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor, &vis, &energy, &pre));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_blend_noise_high() {
        let test_name:&str = "noise-high-energy";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_noise)
        ];
        let energy = Energy::High;

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor, &vis, &energy, &pre));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

}