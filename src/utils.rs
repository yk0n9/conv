use std::{fs::File, io::Write, thread};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::runtime::Runtime;
use whisper_cli::{Language, Model, Size, Transcript, Whisper};

pub static WHISPER: AtomicBool = AtomicBool::new(false);
pub static MERGE: AtomicBool = AtomicBool::new(false);

fn as_lrc(t: &Transcript) -> String {
    t.word_utterances
        .as_ref()
        .unwrap_or(&t.utterances)
        .iter()
        .fold(String::new(), |lrc, fragment| {
            lrc +
                format!(
                    "[{:02}:{:02}.{:02}]\n",
                    fragment.start / 100 / 60,
                    fragment.start / 100 % 60,
                    fragment.start % 100,
                ).as_str() +
                format!(
                    "[{:02}:{:02}.{:02}]{}\n",
                    fragment.start / 100 / 60,
                    fragment.start / 100 % 60,
                    fragment.start % 100,
                    fragment.text
                ).as_str()
        })
}

pub fn whisper(rt: Arc<Runtime>, path: PathBuf, lang: Language, size: Size) {
    rt.spawn(async move {
        WHISPER.store(true, Ordering::Relaxed);
        let mut w = Whisper::new(Model::new(size), Some(lang)).await;
        if let Ok(ref t) = w.transcribe(&path, false, false) {
            let lrc = as_lrc(t);
            let srt = t.as_srt();
            let path_lrc = path.with_extension("lrc");
            let path_srt = path.with_extension("srt");
            let mut file = File::create(path_lrc).unwrap();
            file.write_all(lrc.as_bytes()).unwrap();
            let mut file = File::create(path_srt).unwrap();
            file.write_all(srt.as_bytes()).unwrap();
        }
        WHISPER.store(false, Ordering::Relaxed);
    });
}

pub fn ffmpeg_merge(audio: Option<PathBuf>, image: Option<PathBuf>, subtitle: Option<PathBuf>) {
    thread::spawn(move || {
        MERGE.store(true, Ordering::Relaxed);
        if let (Some(ref image), Some(ref audio), Some(ref subtitle)) = (image, audio, subtitle) {
            let output = audio.with_extension("mp4");
            let output = output.to_str().unwrap();
            let temp = audio.with_file_name("temp").with_extension("mp4");
            let temp = temp.to_str().unwrap();

            if merge(audio.to_str().unwrap(), image.to_str().unwrap(), temp).wait().is_err() {
                MERGE.store(false, Ordering::Relaxed);
                return;
            }
            if to_mp4(subtitle.file_name().unwrap().to_str().unwrap(), output, temp).wait().is_err() {
                MERGE.store(false, Ordering::Relaxed);
                return;
            }
        } else {
            MERGE.store(false, Ordering::Relaxed);
            return;
        }

        MERGE.store(false, Ordering::Relaxed);
    });
}

fn merge(audio: &str, image: &str, temp: &str) -> Child {
    Command::new("ffmpeg")
        .args([
            "-loop",
            "1",
            "-i",
            image,
            "-i",
            audio,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-pix_fmt",
            "yuv420p",
            "-y",
            "-shortest",
            temp,
        ])
        .spawn()
        .unwrap()
}

fn to_mp4(subtitle: &str, output: &str, temp: &str) -> Child {
    Command::new("ffmpeg")
        .args([
            "-i",
            temp,
            "-vf",
            &format!("subtitles={}", subtitle),
            "-c:a",
            "copy",
            "-y",
            output,
        ])
        .spawn()
        .unwrap()
}