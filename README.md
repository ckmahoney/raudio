# Raudio

[RAW-dee-oh]
(rhymes with "audio")

An additive synthesizer for expressive music.

This is a synth designed specifically for Monic Theory and is intended to be applied in non-real time to render generated compositions. 

As Monic Theory is a superset of MIDI, Raudio is also capable rendering any MIDI composition. 

As of today (May 25 2024) this application only accepts Monic playbooks as input. (See [test-tin-pan-score.json](test-tin-pan-score.json)).

It does not accept MIDI as input. It does support inline MIDI, so it *could* accept MIDI as input if that is important to you. 

This project has a few goals. It also has some explicit no-gos of conventional audio tasks. 

The tl;dr is, 


Monics only!!!

## Goals

The Primary Objective is to have a synthesizer that creates dynamic harmonic content for any musical context. 

The supplementary objective is to provide a high level API for describing sound as a small (manageable) set of input parameters.

As a list,

  - Provide a high level API for creating and modulating sound in the frequency domain
  - Provide a secure and predictable environment for rendering audio files 
    - By definition, aliasing is not possible (haha bye FM synthesis)
  - Enable arbitrary fidelity for any sample rate
  - Provide support for time domain signal **analysis** (but not modulation)
  - Melt your face with lush sounds

## Not Goals

  - No support for creating compositions. BYO score.
  - No support for time domain signal **modulation**
    - "Conventional" lowpass and highpass filters not welcome here!
    - Hey distortion, have you tried harmonic amplification? 
    - Reverb? more like stretchy harmonics
  - Does not provide support for sample-based music
    - Leave your sample packs at home kids, we gen our own

## Tests

Most recent test results (May 25 2024)

```
test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 779.69s
```

To test, 

```
cargo test
```

If it didn't already exist, this will crate a new directory `dev-audio` for writing test results.

Many tests will create files. It is intentional that the files are not deleted so the tester can manually inspect the generated asset at their leisure.
If you are low on disk space (less that 3GB remaining) then these tests may not work for you. 

Some tests may be non-deterministic in that they produce a variant in the set of all available frequency spaces of the input test. That is, a new version of the "same" sample will overwrite the previous version. 
This is valid for this applicaton because we prefer to describe elementary components in the context of their own domain; which means there is a range of equally valid outputs.

Some tests take a long time to run because they enumerate all the available parameters. 

## Demos

Some code herein writes to an `audio/demo` directory. This is intended to represent applied music. Music which many people consider `"good"`, but described using this application's data.


## Music

Some code herein writes to an `audio/music` directory. This is intended to be creative, original music! 
You already have the results of the included ambient music generator [here](audio/music) :) 

An example of how this can be used in the `Real World`,
Here's 4 hours of music synthesized with `raudio` circa May 2024

https://www.youtube.com/watch?v=mFipUHqXrw0