extern crate num_complex;
use num_complex::Complex;

fn morlet_wavelet(t: f64, omega_0: f64) -> Complex<f64> {
    let normalization = (1.0 / std::f64::consts::PI).sqrt();
    let plane_wave = Complex::new(0.0, omega_0 * t).exp(); // e^(i*omega_0*t)
    let gaussian_window = (-t.powi(2) / 2.0).exp();
    normalization * plane_wave * gaussian_window
}

pub fn main(omega_0: f64) {
    let t = 1.0; // Example time
    let wavelet_value = morlet_wavelet(t, omega_0);
    println!("Morlet wavelet value: {:?}", wavelet_value);
}