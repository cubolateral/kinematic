use crate::core::audio::SAMPLE_RATE;
use std::{
    io::{BufReader, Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(super) fn decode(bytes: &[u8], deadline: Instant) -> Result<Vec<[f32; 2]>, String> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-protocol_whitelist",
            "pipe",
            "-format_whitelist",
            "wav,mp3,ogg,flac",
            "-i",
            "pipe:0",
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
        ])
        .args(["-f", "f32le", "-acodec", "pcm_f32le", "-ac", "2", "-ar"])
        .arg(SAMPLE_RATE.to_string())
        .arg("pipe:1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not start FFmpeg audio decoder: {error}."))?;
    let mut input = child
        .stdin
        .take()
        .expect("Audio decoder input must be piped.");
    let output = child
        .stdout
        .take()
        .expect("Audio decoder output must be piped.");
    let decoded = std::thread::scope(|scope| {
        let writer = scope.spawn(move || input.write_all(bytes));
        let reader = scope.spawn(move || -> std::io::Result<Vec<[f32; 2]>> {
            let mut output = BufReader::new(output);
            let mut samples = Vec::new();
            let mut frame = [0; 8];
            while output.read(&mut frame[..1])? != 0 {
                output.read_exact(&mut frame[1..])?;
                samples.push([
                    f32::from_le_bytes(frame[..4].try_into().unwrap()),
                    f32::from_le_bytes(frame[4..].try_into().unwrap()),
                ]);
            }
            Ok(samples)
        });
        let status = loop {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                break Err("Audio decoding exceeded the scene construction deadline.".to_owned());
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("Could not wait for audio decoder: {error}."));
                }
            }
        };
        let written = writer
            .join()
            .map_err(|_| "Audio decoder input worker failed.")?;
        let decoded = reader
            .join()
            .map_err(|_| "Audio decoder output worker failed.")?
            .map_err(|error| error.to_string())?;
        if !status?.success() || written.is_err() || decoded.is_empty() {
            return Err("FFmpeg could not decode a valid audio stream.".to_owned());
        }
        Ok(decoded)
    })?;
    let mut samples = decoded;
    if samples.iter().flatten().any(|value| !value.is_finite()) {
        return Err("Decoded audio contains non-finite samples.".into());
    }
    let count = samples.len();
    let fade = SAMPLE_RATE as f32 * 0.002;
    for (index, frame) in samples.iter_mut().enumerate() {
        let gain = (index as f32 / fade)
            .min((count - index - 1) as f32 / fade)
            .min(1.0);
        frame[0] *= gain;
        frame[1] *= gain;
    }
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires FFmpeg with WAV, MP3, OGG and FLAC support."]
    fn decoding_resamples_large_files_and_rejects_invalid_or_late_input() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/lua/audio/assets/chime.wav");
        let wav = std::fs::read(&fixture).unwrap();
        let samples = decode(&wav, Instant::now() + Duration::from_secs(5)).unwrap();
        assert!((samples.len() as f64 / SAMPLE_RATE as f64 - 0.25).abs() < 0.001);
        assert!(samples.iter().any(|sample| sample[0].abs() > 0.1));
        assert!(samples.iter().all(|sample| sample[0] == sample[1]));
        assert_eq!(samples.first().unwrap(), &[0.0; 2]);
        assert_eq!(samples.last().unwrap(), &[0.0; 2]);
        for format in ["mp3", "ogg", "flac"] {
            let encoded = Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(&fixture)
                .args(["-f", format, "pipe:1"])
                .output()
                .unwrap();
            assert!(encoded.status.success());
            let samples = decode(&encoded.stdout, Instant::now() + Duration::from_secs(5)).unwrap();
            assert!(
                samples.len() >= 10_000 && samples.len() < 16_000,
                "Invalid {format} duration."
            );
            assert!(samples.iter().flatten().all(|value| value.is_finite()));
        }
        assert!(decode(b"Invalid audio.", Instant::now() + Duration::from_secs(5)).is_err());
        assert!(
            decode(&wav, Instant::now() - Duration::from_secs(1))
                .unwrap_err()
                .contains("deadline")
        );
        let oversized = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=stereo",
                "-t",
                "167",
                "-f",
                "flac",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(oversized.status.success());
        let large = decode(&oversized.stdout, Instant::now() + Duration::from_secs(10)).unwrap();
        assert!(large.len() * 8 > 64_000_000);
    }
}
