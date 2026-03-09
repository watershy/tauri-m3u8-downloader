use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::thread;
use std::time::Duration;

fn play_audio_bytes(audio_data: &'static [u8]) {
    // Spawning a standard OS thread keeps this completely separate from Tokio's async runtime.
    // It won't block your downloads or your UI.
    // We use `move` to safely transfer the static byte array into the new thread
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                sink.set_volume(0.25);
                let cursor = Cursor::new(audio_data);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    });
}

pub fn play_success_sound() {
    let audio_data = include_bytes!("../../../public/sounds/success.mp3");
    play_audio_bytes(audio_data);
}

pub fn play_error_sound() {
    let audio_data = include_bytes!("../../../public/sounds/error.mp3");
    play_audio_bytes(audio_data);
}