use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, Throughput,
};
use hound::WavReader;
use noise_gate::{NoiseGate, Sink};
use std::fs;

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");

#[derive(Debug, Default)]
struct Counter {
    samples: usize,
    chunks: usize,
}

impl<F> Sink<F> for Counter {
    fn record(&mut self, _: F) {
        self.samples += 1;
    }

    fn end_of_transmission(&mut self) {
        self.chunks += 1;
    }
}

fn parse_wav(raw: &[u8]) -> Vec<[i16; 1]> {
    WavReader::new(raw)
        .unwrap()
        .into_samples::<i16>()
        .map(|s| [s.unwrap()])
        .collect()
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for entry in fs::read_dir(DATA_DIR).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let name = path.file_stem().unwrap().to_str().unwrap();
            let data = fs::read(&path).unwrap();
            add_benchmark(&mut group, name, &data);
        }
    }
}

fn add_benchmark(group: &mut BenchmarkGroup<WallTime>, name: &str, raw: &[u8]) {
    let samples = parse_wav(raw);

    group
        .throughput(Throughput::Elements(samples.len() as u64))
        .bench_function(name, |b| {
            b.iter(|| {
                let mut counter = Counter::default();
                let mut gate = NoiseGate::new(100, 44100 / 2);
                gate.process_frames(&samples, &mut counter);
            });
        });
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
