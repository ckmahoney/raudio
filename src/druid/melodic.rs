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
    (1..n).filter(|i| i % 2 == 0).map(|x| x as f32).collect()
}

static c_square:f32 = 4f32/pi;
/// Expects to be applied in the context of odd k 1,3,5... but is actually given an index. Makes an internal adjustment
fn amp_square(k:f32, x:f32, d:f32) -> f32 {
    let n = k * 2f32 - 1f32;
    c_square/n
}

static c_triangle:f32 = 8f32/(pi*pi);
/// Expects to be applied in the context of odd k 1,3,5... but is actually given an index. Makes an internal adjustment
/// That is; no need to return 0 as only valued k should be given.
fn amp_triangle(k:f32, x:f32, d:f32) -> f32 {
    let n = k * 2f32 - 1f32;
    let sign = (-1f32).powf((n-1f32)/2f32);
    sign * c_triangle/(n * n)
}

static c_sawtooth:f32 = 2f32/pi;
/// Expects to be applied in the context of all k 1,2,3,4...
fn amp_sawtooth(k:f32, x:f32, d:f32) -> f32 {
    let sign = (-1f32).powf(k+1f32);
    sign * c_sawtooth/k
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


    fn nearly_none_square() -> Element {
        Element {
            mode: Mode::Melodic,
            muls: vec![1.0, 3.0, 5.0, 7.0,9.0,11.0,13.0,15.0,17.0,19.0,21.0,23.0],
            modders: melodic::modders_square(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }


    fn nearly_none_triangle() -> Element {
        Element {
            mode: Mode::Melodic,
            muls: vec![1.0, 3.0, 5.0, 7.0,9.0,11.0,13.0,15.0,17.0,19.0,21.0,23.0],
            modders: melodic::modders_triangle(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    fn nearly_none_sawtooth() -> Element {
        Element {
            mode: Mode::Melodic,
            muls: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0,12.0, 13.0,14.0, 15.0,16.0, 17.0,18.0, 19.0,20.0, 21.0,23.0],
            modders: melodic::modders_triangle(),
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }

    #[test]
    fn test_blend_single_element_square() {
        let test_name:&str = "melodic-default-square";
        let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
        let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
        let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
        let mut signal:SampleBuffer = Vec::new();

        let druid:Druid = vec![
            (1f32, nearly_none_square())
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

    #[test]
    fn test_blend_single_element_triangle() {
        let test_name:&str = "melodic-default-triangle";
        let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
        let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
        let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
        let mut signal:SampleBuffer = Vec::new();

        let druid:Druid = vec![
            (1f32, nearly_none_triangle())
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



    #[test]
    fn test_blend_single_element_sawtooth() {
        let test_name:&str = "melodic-default-sawtooth";
        let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
        let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
        let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
        let mut signal:SampleBuffer = Vec::new();

        let druid:Druid = vec![
            (1f32, nearly_none_sawtooth())
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

    #[test]
    fn test_blend_composite() {
        let test_name:&str = "melodic-composite";
        let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
        let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
        let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
        let mut signal:SampleBuffer = Vec::new();

        let druid:Druid = vec![
            (0.44f32, nearly_none_triangle()),
            (0.33f32, nearly_none_square()),
            (0.33f32, nearly_none_sawtooth()),
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