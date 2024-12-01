# Raudio

[RAW-dee-oh]
(rhymes with "audio")

An additive synthesizer for expressive music.

This is one of the backend components of [Ten Pens](htps://tenpens.ink)! 

## About 

Congratulations, you found a newer synthesizer than most synthesizers on Earth! 

What can you do with it? 

Use it to turn math into music!!! 


This is a synth designed specifically for Monic Theory and is intended to be applied in non-real time to render monic playbooks as audio files. 

The application currently accepts Monic playbooks as input. (See [test-druidic-score.json](src/demo/test-druidic-score.json)).


As Monic Theory is a superset of MIDI, `raudio` is also capable rendering any MIDI composition. 


`raudio` supports inline syn-midi (a lightweight MIDI alternative). It does not accept .mid files, but it *could* accept .mid as input if that is important to you. 

This project has a few goals. It also has some explicit no-gos of conventional audio tasks. It also features demos!


## Demos

If you just want to hear what it sounds like, check out the [demo](demo) directory.

## Tests


To run tests: 

```
cargo test
```



### Disk space

Many tests will create files. It is intentional that the files are not deleted so the tester can manually inspect the generated asset at their leisure.

Please make sure you have 1-3GB of disk space available before running the tests. 

You can remove the assets by using `rm -rf dev-audio`.

### Test Output

A lot of tests will render audio samples featuring a speciifc feature of synthesis (ex. amplitude modulation, sequencing notes, stacking chords).

Some tests may be non-deterministic in that they produce a variant in the set of all available frequency spaces of the input test. 

A new and possibly different version of the "same" sample will overwrite the previous version. 

Some tests take a long time to run because they enumerate all the available parameters. 



## License

This repository is licensed under a custom license that encourages contribution and education.

Contributors who have made significant contributions by completing an approved GitHub issue or an approved independent pull request (PR) are granted additional ability to use the Software for commercial purposes. For more details, see the [LICENSE](LICENSE) file.

Contributions are welcome and encouraged.

Non-contributors are permitted to use and view the code for personal or educational purposes only. Commercial use is strictly prohibited for non-contributors.
