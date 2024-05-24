/// # Component Amplitude Envelopes
/// 
/// ## Description
/// 
/// Provides methods for computing a value in [0, 1] given common synthesis parameters.
/// The pub methods accept three parameters: `k`, `x`, and `d`
/// 
/// Where `k` typically represents an index (usize)
/// `x` represents the progression of time from [0, 1]
/// `d` is a free parameter representing duration in cycles.
/// 
/// ## Guarantees
/// 
/// Functions are guaranteed to be responsive to both x and k and guaranteed to return a value in [0, 1] for all x in [0,1].
/// Functions are guaranteed to be defined for all x in [0,1] but no guarantees outside this domain.
/// Functions may optionally respond to d parameter.
/// 
/// View implementations of these (as of May 23 2024)
/// https://www.desmos.com/calculator/ar9rw3klcs
pub type Ranger = fn(f32, f32, f32) -> f32;
pub type Mixer = (f32, Ranger);

static options:[Ranger; 3] = [
    a,
    b,
    c
];

static neg:f32 = -1f32;
static one:f32 = 1f32;
static two:f32 = 2f32;
static half:f32 = 0.5f32;

/// Transformer based on logistic function for output in range [0, 1]
/// Only one conform method is allowed. 
fn conform(y:f32) -> f32 {
    // mutation looks good in desmos; can remove this edit for a more pure conformation
    let z = y - 0.5; 

    let denom:f32 = one + (3f32 * (1.5f32 - z)).exp();
    one / denom
}

/// Given a point (k, x, d) and group of weighted rangers,
/// Apply the weighted sum of all rangers at (k,x,d)
pub fn mix(k:f32, x:f32, d:f32, mixers:&Vec<Mixer>) -> f32 {
    let weight = mixers.iter().fold(0f32, |acc, w| acc + w.0);
    if weight > 1f32 {
        panic!("Cannot mix rangers whose total weight is more than 1")
    };

    mixers.iter().fold(0f32, |y, (w, ranger)| y + (w * ranger(k, x, d)))
}

/// Model based on (1/x)
pub fn a(k:f32, x:f32, d:f32) -> f32 {
    if x == 0f32 {
        return 1f32
    }

    let y = one / (k * x * x.sqrt());
    conform(y)
}

/// Model based on (1/x^2)
pub fn b(k:f32, x:f32, d:f32) -> f32 {
    if x == 0f32 {
        return 1f32
    }

    let y = 0.1f32 * k.sqrt() / (x*x);
    conform(y)
}

/// Model inspired by the logistic function 
pub fn c(k:f32, x:f32, d:f32) -> f32 {
    let p = -0.75f32 * (one + x * (half * k).log10());
    let y = (two / (one - p.exp())) - one;
    conform(y)
}

#[cfg(test)]
mod test {
    use super::*;

    const MONICS: [usize; 59] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
];
    const DOMAIN: [f32; 48000] = {
        let mut array = [0.0; 48000];
        let mut i = 0;
        while i < 48000 {
            array[i] = i as f32 / 48000.0;
            i += 1;
        }
        array
    };

    const min:f32 = 0f32;
    const max:f32 = 1f32;
    const d:f32 = 1f32;

    #[test]
    fn test_valid_range() {
        for (i, ranger) in (&options).iter().enumerate() {
            for k in MONICS {
                let kf = k as f32;
                let mut has_value = false;
                let mut not_one = false;
                for x in DOMAIN {
                    let y = ranger(kf, x, d);
                    if y > 0f32 && !has_value {
                        has_value = true
                    };
                    if y < 1f32 && !not_one {
                        not_one = true
                    };
                    assert!(y >= min, "Ranger {} must not produce values below {}", i, min);
                    assert!(y <= max, "Ranger {} must not produce values above {}", i, max);
                }
                assert!(has_value, "Ranger {} must not be 0 valued over its domain", i);
                assert!(not_one, "Ranger {} must not be 1 valued over its domain", i);
            }
        }
    }

    #[test]
    fn test_mix() {
        let mixers:Vec<Mixer> = (&options).iter().map(|ranger| (1f32/options.len() as f32, *ranger)).collect();
        for k in MONICS {
            let kf = k as f32;
            let mut has_value = false;
            let mut not_one = false;
            for x in DOMAIN {
                let y = mix(kf, x, d, &mixers);
                if y > 0f32 && !has_value {
                    has_value = true
                };
                if y < 1f32 && !not_one {
                    not_one = true
                };
                assert!(y >= min, "Mixing rangers must not produce values below {}", min);
                assert!(y <= max, "Mixing rangers must not produce values above {}", max);
            }
            assert!(has_value, "Mixing rangers must not be 0 valued over its domain");
            assert!(not_one, "Mixing rangers must not be 1 valued over its domain");
        }
    }
}