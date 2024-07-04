use super::*;

/// Constant for square wave amplitude calculation
static C_SQUARE: f32 = 4f32 / pi;

/// Constant for triangle wave amplitude calculation
static C_TRIANGLE: f32 = 8f32 / (pi * pi);


pub fn amps_sine(freq:f32) -> Vec<f32> {
    vec![1f32, 0.33f32, 0.125f32]    
}
    
/// Generates multipliers for a Fourier series sine wave starting at `freq`
pub fn muls_max_k(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).map(|x| x as f32).collect()
}    

/// Generates multipliers for a Fourier series sine wave starting at `freq`
pub fn muls_sine(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    vec![1f32, 2f32, 3f32]
}
/// Generates multipliers for a Fourier series sawtooth wave starting at `freq`
pub fn muls_sawtooth(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).map(|x| x as f32).collect()
}

/// Generates amplitudes for a Fourier series sawtooth wave starting at `freq`
pub fn amps_sawtooth(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).map(|i| 2f32 / (pi * i as f32)).collect()
}

/// Generates phases for a Fourier series sawtooth wave starting at `freq`
pub fn phases_sawtooth(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).map(|i| if i % 2 == 0 { pi } else { 0f32 }).collect()
}

/// Generates multipliers for a Fourier series square wave starting at `freq`
pub fn muls_square(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).filter(|&i| i % 2 != 0).map(|i| i as f32).collect()
}

/// Generates amplitudes for a Fourier series square wave starting at `freq`
pub fn amps_square(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).filter(|&i| i % 2 != 0).map(|i| 4f32 / (pi * i as f32)).collect()
}

/// Generates multipliers for a Fourier series triangle wave starting at `freq`
pub fn muls_triangle(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).filter(|i| i % 2 != 0).map(|i| i as f32).collect()
}

/// Generates amplitudes for a Fourier series triangle wave starting at `freq`
pub fn amps_triangle(freq: f32) -> Vec<f32> {
    let n = (NFf / freq) as usize;
    (1..=n).filter(|i| i % 2 != 0).map(|i| {
        let sign = if (i - 1) / 2 % 2 == 0 { 1f32 } else { -1f32 };
        sign * 8f32 / (pi * pi * (i as f32).powi(2))
    }).collect()
}



pub fn square(freq:f32) -> (Vec<f32>,Vec<f32>,Vec<f32>) {
    let muls = muls_square(freq);
    let amps = amps_square(freq);
    let phases = vec![0f32; muls.len()];
    (
        amps,
        muls,
        phases
    )
}

pub fn sawtooth(freq:f32) -> (Vec<f32>,Vec<f32>,Vec<f32>) {
    let muls = muls_sawtooth(freq);
    let amps = amps_sawtooth(freq);
    let phases = vec![0f32; muls.len()];
    (
        amps,
        muls,
        phases
    )
}

pub fn triangle(freq:f32) -> (Vec<f32>,Vec<f32>,Vec<f32>) {
    let muls = muls_triangle(freq);
    let amps = amps_triangle(freq);
    let phases = vec![0f32; muls.len()];
    (
        amps,
        muls,
        phases
    )
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

    fn nearly_none_square(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
        let (amps, muls, phss) = square(fund);
        Element {
            mode: Mode::Melodic,
            muls,
            amps,
            phss,
            modders: modders_none(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    fn nearly_none_triangle(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
        let (amps, muls, phss) = triangle(fund);
        Element {
            mode: Mode::Melodic,
            muls,
            amps,
            phss,
            modders: modders_none(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    fn nearly_none_sawtooth(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
        let (amps, muls, phss) = sawtooth(fund);
        Element {
            mode: Mode::Melodic,
            muls,
            amps,
            phss,
            modders: modders_none(),
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

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_square)
        ];

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
    fn test_blend_single_element_triangle() {
        let test_name:&str = "melodic-default-triangle";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_triangle)
        ];

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
    fn test_blend_single_element_sawtooth() {
        let test_name:&str = "melodic-default-sawtooth";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (1f32, nearly_none_sawtooth)
        ];

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
    fn test_blend_composite() {
        let test_name:&str = "melodic-composite";
        let (freqs, durs, frexs) = test_data();
        let mut signal:SampleBuffer = Vec::new();

        let (vis, en, pre) = test_vep();
        let elementor:Elementor = vec![
            (0.34f32, nearly_none_triangle),
            (0.33f32, nearly_none_square),
            (0.33f32, nearly_none_sawtooth),
        ];

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor, &vis, &en, &pre));
        }

        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }
}