use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime,
    BenchmarkGroup, Criterion, Throughput,
};
use hound::WavReader;
use noise_gate::{NoiseGate, Sink};
use dasp::{Frame, sample::FromSample, Sample, sample::ToSample};
use std::{fs, path::Path};

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for entry in fs::read_dir(DATA_DIR).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let name = path.file_stem().unwrap().to_str().unwrap();
            add_benchmark(&mut group, name, &path);
        }
    }
}

fn add_benchmark(
    group: &mut BenchmarkGroup<WallTime>,
    name: &str,
    path: &Path,
) {
    let reader = WavReader::open(path).unwrap();

    let desc = reader.spec();
    assert_eq!(desc.channels, 1, "We've hard-coded frames to be [i16; 1]");
    let release_time = 2 * desc.sample_rate as usize;

    let samples = reader
        .into_samples::<i16>()
        .map(|s| [s.unwrap()])
        .collect::<Vec<_>>();

    let noise_threshold = average(&samples);

    group
        .throughput(Throughput::Elements(samples.len() as u64))
        .bench_function(name, |b| {
            b.iter(|| {
                let mut counter = Counter::default();
                let mut gate = NoiseGate::new(noise_threshold, release_time);
                gate.process_frames(&samples, &mut counter);
            });
        });
}

fn average<F>(samples: &[F]) -> F::Sample
where
    F: Frame,
    F::Sample: FromSample<f32>,
    F::Sample: ToSample<f32>,
{
    let sum: f32 = samples.iter().fold(0.0, |sum, frame| {
        sum + frame.channels().map(|s| s.to_sample()).sum::<f32>()
    });
    (sum / samples.len() as f32).round().to_sample()
}

#[derive(Debug, Default)]
struct Counter {
    samples: usize,
    chunks: usize,
}

impl<F> Sink<F> for Counter {
    fn record(&mut self, _: F) { self.samples += black_box(1); }

    fn end_of_transmission(&mut self) { self.chunks += black_box(1); }
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
