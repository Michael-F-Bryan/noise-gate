# Noise Gate

A simple [Noise Gate][wiki] algorithm for splitting an audio stream into chunks
based on volume/silence.

For an in-depth explanation of how this crate works check out [the accompanying
blog post][blog].

## Getting Started

This project is just a crate so you'll need to add it to your own program if
you want to use it.

The [`wav-splitter`](examples/wav-splitter.rs) example shows how you could
pipe the input from a WAV file through the `NoiseGate`. It also contains a
simple `Sink` which will write each snippet of continuous audio to WAV files
on disk.

For example, to split `data/N11379_KSCK.wav` with a noise threshold of `50`
and release time of `0.3` seconds, writing the clips to the `output/` 
directory, you would run the example as follows:

```console
$ cargo run --release --example wav-splitter -- \
    --output-dir output \
    --threshold 50 \
    --release-time 0.3 \
    data/N11379_KSCK.wav
$ ls output
clip_0.wav   clip_3.wav   clip_6.wav   clip_9.wav   clip_12.wav  clip_15.wav
clip_18.wav  clip_21.wav  clip_1.wav   clip_4.wav   clip_7.wav   clip_10.wav
clip_13.wav  clip_16.wav  clip_19.wav  clip_22.wav  clip_2.wav   clip_5.wav
clip_8.wav   clip_11.wav  clip_14.wav  clip_17.wav  clip_20.wav
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[blog]: http://adventures.michaelfbryan.com/posts/audio-processing-for-dummies/
[wiki]: https://en.wikipedia.org/wiki/Noise_gate
