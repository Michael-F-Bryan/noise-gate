use hound::{WavReader, WavSpec, WavWriter};
use noise_gate::NoiseGate;
use dasp::Frame;

use std::{
    error::Error,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    // open the WAV file
    let reader = WavReader::open(&args.input_file)?;
    // we need the header to determine the sample rate
    let header = reader.spec();
    // read all the samples into memory, converting them to a single-channel
    // audio stream
    let samples = reader
        .into_samples::<i16>()
        .map(|result| result.map(|sample| [sample]))
        .collect::<Result<Vec<_>, _>>()?;

    let release_time = (header.sample_rate as f32 * args.release_time).round();

    // make sure the output directory exists
    fs::create_dir_all(&args.output_dir)?;
    // initialize our sink
    let mut sink = Sink::new(args.output_dir, args.prefix, header);

    // set up the NoiseGate
    let mut gate = NoiseGate::new(args.noise_threshold, release_time as usize);
    // and process all the samples
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
        default_value = "0.25"
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
            // Lazily initialize the writer. This lets us drop the writer when 
            // sent an end_of_transmission and have it automatically start
            // writing to a new clip when necessary.
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

        // write all the channels as interlaced audio
        for channel in frame.channels() {
            writer.write_sample(channel).unwrap();
        }
    }

    fn end_of_transmission(&mut self) {
        // if we were previously recording a transmission, remove the writer
        // and let it flush to disk
        if let Some(writer) = self.writer.take() {
            writer.finalize().unwrap();
        }
    }
}
