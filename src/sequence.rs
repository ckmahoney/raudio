use crate::synth_config::SynthConfig;

pub fn allocate_buffers(num_threads: usize, config: &SynthConfig) -> Vec<Vec<f32>> {
    (0..num_threads).map(|_| Vec::new()).collect()
}

pub fn merge_buffers(buffers: Vec<Vec<f32>>) -> Vec<f32> {
    buffers.into_iter().flatten().collect()
}

pub enum ErrorType {
    InvalidInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 44100;
    fn test_config() -> SynthConfig {
        SynthConfig {
            sample_rate: 44100,
            min_frequency: 20.0,
            max_frequency: 20000.0,
            amplitude_scaling: 1.0,
            phase_offset: 0.0,
            tuning_offset_hz: 0.0,
            cps: 1.0
        }
    }
    #[test]
    fn test_buffer_allocation() {
        let num_threads = 2;
        let buffers = allocate_buffers(num_threads, &test_config());
        assert_eq!(buffers.len(), num_threads);

        for buffer in buffers {
            assert!(buffer.is_empty());
        }
    }

    #[test]
    fn test_buffer_merging() {
        let buffer1 = vec![0.5; 100];
        let buffer2 = vec![0.3; 150];
        let merged_buffer = merge_buffers(vec![buffer1, buffer2]);

        assert_eq!(merged_buffer.len(), 250);
        // Check if the merge is correct: first part from buffer1, second part from buffer2
        assert_eq!(merged_buffer[99], 0.5); // Last element of buffer1
        assert_eq!(merged_buffer[100], 0.3); // First element of buffer2
    }
}
