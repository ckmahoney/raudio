#[derive(Debug)]
pub enum NoiseColor {
    Violet,
    Blue,
    Equal,
    Pink,
    Red,
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
            NoiseColor::Blue => (f as f32).sqrt(),
            NoiseColor::Equal => 1.0,
            NoiseColor::Pink => 1.0 / (f as f32).sqrt(),
            NoiseColor::Red => 1.0 / (f as f32).powi(2),
        }
    }
}