# Raudio

[RAW-dee-oh]
(rhymes with "audio")

An additive synthesizer for expressive music.

This is a synth designed specifically for Monic Theory and is intended to be applied in non-real time to render monic playbooks as audio files. 

As Monic Theory is a superset of MIDI, `raudio` is also capable rendering any MIDI composition. 

As of today (May 25 2024) this application only accepts Monic playbooks as input. (See [test-druidic-score.json](src/demo/test-druidic-score.json)).

`raudio` does not currently accept MIDI as input. It does support inline syn-midi (a lightweight MIDI alternative), so it *could* accept MIDI as input if that is important to you. 

This project has a few goals. It also has some explicit no-gos of conventional audio tasks. 


## Demos

Some code herein writes to an `audio/demo` directory. This is intended to represent applied music. Music which many people consider `"good"`, but described using this application's data.

Current demos:
  - [Percussion synths](src/demo/beat.rs)
  - [Melodic synths](src/demo/trio.rs)
  - [Frequency Domain DSP effects](src/demo/effects.rs)


To render the demos, use 

```
cargo test demo
```

You will then have files output to directory `audio/demo`! Play the files with your favorite audio player, such as with `sox`:

```
play audio/demo/beat.wav repeat 3 reverb 20
```

## Music

Some code herein writes to an `audio/music` directory. This is intended to be creative, original music! 

An example of how this can be used in the `Real World`,
Here's 4 hours of music synthesized with `raudio` circa May 2024

https://www.youtube.com/watch?v=mFipUHqXrw0


## Goals

The Primary Objective is to have a synthesizer that creates dynamic harmonic content for any musical context. 

All of the music rendered by `raudio` should feel natural or organic. Your elders can appreciate it as well equally as well as yourself.

None of the music rendered by `raudio` should feel "mechanical" or "synthetic". 

The supplementary objective is to provide a high level API for describing sound as a small (manageable) set of input parameters.

As a list,

  - Provide a high level API for creating and modulating sound in the frequency domain
  - Provide a secure and predictable environment for rendering audio files 
    - By definition, aliasing is not possible (haha bye FM synthesis)
  - Support for per-sample paramter render 
    - Highest resolution output with respect to synth control parameters. In other words, all computations are a-rate (no k-rate).
  - Provide support for time domain signal **analysis** (but not modulation)
  - Melt your heart and glowup your soul with lush sounds

## Not Goals

  - No support for creating compositions (bring your own score).
  - No support for time domain signal **modulation**
    - "Conventional" lowpass and highpass filters not welcome here!
    - Reverb? more like stretchy harmonics


## Tests


To test, 

```
cargo test
```


Many tests will create files. It is intentional that the files are not deleted so the tester can manually inspect the generated asset at their leisure.
Please make sure you have 1-3GB of disk space available before running the tests. You can remove the assets by using `rm -rf dev-audio`.


If it didn't already exist, this will crate a new directory `dev-audio` for writing test results.


Some tests may be non-deterministic in that they produce a variant in the set of all available frequency spaces of the input test. That is, a new version of the "same" sample will overwrite the previous version. 
This is valid for this applicaton because we prefer to describe elementary components in the context of their own domain; which means there is a range of equally valid outputs.


Some tests take a long time to run because they enumerate all the available parameters. 



## License

This repository is licensed under a custom license that encourages contribution and education.

Contributors who have made significant contributions by completing an approved GitHub issue or an approved independent pull request (PR) are granted additional ability to use the Software for commercial purposes. For more details, see the [LICENSE](LICENSE) file.

Contributions are welcome and encouraged.

Non-contributors are permitted to use and view the code for personal or educational purposes only. Commercial use is strictly prohibited for non-contributors.
