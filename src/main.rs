#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;
use std::path::Path;
use utils::write_to;

mod utils;

use whisper_cli::{Language, Model, Size, Whisper};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Locally transcribe audio files, using Whisper.",
    long_about = "Generate a transcript of an audio file using the Whisper speech-to-text engine. The transcript will be saved as a .txt, .vtt, and .srt file in the same directory as the audio file."
)]
struct Args {
    /// Name of the Whisper model to use
    #[clap(short, long, default_value = "medium")]
    model: Size,

    /// Language spoken in the audio. Attempts to auto-detect by default.
    #[clap(short, long)]
    lang: Option<Language>,

    /// Path to the audio file to transcribe
    audio: String,

    /// Toggle translation
    #[clap(short, long, default_value = "false")]
    translate: bool,

    /// Set a relative output directory
    #[clap(short, long)]
    relative_output_dir: Option<String>,

    /// Generate timestamps for each word
    #[clap(short, long, default_value = "false")]
    karaoke: bool,

    /// Custom ffmpeg filter
    #[clap(short, long)]
    ffmpeg_filter: Option<String>,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    let audio = Path::new(&args.audio);
    let file_name = audio.file_name().unwrap().to_str().unwrap();

    assert!(audio.exists(), "The provided audio file does not exist.");

    if args.model.is_english_only() && (args.lang == Some(Language::Auto) || args.lang.is_none()) {
        args.lang = Some(Language::English);
    }

    assert!(
        !args.model.is_english_only() || args.lang == Some(Language::English),
        "The selected model only supports English."
    );

    let mut whisper = Whisper::new(Model::new(args.model), args.lang).await;
    let transcript = whisper
        .transcribe(audio, args.translate, args.karaoke, args.ffmpeg_filter)
        .unwrap();

    let output_dir = audio
        .with_file_name("")
        .join(args.relative_output_dir.unwrap_or(String::from("")));
    std::fs::create_dir_all(&output_dir).unwrap();

    println!("{}", transcript.as_text());
    println!("writing to {:?}", output_dir);
    println!(
        "text location: {:?}",
        output_dir.join(format!("{file_name}.txt"))
    );

    write_to(
        output_dir.join(format!("{file_name}.txt")),
        &transcript.as_text(),
    );
    write_to(
        output_dir.join(format!("{file_name}.vtt")),
        &transcript.as_vtt(),
    );
    write_to(
        output_dir.join(format!("{file_name}.srt")),
        &transcript.as_srt(),
    );
    // TODO: json output
    // TODO: recursive directory search
    // TODO: support custom ffmpeg filter
    // maybe a serve mode with a small HTTP API? Post a file - get a transcript back

    println!("time: {:?}", transcript.processing_time);
}
