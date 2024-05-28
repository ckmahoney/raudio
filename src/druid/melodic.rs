use super::*;

fn muls_sawtooth(freq:f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..n).map(|x| x as f32).collect()
}

/// Produce the multipliers for a Fourier series square wave starting at `freq`
fn muls_square(freq:f32) -> Vec<f32> {
    muls_triangle(freq) // they are both odd k series
}

/// Produce the multipliers for a Fourier series triangle wave starting at `freq`
fn muls_triangle(freq:f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..n).filter(|i| i % 2 == 1).map(|x| x as f32).collect()
}

static c_square:f32 = 4f32/pi;
/// Expects to be applied in the context of odd k 1,3,5... but is actually given an index. Makes an internal adjustment
fn amp_square(k:usize, x:f32, d:f32) -> f32 {
    let n = (k as f32) * 2f32 - 1f32;
    c_square/n
}

static c_triangle:f32 = 8f32/(pi*pi);
/// Expects to be applied in the context of odd k 1,3,5... but is actually given an index. Makes an internal adjustment
/// That is; no need to return 0 as only valued k should be given.
fn amp_triangle(k:usize, x:f32, d:f32) -> f32 {
    let n = (k as f32) * 2f32 - 1f32;
    let sign = (-1f32).powf((n-1f32)/2f32);
    sign * c_triangle/(n * n)
}

static c_sawtooth:f32 = 2f32/pi;
/// Expects to be applied in the context of all k 1,2,3,4...
fn amp_sawtooth(k:usize, x:f32, d:f32) -> f32 {
    let sign = (-1f32).powf(k as f32+1f32);
    sign * c_sawtooth/k as f32
}

/// Provides amplitude modulation to create a square wave (expecting odd-valued multipliers
pub fn modders_square() -> Modders {
    [
        Some(vec![(1f32, amp_square)]),
        None,
        None
    ]
}

/// Provides amplitude modulation to create a sawtooth wave (expecting odd-valued multipliers)
pub fn modders_sawtooth() -> Modders {
    [
        Some(vec![(1f32, amp_sawtooth)]),
        None,
        None
    ]
}

/// Provides amplitude modulation to create a sawtooth wave (expecting odd-valued multipliers)
pub fn modders_triangle() -> Modders {
    [
        Some(vec![(1f32, amp_triangle)]),
        None,
        None
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

    fn max_mul(fund:f32, evens:bool) -> Vec<f32> {
        let max_k = (NFf / fund) as usize;
        if evens {
            (1..max_k).map(|x| x as f32).collect()
        } else {
            (1..max_k).filter(|x| x % 2 == 1).map(|x| x as f32).collect()
        }
    }

    fn nearly_none_square(fund:f32) -> Element {
        Element {
            mode: Mode::Melodic,
            muls: max_mul(fund, false),
            amps: vec![1f32; max_mul(fund, false).len()],
            phss: vec![pi2;  max_mul(fund, false).len()],
            modders: modders_square(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    fn nearly_none_triangle(fund:f32) -> Element {
        Element {
            mode: Mode::Melodic,
            muls: max_mul(fund, false),
            amps: vec![1f32; max_mul(fund, false).len()],
            phss: vec![pi2;  max_mul(fund, false).len()],
            modders: modders_triangle(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    fn nearly_none_sawtooth(fund:f32) -> Element {
        Element {
            mode: Mode::Melodic,
            muls: max_mul(fund, true),
            amps: vec![1f32; max_mul(fund, true).len()],
            phss: vec![pi2;  max_mul(fund, true).len()],
            modders: modders_triangle(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    #[test]
    fn test_blend_single_element_square() {
        let test_name:&str = "melodic-default-square";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let elementor:Elementor = vec![
            (1f32, nearly_none_square)
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_blend_single_element_triangle() {
        let test_name:&str = "melodic-default-triangle";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let elementor:Elementor = vec![
            (1f32, nearly_none_triangle)
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_blend_single_element_sawtooth() {
        let test_name:&str = "melodic-default-sawtooth";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let elementor:Elementor = vec![
            (1f32, nearly_none_sawtooth)
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_blend_composite() {
        let test_name:&str = "melodic-composite";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let elementor:Elementor = vec![
            (0.34f32, nearly_none_triangle),
            (0.33f32, nearly_none_square),
            (0.33f32, nearly_none_sawtooth),
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor));
        }

        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }
}