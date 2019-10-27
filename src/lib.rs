use sample::{Frame, Sample, SignedSample};

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
    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }
}

impl<S: Sample> NoiseGate<S> {
    pub fn process_frames<K, F>(&mut self, frames: &[F], sink: &mut K)
    where
        F: Frame<Sample = S>,
        K: Sink<F>,
    {
        for &frame in frames {
            let previously_open = self.is_open();

            self.state = next_state(self.state, frame, self.open_threshold, self.release_time);

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

    frame
        .channels()
        .map(|sample| sample.to_signed_sample())
        .map(abs)
        .all(|sample| sample < threshold)
}

fn abs<S: SignedSample>(sample: S) -> S {
    let zero = S::equilibrium();
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

fn next_state<F: Frame>(
    state: State,
    frame: F,
    open_threshold: F::Sample,
    release_time: usize,
) -> State {
    match state {
        State::Open => {
            if below_threshold(frame, open_threshold) {
                State::Closing {
                    remaining_samples: release_time,
                }
            } else {
                State::Open
            }
        }

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
        }

        State::Closed => {
            if below_threshold(frame, open_threshold) {
                State::Closed
            } else {
                State::Open
            }
        }
    }
}

pub trait Sink<F> {
    /// Add a frame to the current recording, starting a new recording if
    /// necessary.
    fn record(&mut self, frame: F);
    /// Reached the end of the samples, do necessary cleanup (e.g. flush to disk).
    fn end_of_transmission(&mut self);
}
