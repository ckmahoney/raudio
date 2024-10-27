use crate::time;

#[derive(Copy,Clone,Debug)]
pub struct DelayParams {
    pub len_seconds: f32,
    pub n_echoes: usize,
    pub gain:f32,
    pub mix: f32,
}

pub fn is_passthrough(params:&DelayParams) -> bool {
    params.mix == 0f32 || params.len_seconds == 0f32 || params.gain == 0f32 || params.n_echoes == 0
}

pub static passthrough:DelayParams = DelayParams {
    mix: 0f32, 
    len_seconds: 0f32,
    gain: 0f32,
    n_echoes: 0
};

/// determine the amplitude coeffecieint for a delay replica index
#[inline]
pub fn gain(j:usize, replica:usize, params: &DelayParams) -> f32 {
    if replica == 0 || is_passthrough(params) {
        return 1f32
    }
    let samples_per_echo: usize = time::samples_from_dur(1f32, params.len_seconds);
    let min_distance = samples_per_echo * replica;
    if j < min_distance {
        return 0f32
    }

    params.mix * params.gain.powi(replica as i32)
}

/// Given a delay params, identify the new duration of the sample for a given context. 
pub fn length(cps:f32, dur:f32, params:&DelayParams) -> usize {
    let samples_per_echo: usize = time::samples_from_dur(1f32, params.len_seconds);
    let max_distance = samples_per_echo * params.n_echoes;
    max_distance
}

/// Given a list of durations for a line, determine the complete line length including its delays.
pub fn line_len(cps:f32, durs:Vec<f32>, params:&DelayParams) -> usize {
    let delays_durs:Vec<usize> = durs.iter().map(|d| length(cps, *d, params)).collect();
    // determine if any of the delays overlap. we only care about the longer events towards the end.
    0
}


mod test {
    use super::*;

    fn test_with_outlived_final_note() {
        let durs:Vec<f32> = vec![2f32, 3f32, 10f32, 2f32];
        let durs:Vec<f32> = vec![2f32, 3f32, 10f32, 10f32];
        let durs:Vec<f32> = vec![2f32, 3f32, 10f32, 11f32];

        let params = DelayParams { mix: 0.5f32, gain: 0.99, len_seconds: 1f32, n_echoes: 5};

        let total_dur:f32 = durs.iter().sum();

    }

}