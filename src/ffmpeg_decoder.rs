use anyhow::{anyhow, Result};
use audrey::Reader;
use std::env::temp_dir;
use std::path::Path;
use std::process::Stdio;
use std::{fs::File, process::Command};

// ffmpeg -i input.mp3 -ar 16000 output.wav
fn use_ffmpeg<P: AsRef<Path>>(input_path: P, filter: Option<String>) -> Result<Vec<i16>> {
    let temp_file = temp_dir().join(format!("{}.wav", uuid::Uuid::new_v4()));
    let filter = filter.unwrap_or_else(|| "pan=mono|c0=0.5*c0+0.5*c1".to_string());

    let args = [
        "-i",
        input_path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("invalid path"))?,
        "-ar",
        "16000",
        "-ac",
        "1",
        "-af",
        &filter,
        "-c:a",
        "pcm_s16le",
        (temp_file.to_str().unwrap()),
        "-hide_banner",
        "-y",
        "-loglevel",
        "error",
    ];

    let mut pid = Command::new("ffmpeg")
        .args(args)
        .stdin(Stdio::null())
        .spawn()?;

    if pid.wait()?.success() {
        let output = File::open(&temp_file)?;
        let mut reader = Reader::new(output)?;
        let samples: Result<Vec<i16>, _> = reader.samples().collect();
        std::fs::remove_file(temp_file)?;
        samples.map_err(std::convert::Into::into)
    } else {
        Err(anyhow!("unable to convert file"))
    }
}

pub fn read_file<P: AsRef<Path>>(audio_file_path: P, filter: Option<String>) -> Result<Vec<f32>> {
    let audio_buf = use_ffmpeg(&audio_file_path, filter)?;
    Ok(whisper_rs::convert_integer_to_float_audio(&audio_buf))
}
