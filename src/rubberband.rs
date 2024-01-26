use std::collections::HashMap;

use thiserror::Error;

pub use rubberband_sys::RubberBandOption;
extern crate rubberband_sys;

use rubberband_sys::*;

pub fn set_default_debug_level(level: DebugLevel) {
    unsafe {
        rubberband_set_default_debug_level(level.to_value());
    }
}

pub struct OfflineStretcher {
    inner: RubberBandState,
    studied: bool,
}

impl Drop for OfflineStretcher {
    fn drop(&mut self) {
        unsafe {
            rubberband_delete(self.inner);
        }
    }
}

impl OfflineStretcher {
    pub fn new(sample_rate: u32, channels: u32) -> Self {
        let inner = unsafe {
            rubberband_new(
                sample_rate,
                channels,
                RubberBandOption_RubberBandOptionProcessOffline as _,
                1.0,
                1.0,
            )
        };
        Self {
            inner,
            studied: false,
        }
    }

    pub fn builder() -> OfflineStretcherBuilder {
        OfflineStretcherBuilder::new()
    }

    pub fn reset(&mut self) {
        self.studied = false;
        unsafe {
            rubberband_reset(self.inner);
        }
    }

    /// Provide a block of "samples" sample frames for tthe stretcher to study and calculate a
    /// stretch profile from.
    ///
    /// # Examples
    /// ```
    /// use rubberband::OfflineStretcher;
    /// let mut stretcher = OfflineStretcher::new(44100, 2);
    /// let input = vec![
    ///     vec![0.0, 0.5, 1.0, 0.5],
    ///     vec![0.0, 0.5, 1.0, 0.5],
    /// ];
    /// stretcher.study(&input, true);
    /// ```
    pub fn study<I: AsRef<[f32]>>(&mut self, input: &[I], last: bool) {
        if input.len() > 0 {
            let size = input.iter().map(|i| i.as_ref().len()).min().unwrap();
            // Convert a 2-dimensional vector to a vector of a pointer.
            let input: Vec<_> = input.iter().map(|i| i.as_ref().as_ptr()).collect();
            unsafe {
                rubberband_study(self.inner, input.as_ptr(), size as _, last as _);
            }
        } else {
            unsafe {
                rubberband_study(self.inner, std::ptr::null(), 0, last as _);
            }
        }
        self.studied = true;
    }

    /// Provide a block of "samples" sample frames for processing.
    ///
    /// # Examples
    /// ```
    /// use rubberband::OfflineStretcher;
    /// let mut stretcher = OfflineStretcher::new(44100, 2);
    /// let input = vec![vec![0.0; stretcher.samples_required()]; stretcher.channel_count()];
    /// stretcher.process(&input, true);
    /// ```
    pub fn process<I: AsRef<[f32]>>(&mut self, input: &[I], last: bool) {
        if !self.studied {
            return;
        }

        if input.len() > 0 {
            let size = input.iter().map(|i| i.as_ref().len()).min().unwrap();
            // Convert a 2-dimensional vector to a vector of a pointer.
            let input: Vec<_> = input.iter().map(|i| i.as_ref().as_ptr()).collect();
            unsafe {
                rubberband_process(self.inner, input.as_ptr(), size as _, last as _);
            }
        } else {
            unsafe {
                rubberband_process(self.inner, std::ptr::null(), 0, last as _);
            }
        }
    }

    /// Tell how many audio sample frames are available.
    ///
    /// # Error
    /// If the stretcher doesn't have enough available data, an error will be returned.
    pub fn available(&self) -> Result<usize, NotAvailableError> {
        let available = unsafe { rubberband_available(self.inner) };
        if available > 0 {
            Ok(available as _)
        } else if available < 0 {
            Err(NotAvailableError::Finished)
        } else {
            Err(NotAvailableError::NotEnoughData)
        }
    }

    /// Obtain some processed output data from the stretcher.
    ///
    /// Up to `output.len()` samples will be stored. The return value is the actual number of
    /// sample frames retrieved.
    ///
    /// # Examples
    /// ```
    /// use rubberband::OfflineStretcher;
    /// let mut stretcher = OfflineStretcher::new(44100, 2);
    /// if let Ok(available) = stretcher.available() {
    ///     let mut output = vec![vec![0.0; available]; stretcher.channel_count()];
    ///     stretcher.retrieve(&mut output);
    /// }
    /// ```
    pub fn retrieve<O: AsMut<[f32]>>(&mut self, output: &mut [O]) -> usize {
        if output.len() > 0 {
            let size = output.iter_mut().map(|o| o.as_mut().len()).min().unwrap();
            // Convert a 2-dimensional vector to a vector of a pointer.
            let output: Vec<_> = output.iter_mut().map(|o| o.as_mut().as_mut_ptr()).collect();
            unsafe { rubberband_retrieve(self.inner, output.as_ptr(), size as _) as _ }
        } else {
            0
        }
    }

    /// Set the time ratio.
    ///
    /// This will not be called after `study()` or `process()` has been called.
    pub fn set_time_ratio(&mut self, ratio: f64) {
        if !self.studied {
            unsafe {
                rubberband_set_time_ratio(self.inner, ratio);
            }
        }
    }

    /// Set the pitch scaling.
    ///
    /// This will not be called after `study()` or `process()` has been called.
    pub fn set_pitch_scale(&mut self, scale: f64) {
        if !self.studied {
            unsafe {
                rubberband_set_pitch_scale(self.inner, scale);
            }
        }
    }

    /// Change a `PhaseMode`.
    ///
    /// If running multi-threaded, the change may not take effect immediately.
    pub fn set_phase_mode(&mut self, phase: PhaseMode) {
        unsafe {
            rubberband_set_phase_option(self.inner, phase.to_value() as _);
        }
    }

    /// Change a `FormantMode`.
    ///
    /// If running multi-threaded, the change may not take effect immediately.
    pub fn set_formant_mode(&mut self, formant: FormantMode) {
        unsafe {
            rubberband_set_formant_option(self.inner, formant.to_value() as _);
        }
    }

    /// Tell the stretcher exactly how many input samples it will receive.
    pub fn set_expected_input_duration(&mut self, samples: u32) {
        unsafe {
            rubberband_set_expected_input_duration(self.inner, samples);
        }
    }

    /// Provide a set of mappings from "before" to "after" sample numbers so as to enforce a
    /// particular stretch profile.
    ///
    /// This will not be called after `study()` or `process()` has been called.
    pub fn set_key_frame_map(&mut self, map: &HashMap<u32, u32>) {
        if !self.studied {
            let (mut from, mut to): (Vec<u32>, Vec<u32>) = map.iter().unzip();
            unsafe {
                rubberband_set_key_frame_map(
                    self.inner,
                    map.len() as _,
                    from.as_mut_ptr(),
                    to.as_mut_ptr(),
                );
            }
        }
    }

    pub fn set_debug_level(&mut self, level: DebugLevel) {
        unsafe {
            rubberband_set_debug_level(self.inner, level.to_value());
        }
    }

    pub fn time_ratio(&self) -> f64 {
        unsafe { rubberband_get_time_ratio(self.inner) }
    }

    pub fn pitch_scale(&self) -> f64 {
        unsafe { rubberband_get_pitch_scale(self.inner) }
    }

    /// Return the processing latency.
    ///
    /// In this mode, this always returns 0.
    pub fn latency(&self) -> u32 {
        unsafe { rubberband_get_latency(self.inner) }
    }

    pub fn samples_required(&self) -> usize {
        unsafe { rubberband_get_samples_required(self.inner) as _ }
    }

    pub fn channel_count(&self) -> usize {
        unsafe { rubberband_get_channel_count(self.inner) as _ }
    }
}

pub struct OfflineStretcherBuilder {
    stretch_mode: StretchMode,
    transient_mode: TransientMode,
    detector_mode: DetectorMode,
    phase_mode: PhaseMode,
    thread_mode: ThreadMode,
    window_size: WindowSize,
    smooth_mode: SmoothMode,
    formant_mode: FormantMode,
    pitch_shift_mode: PitchShiftMode,
    channel_mode: ChannelMode,
    time_ratio: f64,
    pitch_scale: f64,
    max_process_size: Option<u32>,
}

impl OfflineStretcherBuilder {
    pub fn new() -> Self {
        Self {
            stretch_mode: StretchMode::Elastic,
            transient_mode: TransientMode::Crisp,
            detector_mode: DetectorMode::Compound,
            phase_mode: PhaseMode::Laminar,
            thread_mode: ThreadMode::Auto,
            window_size: WindowSize::Standard,
            smooth_mode: SmoothMode::Off,
            formant_mode: FormantMode::Shifted,
            pitch_shift_mode: PitchShiftMode::HighSpeed,
            channel_mode: ChannelMode::Apart,
            time_ratio: 1.0,
            pitch_scale: 1.0,
            max_process_size: None,
        }
    }

    pub fn percussive() -> Self {
        Self {
            phase_mode: PhaseMode::Independent,
            window_size: WindowSize::Short,
            ..Self::new()
        }
    }

    pub fn build(self, sample_rate: u32, channels: u32) -> OfflineStretcher {
        let options = RubberBandOption_RubberBandOptionProcessOffline
            | self.stretch_mode.to_value()
            | self.transient_mode.to_value()
            | self.detector_mode.to_value()
            | self.phase_mode.to_value()
            | self.thread_mode.to_value()
            | self.window_size.to_value()
            | self.smooth_mode.to_value()
            | self.formant_mode.to_value()
            | self.pitch_shift_mode.to_value()
            | self.channel_mode.to_value();
        let inner = unsafe {
            rubberband_new(
                sample_rate,
                channels,
                options as _,
                self.time_ratio,
                self.pitch_scale,
            )
        };
        if let Some(max_process_size) = self.max_process_size {
            unsafe {
                rubberband_set_max_process_size(inner, max_process_size);
            }
        }
        OfflineStretcher {
            inner,
            studied: false,
        }
    }

    pub fn stretch(self, mode: StretchMode) -> Self {
        Self {
            stretch_mode: mode,
            ..self
        }
    }

    pub fn transient(self, mode: TransientMode) -> Self {
        Self {
            transient_mode: mode,
            ..self
        }
    }

    pub fn detector(self, mode: DetectorMode) -> Self {
        Self {
            detector_mode: mode,
            ..self
        }
    }

    pub fn phase(self, mode: PhaseMode) -> Self {
        Self {
            phase_mode: mode,
            ..self
        }
    }

    pub fn thread(self, mode: ThreadMode) -> Self {
        Self {
            thread_mode: mode,
            ..self
        }
    }

    pub fn window_size(self, size: WindowSize) -> Self {
        Self {
            window_size: size,
            ..self
        }
    }

    pub fn smooth(self, mode: SmoothMode) -> Self {
        Self {
            smooth_mode: mode,
            ..self
        }
    }

    pub fn formant(self, mode: FormantMode) -> Self {
        Self {
            formant_mode: mode,
            ..self
        }
    }

    pub fn pitch_shift(self, mode: PitchShiftMode) -> Self {
        Self {
            pitch_shift_mode: mode,
            ..self
        }
    }

    pub fn channel(self, mode: ChannelMode) -> Self {
        Self {
            channel_mode: mode,
            ..self
        }
    }

    pub fn time_ratio(self, ratio: f64) -> Self {
        Self {
            time_ratio: ratio,
            ..self
        }
    }

    pub fn pitch_scale(self, scale: f64) -> Self {
        Self {
            pitch_scale: scale,
            ..self
        }
    }

    pub fn max_process_size(self, samples: u32) -> Self {
        Self {
            max_process_size: Some(samples),
            ..self
        }
    }
}

pub struct RealTimeStretcher {
    inner: RubberBandState,
}

impl Drop for RealTimeStretcher {
    fn drop(&mut self) {
        unsafe {
            rubberband_delete(self.inner);
        }
    }
}

impl RealTimeStretcher {
    pub fn new(sample_rate: u32, channels: u32) -> Self {
        let inner = unsafe {
            rubberband_new(
                sample_rate,
                channels,
                (RubberBandOption_RubberBandOptionProcessRealTime
                    | RubberBandOption_RubberBandOptionStretchPrecise) as _,
                1.0,
                1.0,
            )
        };
        Self { inner }
    }

    pub fn builder() -> RealTimeStretcherBuilder {
        RealTimeStretcherBuilder::new()
    }

    pub fn reset(&mut self) {
        unsafe {
            rubberband_reset(self.inner);
        }
    }

    /// Provide a block of "samples" sample frames for processing.
    ///
    /// # Examples
    /// ```
    /// use rubberband::RealTimeStretcher;
    /// let mut stretcher = RealTimeStretcher::new(44100, 2);
    /// let input = vec![vec![0.0; stretcher.samples_required()]; stretcher.channel_count()];
    /// stretcher.process(&input, true);
    /// ```
    pub fn process<I: AsRef<[f32]>>(&mut self, input: &[I], last: bool) {
        if input.len() > 0 {
            let size = input.iter().map(|i| i.as_ref().len()).min().unwrap();
            // Convert a 2-dimensional vector to a vector of a pointer.
            let input: Vec<_> = input.iter().map(|i| i.as_ref().as_ptr()).collect();
            unsafe {
                rubberband_process(self.inner, input.as_ptr(), size as _, last as _);
            }
        } else {
            unsafe {
                rubberband_process(self.inner, std::ptr::null(), 0, last as _);
            }
        }
    }

    /// Tell how many audio sample frames are available.
    ///
    /// # Error
    /// If the stretcher doesn't have enough available data, an error will be returned.
    pub fn available(&self) -> Result<usize, NotAvailableError> {
        let available = unsafe { rubberband_available(self.inner) };
        if available > 0 {
            Ok(available as _)
        } else if available < 0 {
            Err(NotAvailableError::Finished)
        } else {
            Err(NotAvailableError::NotEnoughData)
        }
    }

    /// Obtain some processed output data from the stretcher.
    ///
    /// Up to `output.len()` samples will be stored. The return value is the actual number of
    /// sample frames retrieved.
    ///
    /// # Examples
    /// ```
    /// use rubberband::RealTimeStretcher;
    /// let mut stretcher = RealTimeStretcher::new(44100, 2);
    /// if let Ok(available) = stretcher.available() {
    ///     let mut output = vec![vec![0.0; available]; stretcher.channel_count()];
    ///     stretcher.retrieve(&mut output);
    /// }
    /// ```
    pub fn retrieve<O: AsMut<[f32]>>(&mut self, output: &mut [O]) -> usize {
        if output.len() > 0 {
            let size = output.iter_mut().map(|o| o.as_mut().len()).min().unwrap();
            // Convert a 2-dimensional vector to a vector of a pointer.
            let output: Vec<_> = output.iter_mut().map(|o| o.as_mut().as_mut_ptr()).collect();
            unsafe { rubberband_retrieve(self.inner, output.as_ptr(), size as _) as _ }
        } else {
            0
        }
    }

    /// Set the time ratio.
    ///
    /// You should call this from the same thread as `process()`.
    pub fn set_time_ratio(&mut self, ratio: f64) {
        unsafe {
            rubberband_set_time_ratio(self.inner, ratio);
        }
    }

    /// Set the pitch scaling.
    ///
    /// You should call this from the same thread as `process()`.
    pub fn set_pitch_scale(&mut self, scale: f64) {
        unsafe {
            rubberband_set_pitch_scale(self.inner, scale);
        }
    }

    pub fn set_transient_mode(&mut self, transient: TransientMode) {
        unsafe {
            rubberband_set_transients_option(self.inner, transient.to_value() as _);
        }
    }

    pub fn set_detector_mode(&mut self, detector: DetectorMode) {
        unsafe {
            rubberband_set_detector_option(self.inner, detector.to_value() as _);
        }
    }

    pub fn set_phase_mode(&mut self, phase: PhaseMode) {
        unsafe {
            rubberband_set_phase_option(self.inner, phase.to_value() as _);
        }
    }

    pub fn set_formant_mode(&mut self, formant: FormantMode) {
        unsafe {
            rubberband_set_formant_option(self.inner, formant.to_value() as _);
        }
    }

    pub fn set_pitch_shift_mode(&mut self, pitch_shift: PitchShiftMode) {
        unsafe {
            rubberband_set_pitch_option(self.inner, pitch_shift.to_value() as _);
        }
    }

    pub fn set_debug_level(&mut self, level: DebugLevel) {
        unsafe {
            rubberband_set_debug_level(self.inner, level.to_value());
        }
    }

    pub fn time_ratio(&self) -> f64 {
        unsafe { rubberband_get_time_ratio(self.inner) }
    }

    pub fn pitch_scale(&self) -> f64 {
        unsafe { rubberband_get_pitch_scale(self.inner) }
    }

    pub fn latency(&self) -> u32 {
        unsafe { rubberband_get_latency(self.inner) }
    }

    pub fn samples_required(&self) -> usize {
        unsafe { rubberband_get_samples_required(self.inner) as _ }
    }

    pub fn channel_count(&self) -> usize {
        unsafe { rubberband_get_channel_count(self.inner) as _ }
    }
}

pub struct RealTimeStretcherBuilder {
    transient_mode: TransientMode,
    detector_mode: DetectorMode,
    phase_mode: PhaseMode,
    window_size: WindowSize,
    smooth_mode: SmoothMode,
    formant_mode: FormantMode,
    pitch_shift_mode: PitchShiftMode,
    channel_mode: ChannelMode,
    time_ratio: f64,
    pitch_scale: f64,
    max_process_size: Option<u32>,
}

impl RealTimeStretcherBuilder {
    pub fn new() -> Self {
        Self {
            transient_mode: TransientMode::Crisp,
            detector_mode: DetectorMode::Compound,
            phase_mode: PhaseMode::Laminar,
            window_size: WindowSize::Standard,
            smooth_mode: SmoothMode::Off,
            formant_mode: FormantMode::Shifted,
            pitch_shift_mode: PitchShiftMode::HighSpeed,
            channel_mode: ChannelMode::Apart,
            time_ratio: 1.0,
            pitch_scale: 1.0,
            max_process_size: None,
        }
    }

    pub fn percussive() -> Self {
        Self {
            phase_mode: PhaseMode::Independent,
            window_size: WindowSize::Short,
            ..Self::new()
        }
    }

    pub fn build(self, sample_rate: u32, channels: u32) -> RealTimeStretcher {
        let options = RubberBandOption_RubberBandOptionProcessRealTime
            | RubberBandOption_RubberBandOptionStretchPrecise
            | self.transient_mode.to_value()
            | self.detector_mode.to_value()
            | self.phase_mode.to_value()
            | RubberBandOption_RubberBandOptionThreadingAuto
            | self.window_size.to_value()
            | self.smooth_mode.to_value()
            | self.formant_mode.to_value()
            | self.pitch_shift_mode.to_value()
            | self.channel_mode.to_value();
        let inner = unsafe {
            rubberband_new(
                sample_rate,
                channels,
                options as _,
                self.time_ratio,
                self.pitch_scale,
            )
        };
        if let Some(max_process_size) = self.max_process_size {
            unsafe {
                rubberband_set_max_process_size(inner, max_process_size);
            }
        }
        RealTimeStretcher { inner }
    }

    pub fn transient(self, mode: TransientMode) -> Self {
        Self {
            transient_mode: mode,
            ..self
        }
    }

    pub fn detector(self, mode: DetectorMode) -> Self {
        Self {
            detector_mode: mode,
            ..self
        }
    }

    pub fn phase(self, mode: PhaseMode) -> Self {
        Self {
            phase_mode: mode,
            ..self
        }
    }

    pub fn window_size(self, size: WindowSize) -> Self {
        Self {
            window_size: size,
            ..self
        }
    }

    pub fn smooth(self, mode: SmoothMode) -> Self {
        Self {
            smooth_mode: mode,
            ..self
        }
    }

    pub fn formant(self, mode: FormantMode) -> Self {
        Self {
            formant_mode: mode,
            ..self
        }
    }

    pub fn pitch_shift(self, mode: PitchShiftMode) -> Self {
        Self {
            pitch_shift_mode: mode,
            ..self
        }
    }

    pub fn channel(self, mode: ChannelMode) -> Self {
        Self {
            channel_mode: mode,
            ..self
        }
    }

    pub fn time_ratio(self, ratio: f64) -> Self {
        Self {
            time_ratio: ratio,
            ..self
        }
    }

    pub fn pitch_scale(self, scale: f64) -> Self {
        Self {
            pitch_scale: scale,
            ..self
        }
    }

    pub fn max_process_size(self, samples: u32) -> Self {
        Self {
            max_process_size: Some(samples),
            ..self
        }
    }
}

/// Profile used for variable timestretching.
pub enum StretchMode {
    /// Preserving the quality of transient sounds as much as possible.
    ///
    /// Offline mode only, and the default in that mode.
    Elastic,

    /// To maintain as close as possible to a linear stretch ratio throughout.
    ///
    /// This is always used in real-time mode.
    Precise,
}

impl StretchMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            StretchMode::Elastic => RubberBandOption_RubberBandOptionStretchElastic,
            StretchMode::Precise => RubberBandOption_RubberBandOptionStretchPrecise,
        }
    }
}

/// Component frequency phase-reset mechanism.
///
/// This may be used at transient points to provide clarity and realism to percussion and other
/// significant transient sounds.
pub enum TransientMode {
    /// Reset component phases at the peak of each transient.
    ///
    /// This is the default.
    Crisp,

    /// Reset component phases at the peek of each transient, outside a frequency range typical of
    /// musical fundamental frequencies.
    Mixed,

    /// Do not reset component phases at any point.
    Smooth,
}

impl TransientMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            TransientMode::Crisp => RubberBandOption_RubberBandOptionTransientsCrisp,
            TransientMode::Mixed => RubberBandOption_RubberBandOptionTransientsMixed,
            TransientMode::Smooth => RubberBandOption_RubberBandOptionTransientsSmooth,
        }
    }
}

/// Type of transient detector used.
pub enum DetectorMode {
    /// For a general-purpose.
    ///
    /// This is the default.
    Compound,

    /// Detect percussive transients.
    Percussive,

    /// For an onset with less of a bias toward percussive transients.
    Soft,
}

impl DetectorMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            DetectorMode::Compound => RubberBandOption_RubberBandOptionDetectorCompound,
            DetectorMode::Percussive => RubberBandOption_RubberBandOptionDetectorPercussive,
            DetectorMode::Soft => RubberBandOption_RubberBandOptionDetectorSoft,
        }
    }
}

/// Adjustment of component frequency phases.
pub enum PhaseMode {
    /// Try to retain the continuity of phase relationships between adjacent frequency bins whose
    /// phases are behaving in similar ways.
    ///
    /// This is the default.
    Laminar,

    /// Adjust the phase in each frequency bin independently from its neighbours.
    Independent,
}

impl PhaseMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            PhaseMode::Laminar => RubberBandOption_RubberBandOptionPhaseLaminar,
            PhaseMode::Independent => RubberBandOption_RubberBandOptionPhaseIndependent,
        }
    }
}

pub enum ThreadMode {
    /// Permit the stretcher to determine its own threading model.
    ///
    /// Usually using one thread per audio channel in offline mode, one thread only in real-time
    /// mode. This is the default.
    Auto,

    /// Never use more than one thread.
    Never,

    /// Same as `Auto` except omit the check for multiple CPUs.
    Always,
}

impl ThreadMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            ThreadMode::Auto => RubberBandOption_RubberBandOptionThreadingAuto,
            ThreadMode::Never => RubberBandOption_RubberBandOptionThreadingNever,
            ThreadMode::Always => RubberBandOption_RubberBandOptionThreadingAlways,
        }
    }
}

/// Window size for FFT processing.
pub enum WindowSize {
    /// This is the default.
    Standard,
    Short,
    Long,
}

impl WindowSize {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            WindowSize::Standard => RubberBandOption_RubberBandOptionWindowStandard,
            WindowSize::Short => RubberBandOption_RubberBandOptionWindowShort,
            WindowSize::Long => RubberBandOption_RubberBandOptionWindowLong,
        }
    }
}

/// Whether the use of window-presum FFT and time-domain smoothing.
pub enum SmoothMode {
    /// This is the default.
    Off,

    On,
}

impl SmoothMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            SmoothMode::Off => RubberBandOption_RubberBandOptionSmoothingOff,
            SmoothMode::On => RubberBandOption_RubberBandOptionSmoothingOn,
        }
    }
}

/// Formant shape
pub enum FormantMode {
    /// Apply no special formant processing.
    ///
    /// This is the default.
    Shifted,

    /// Preserve the spectral envelope of the unshifted signal.
    Preserved,
}

impl FormantMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            FormantMode::Shifted => RubberBandOption_RubberBandOptionFormantShifted,
            FormantMode::Preserved => RubberBandOption_RubberBandOptionFormantPreserved,
        }
    }
}

pub enum PitchShiftMode {
    /// This is the default.
    HighSpeed,

    HighQuality,

    HighConsistency,
}

impl PitchShiftMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            PitchShiftMode::HighSpeed => RubberBandOption_RubberBandOptionPitchHighSpeed,
            PitchShiftMode::HighQuality => RubberBandOption_RubberBandOptionPitchHighQuality,
            PitchShiftMode::HighConsistency => {
                RubberBandOption_RubberBandOptionPitchHighConsistency
            }
        }
    }
}

/// Way for processing two-channel audio.
pub enum ChannelMode {
    /// Process each channel individually.
    ///
    /// This is the default.
    Apart,

    /// The first two channels are processed in mid-side format.
    ///
    /// Any channels beyond the first two are processed individually.
    Together,
}

impl ChannelMode {
    pub fn to_value(self) -> RubberBandOption {
        match self {
            ChannelMode::Apart => RubberBandOption_RubberBandOptionChannelsApart,
            ChannelMode::Together => RubberBandOption_RubberBandOptionChannelsTogether,
        }
    }
}

pub enum DebugLevel {
    /// This is the default unless call `set_default_debug_level()`.
    Error,

    Info,

    Verbose,

    VeryVerbose,
}

impl DebugLevel {
    pub fn to_value(self) -> i32 {
        match self {
            DebugLevel::Error => 0,
            DebugLevel::Info => 1,
            DebugLevel::Verbose => 2,
            DebugLevel::VeryVerbose => 3,
        }
    }
}

/// Error that might occur when calling `available()`.
#[derive(Debug, Error)]
pub enum NotAvailableError {
    /// The stretch process has been finished, and all output read.
    #[error("the process has been finished.")]
    Finished,

    /// Not enough data has been processed.
    #[error("not enough data has been processed.")]
    NotEnoughData,
}
