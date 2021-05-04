//! A basic [*Noise Gate*][wiki] algorithm.
//!
//! [wiki]: https://en.wikipedia.org/wiki/Noise_gate
#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    future_incompatible,
    bare_trait_objects,
    elided_lifetimes_in_paths,
    trivial_casts,
    unreachable_pub
)]


use dasp::sample::SignedSample;
use dasp::{Frame, Sample};

/// A [*Noise Gate*][wiki] which can be used to split a stream of audio based
/// on volume, skipping periods of silence.
///
/// [wiki]: https://en.wikipedia.org/wiki/Noise_gate
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseGate<S> {
    /// The volume level at which the gate will open (begin recording).
    pub open_threshold: S,
    /// The amount of time (in samples) the gate takes to go from open to fully
    /// closed.
    pub release_time: usize,
    state: State,
}

impl<S> NoiseGate<S> {
    /// Create a new [`NoiseGate`].
    pub const fn new(open_threshold: S, release_time: usize) -> Self {
        NoiseGate {
            open_threshold,
            release_time,
            state: State::Closed,
        }
    }

    /// Is the gate currently passing samples through to the [`Sink`]?
    pub fn is_open(&self) -> bool {
        match self.state {
            State::Open | State::Closing { .. } => true,
            State::Closed => false,
        }
    }

    /// Is the gate currently ignoring silence?
    pub fn is_closed(&self) -> bool { !self.is_open() }
}

impl<S: Sample> NoiseGate<S> {
    /// Process a batch of frames, passing spans of noise through to a `sink`.
    pub fn process_frames<K, F>(&mut self, frames: &[F], sink: &mut K)
    where
        F: Frame<Sample = S>,
        K: Sink<F>,
    {
        for &frame in frames {
            let previously_open = self.is_open();

            self.state = next_state(
                self.state,
                frame,
                self.open_threshold,
                self.release_time,
            );

            if self.is_open() {
                sink.record(frame);
            } else if previously_open {
                // the gate was previously open and has just closed
                sink.end_of_transmission();
            }
        }
    }
}

fn below_threshold<F>(frame: F, threshold: F::Sample) -> bool
where
    F: Frame,
{
    let threshold = abs(threshold.to_signed_sample());
    let negated_threshold =
        F::Sample::EQUILIBRIUM.to_signed_sample() - threshold;

    frame
        .channels()
        .map(|sample| sample.to_signed_sample())
        .all(|sample| negated_threshold < sample && sample < threshold)
}

fn abs<S: SignedSample>(sample: S) -> S {
    let zero = S::EQUILIBRIUM;
    if sample >= zero {
        sample
    } else {
        -sample
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Open,
    Closing { remaining_samples: usize },
    Closed,
}

fn next_state<F>(
    state: State,
    frame: F,
    open_threshold: F::Sample,
    release_time: usize,
) -> State
where
    F: Frame,
{
    match state {
        State::Open => {
            if below_threshold(frame, open_threshold) {
                State::Closing {
                    remaining_samples: release_time,
                }
            } else {
                State::Open
            }
        },

        State::Closing { remaining_samples } => {
            if below_threshold(frame, open_threshold) {
                if remaining_samples == 0 {
                    State::Closed
                } else {
                    State::Closing {
                        remaining_samples: remaining_samples - 1,
                    }
                }
            } else {
                State::Open
            }
        },

        State::Closed => {
            if below_threshold(frame, open_threshold) {
                State::Closed
            } else {
                State::Open
            }
        },
    }
}

/// A consumer of [`Frame`]s.
pub trait Sink<F> {
    /// Add a frame to the current recording, starting a new recording if
    /// necessary.
    fn record(&mut self, frame: F);
    /// Reached the end of the samples, do necessary cleanup (e.g. flush to
    /// disk).
    fn end_of_transmission(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN_THRESHOLD: i16 = 100;
    const RELEASE_TIME: usize = 5;

    macro_rules! test_state_transition {
        ($name:ident: $from:expr, $sample:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let start: State = $from;
                let expected: State = $expected;
                let frame: [i16; 1] = [$sample];

                let got =
                    next_state(start, frame, OPEN_THRESHOLD, RELEASE_TIME);

                assert_eq!(got, expected);
            }
        };
    }

    test_state_transition!(open_to_open: State::Open, 101 => State::Open);
    test_state_transition!(open_to_closing: State::Open, 40 => State::Closing { remaining_samples: RELEASE_TIME });
    test_state_transition!(closing_to_closed: State::Closing { remaining_samples: 0 }, 40 => State::Closed);
    test_state_transition!(closing_to_closing: State::Closing { remaining_samples: 1 }, 40 => State::Closing { remaining_samples: 0 });
    test_state_transition!(reopen_when_closing: State::Closing { remaining_samples: 1 }, 101 => State::Open);
    test_state_transition!(closed_to_closed: State::Closed, 40 => State::Closed);
    test_state_transition!(closed_to_open: State::Closed, 101 => State::Open);
}
