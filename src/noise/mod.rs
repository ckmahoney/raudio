
fn violet(length: usize) -> SampleBuffer {
    let w = white(length);
    let mut violet = vec![0.0; length];

    for i in 1..length {
        violet[i] = w[i] - w[i - 1];
    }

    violet
}

fn white(length: usize) -> SampleBuffer {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn pink(length: usize) -> SampleBuffer {
    let mut rng = rand::thread_rng();
    let num_rows = 16;
    let mut rows = vec![0.0; num_rows];
    let mut pink = vec![0.0; length];

    for i in 0..length {
        let row = rng.gen_range(0..num_rows);
        rows[row] = rng.gen_range(-1.0..1.0);
        
        let running_sum: f32 = rows.iter().sum();
        pink[i] = running_sum / num_rows as f32;
    }

    pink
}