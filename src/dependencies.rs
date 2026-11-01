use std::{fs, io, path::PathBuf};

pub(crate) fn executable(name: &str) -> io::Result<PathBuf> {
    let root = prepare()?;
    let suffix = std::env::consts::EXE_SUFFIX;

    Ok(match name {
        "luals" => root.join(format!("luals/bin/lua-language-server{suffix}")),
        "ffmpeg" => root.join(format!("ffmpeg/bin/ffmpeg{suffix}")),
        _ => return Err(io::Error::other("Unknown bundled dependency.")),
    })
}

#[cfg(not(feature = "bundled"))]
fn prepare() -> io::Result<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/runtime")
        .join(std::env::consts::OS)
        .join("runtime");

    if !root.is_dir() {
        return Err(io::Error::other(
            "Dependencies are unavailable. Run powershell/pwsh -File scripts/distribute.ps1 prepare.",
        ));
    }

    Ok(root)
}

#[cfg(feature = "bundled")]
fn prepare() -> io::Result<PathBuf> {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

    if let Some(root) = ROOT.get() {
        return Ok(root.clone());
    }

    let bytes = include_bytes!(env!("KINEMATIC_BUNDLE"));
    let id = env!("KINEMATIC_DEPENDENCIES_ID");

    if id.len() != 64 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(io::Error::other("Invalid dependency content identity."));
    }

    let base = crate::storage::global_path("cache/dependencies")?;
    let root = install(bytes, id, &base)?;
    let _ = ROOT.set(root.clone());

    Ok(root)
}

#[cfg(any(feature = "bundled", test))]
fn install(bytes: &[u8], id: &str, base: &std::path::Path) -> io::Result<PathBuf> {
    use std::io::Read;

    fs::create_dir_all(&base)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&base, fs::Permissions::from_mode(0o700))?;
    }

    let root = base.join(id);
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(base.join(format!("{id}.lock")))?;

    lock.lock()?;

    if !root.join(".ready").is_file() {
        let staging = base.join(format!("{id}.partial"));

        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }

        fs::create_dir(&staging)?;

        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));

        for entry in archive.entries()? {
            let mut entry = entry?;

            if !entry.header().entry_type().is_file() && !entry.header().entry_type().is_dir() {
                return Err(io::Error::other("Unsupported dependency archive entry."));
            }

            if !entry.unpack_in(&staging)? {
                return Err(io::Error::other("Invalid dependency archive path."));
            }
        }
        // Consume the gzip trailer before committing the cache.
        io::copy(&mut archive.into_inner().take(u64::MAX), &mut io::sink())?;

        fs::write(staging.join(".ready"), [])?;

        fs::rename(&staging, &root)?;
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_extraction_is_replaced_and_concurrent_installs_reuse_the_cache() {
        let base =
            std::env::temp_dir().join(format!("kinematic-dependencies-{}", std::process::id()));

        fs::create_dir_all(base.join("test.partial")).unwrap();
        fs::write(base.join("test.partial/incomplete"), b"Incomplete.").unwrap();

        let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        let mut header = tar::Header::new_gnu();

        header.set_size(6);
        header.set_mode(0o700);
        header.set_cksum();

        archive
            .append_data(&mut header, "tool", &b"Ready."[..])
            .unwrap();

        let bytes = archive.into_inner().unwrap().finish().unwrap();

        assert!(install(b"Invalid.", "test", &base).is_err());
        assert!(!base.join("test/.ready").exists());

        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    let root = install(&bytes, "test", &base).unwrap();

                    assert_eq!(fs::read(root.join("tool")).unwrap(), b"Ready.");
                    assert!(!root.join("incomplete").exists());
                });
            }
        });
        // A warm cache must not try to unpack again.
        assert_eq!(
            install(b"Invalid.", "test", &base).unwrap(),
            base.join("test")
        );

        fs::remove_dir_all(base).unwrap();
    }
}

pub(crate) fn self_test() -> Result<(), String> {
    use crate::{core::ProjectContext, editor::lsp::Client, scripting::LuaProject};

    use std::{
        path::Path,
        process::Command,
        time::{Duration, Instant},
    };

    fn check() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::current_dir()?.join(format!("smoke-{}", std::process::id()));
        let project = LuaProject::create(&root, "Smoke", false)?;
        let script = "return function(s) local f=k.audio_file('assets/tone.wav'); s:audio{source=f}:play(); s:wait(0.25) end\n";

        fs::write(root.join("scenes/intro.lua"), script)?;

        let tone = Command::new(executable("ffmpeg")?)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.25",
            ])
            .arg(root.join("assets/tone.wav"))
            .status()?;

        if !tone.success() {
            return Err("FFmpeg tone generation failed.".into());
        }

        let samples = crate::scripting::audio::load_file(
            fs::read(root.join("assets/tone.wav"))?,
            Instant::now() + Duration::from_secs(10),
        )?;

        if samples.len() != 12_000 || !samples.iter().flatten().any(|v| v.abs() > 0.01) {
            return Err("Audio decoding failed.".into());
        }

        crate::audio::self_test(&samples)?;

        let mut client = Client::start(
            ProjectContext::new(&root)?,
            &crate::scripting::definitions(),
        )?;
        let path = Path::new("scenes/probe.lua");
        let text = "local v = k.vec2(1, 2)\nv.";

        client.sync(path, text, 1);

        let deadline = Instant::now() + Duration::from_secs(20);

        loop {
            client.request("textDocument/completion", path, text, 1, 1, 2);

            let until = Instant::now() + Duration::from_millis(100);
            let mut complete = false;

            while Instant::now() < until {
                for reply in client.poll() {
                    let items = reply
                        .value
                        .as_array()
                        .or_else(|| reply.value["items"].as_array());

                    complete |=
                        items.is_some_and(|items| items.iter().any(|item| item["label"] == "x"));
                }

                if complete {
                    break;
                }

                std::thread::sleep(Duration::from_millis(10));
            }

            if complete {
                break;
            }

            if Instant::now() >= deadline {
                return Err(format!("LuaLS completion failed: {:?}.", client.messages).into());
            }
        }

        drop(client);
        drop(project);

        crate::app::app().open_project(&root).run_export()?;

        let output = root.join("output/Smoke.mp4");
        let decoded = Command::new(executable("ffmpeg")?)
            .args(["-v", "error", "-i"])
            .arg(&output)
            .args(["-map", "0:v:0", "-map", "0:a:0", "-f", "null", "-"])
            .status()?;

        if !decoded.success() || fs::metadata(output)?.len() == 0 {
            return Err("Export validation failed.".into());
        }

        let audio = Command::new(executable("ffmpeg")?)
            .args(["-v", "error", "-i"])
            .arg(root.join("output/Smoke.mp4"))
            .args(["-map", "0:a:0", "-f", "f32le", "-acodec", "pcm_f32le", "-"])
            .output()?;

        if !audio.status.success()
            || !audio
                .stdout
                .chunks_exact(4)
                .any(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()).abs() > 0.01)
        {
            return Err("Exported audio is silent or invalid.".into());
        }

        fs::remove_dir_all(root)?;

        println!("Validated LuaLS completion, audio decoding/playback and video/audio export.");

        Ok(())
    }

    check().map_err(|error| error.to_string())
}
