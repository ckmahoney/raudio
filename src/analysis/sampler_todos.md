# TODOs for Sampler Implementation

## Goal
To provide a robust and extensible audio sampler system integrated with `raudio`, capable of handling multi-channel audio, various bit depths, and resource-aware processing. The sampler should maintain high performance, scalability, and adaptability for professional audio applications.

---

## 1. Channel Mixing
### Current State
- Each channel is stored as a separate vector.
- No utilities for channel transformations.

### Improvements
- **Mono to Stereo Conversion**: Provide a utility to duplicate mono channels into stereo.
- **Stereo to Surround**: Enable redistribution of stereo channels into custom surround configurations (e.g., 5.1).
- **Weighted Mixing**: Add options for weighted channel mixing to balance output.

### Tasks
- Create a function `mix_channels`:
  - Input: `Vec<Vec<f32>>` (multi-channel samples), target channel count.
  - Output: `Vec<Vec<f32>>` (remixed samples).
- Document the method's application in scenarios like spatial audio design.

---

## 2. Normalization Across Channels
### Current State
- Per-channel normalization without inter-channel balance consideration.

### Improvements
- Normalize amplitudes across all channels to ensure consistent perceived loudness.
- Preserve the relative balance between channels to avoid panning issues.

### Tasks
- Enhance the `normalize` function:
  - Compute a shared normalization factor based on all channels.
  - Apply the factor to all channels simultaneously.
- Add tests for multi-channel audio with varying dynamics.

---

## 3. Handling Large Files
### Current State
- Entire audio files are loaded into memory, limiting scalability.

### Improvements
- Introduce streaming to process large files in chunks.
- Maintain low memory overhead during operations like resampling and normalization.

### Tasks
- Implement chunked reading in `read_audio_file`:
  - Process audio samples incrementally.
  - Stream processed chunks to disk or output buffer.
- Add optional streaming support to `write_audio_file` for large multi-channel buffers.

---

## 4. Bit Depth Flexibility
### Current State
- Output audio files are fixed at 16-bit depth.

### Improvements
- Support writing audio in 24-bit integer and 32-bit float formats for high-fidelity applications.
- Enable format selection at runtime.

### Tasks
- Extend `write_audio_file`:
  - Accept a `bit_depth` parameter (e.g., `16`, `24`, `32f`).
  - Adjust the sample scaling logic based on the selected format.
- Update metadata handling to reflect the new bit depths.
