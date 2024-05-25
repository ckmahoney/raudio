# Raudio

(RAW-dee-oh, rhymes with "audio")

An additive synthesizer for expressive music.

This is a synth designed specifically for Monic Theory and is intended to be applied in non-real time to render generated compositions. 

As Monic Theory is a superset of MIDI, Raudio is also capable rendering any MIDI composition. 

# Scope

This project has a few goals. It also has some explicit no-gos of conventional audio tasks. 

## Goals

    - Provide a high level API for creating and modulating sound in the frequency domain
    - Provide a secure and safe environment for rendering audio files 
        - By definition, aliasing is not possible (haha bye FM synthesis)
    - Enable arbitrary fidelity for any sample rate
    - Provide support for time domain signal analysis (but not modulation)
    - Melt your face with lush sounds

## Not Goals

  - Does not provide support for time domain signal modulation
    - Conventional Lowpass and highpass filters not welcome here!
    - Hey distortion, we've got cooler ways to do you
    - Reverb, more like stretchy harmonics
  - Does not provide support for sample-based music
    - Leave your sample packs at home kids, we gen our own

