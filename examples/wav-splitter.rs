use hound::{WavReader, WavSpec, WavWriter};
use noise_gate::NoiseGate;
use sample::Frame;
use std::{
    error::Error,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    let reader = WavReader::open(&args.input_file)?;
    let header = reader.spec();
    let samples = reader
        .into_samples::<i16>()
        .map(|result| result.map(|sample| [sample]))
        .collect::<Result<Vec<_>, _>>()?;

    let release_time = (header.sample_rate as f32 * args.release_time).round();

    fs::create_dir_all(&args.output_dir)?;
    let mut sink = Sink::new(args.output_dir, args.prefix, header);

    let mut gate = NoiseGate::new(args.noise_threshold, release_time as usize);
    gate.process_frames(&samples, &mut sink);

    Ok(())
}

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    #[structopt(help = "The WAV file to read")]
    pub input_file: PathBuf,
    #[structopt(short = "t", long = "threshold", help = "The noise threshold")]
    pub noise_threshold: i16,
    #[structopt(
        short = "r",
        long = "release-time",
        help = "The release time in seconds",
        default_value = "1"
    )]
    pub release_time: f32,
    #[structopt(
        short = "o",
        long = "output-dir",
        help = "Where to write the split files",
        default_value = "."
    )]
    pub output_dir: PathBuf,
    #[structopt(
        short = "p",
        long = "prefix",
        help = "A prefix to insert before each clip",
        default_value = "clip_"
    )]
    pub prefix: String,
}

pub struct Sink {
    output_dir: PathBuf,
    clip_number: usize,
    prefix: String,
    spec: WavSpec,
    writer: Option<WavWriter<BufWriter<File>>>,
}

impl Sink {
    pub fn new(output_dir: PathBuf, prefix: String, spec: WavSpec) -> Self {
        Sink {
            output_dir,
            prefix,
            spec,
            clip_number: 0,
            writer: None,
        }
    }

    fn get_writer(&mut self) -> &mut WavWriter<BufWriter<File>> {
        if self.writer.is_none() {
            let filename = self
                .output_dir
                .join(format!("{}{}.wav", self.prefix, self.clip_number));
            self.clip_number += 1;
            self.writer = Some(WavWriter::create(filename, self.spec).unwrap());
        }

        self.writer.as_mut().unwrap()
    }
}

impl<F> noise_gate::Sink<F> for Sink
where
    F: Frame,
    F::Sample: hound::Sample,
{
    fn record(&mut self, frame: F) {
        let writer = self.get_writer();

        for channel in frame.channels() {
            writer.write_sample(channel).unwrap();
        }
    }

    fn end_of_transmission(&mut self) {
        if let Some(writer) = self.writer.take() {
            writer.finalize().unwrap();
        }
    }
}
