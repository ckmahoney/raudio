pub mod delay;
pub mod monic_theory;
pub mod sine_cache;
pub mod time;
pub mod transient;
pub mod volume;
pub mod xform_freq;

use crate::synth::{pi,pi2};

/// Double or halve a value `b` to be within 1 octave of `a`
pub fn fit(a:f32, b:f32) -> f32 {
    if b >= a && b < (a*2.) {
        return b
    } else if b < a {
        return fit(a, b*2.0)
    } else {
        return fit (a, b/2.0)
    }
}


/// for function f(x) with range in [a, b]
/// returns g(x) for a given value y representing f(x).
pub fn map_range_lin(f_a:f32, f_b:f32, g_a:f32, g_b:f32, y:f32) -> f32 {
    let mean_g:f32 = (g_b + g_a) / 2f32;
    let range_f = (f_b - f_a).abs();
    let range_g:f32 = (g_b - g_a).abs();

    let linear_interp = range_g / range_f;
    mean_g + (linear_interp * y)
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_map_range_lin() {
        let min_f = -1f32;
        let max_f = 1f32;
        let min_g = 2f32; 
        let max_g = 3f32;

        let mut y = 0f32.sin();
        let mut expected = 2.5f32;
        let mut actual = map_range_lin(min_f, max_f, min_g, max_g, y);
        assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);

        y = (pi/2f32).sin();
        expected = 3.0f32;
        actual = map_range_lin(min_f, max_f, min_g, max_g, y);
        assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);

        y = pi.sin();
        expected = 2.5f32;
        actual = map_range_lin(min_f, max_f, min_g, max_g, y);
        assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);


        y = (3f32 * pi/2f32).sin();
        expected = 2.0f32;
        actual = map_range_lin(min_f, max_f, min_g, max_g, y);
        assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);
    }
}