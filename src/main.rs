use anyhow::Context;
use clap::Parser;
use indicatif::ProgressStyle;
use parse_size::parse_size;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Deflate compression level (between 1 and 264)
    #[arg(long)]
    compression_level: Option<i64>,

    /// Uncompressed size
    uncompressed_size: String,

    /// Output path
    output_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let uncompressed_size =
        parse_size(args.uncompressed_size).with_context(|| "cannot parse size")?;

    let output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&args.output_path)
        .with_context(|| "cannot open output path for write")?;
    let output_file = BufWriter::new(output_file);

    let mut zip = ZipWriter::new(output_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(args.compression_level)
        .large_file(true);

    zip.start_file("kaboom", options)?;

    let zero_buffer = [0u8; 8192];
    let mut remaining = uncompressed_size as usize;
    let bar = indicatif::ProgressBar::new(uncompressed_size);
    bar.set_style(
        ProgressStyle::with_template(
            "{wide_bar} {decimal_bytes}/{decimal_total_bytes} - rate: {decimal_bytes_per_sec} - ETA: {eta}",
        )?
        .progress_chars("##-"),
    );

    while remaining > 0 {
        let to_write = remaining.min(zero_buffer.len());
        zip.write_all(&zero_buffer[..to_write])?;
        remaining -= to_write;
        bar.inc(to_write as u64);
    }

    zip.finish()?;
    bar.finish();

    let zip_file = File::open(&args.output_path).with_context(|| "cannot open zip file")?;
    let metadata = zip_file
        .metadata()
        .with_context(|| "cannot stat zip file")?;

    println!("zip archive is {}", indicatif::HumanBytes(metadata.len()));

    Ok(())
}
