pub mod noise;
pub mod transient;
pub mod volume;


pub fn fit(a:f32, b:f32) -> f32 {
    if b >= a && b < (a*2.) {
        return b
    } else if b < a {
        return fit(a, b*2.0)
    } else {
        return fit (a, b/2.0)
    }
}