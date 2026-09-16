use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

use crate::world::SoundEvent;

const SAMPLE_RATE: u32 = 32_000;
const EVENTS: [SoundEvent; 7] = [
    SoundEvent::Jump,
    SoundEvent::Coin,
    SoundEvent::Bump,
    SoundEvent::Stomp,
    SoundEvent::Hurt,
    SoundEvent::Checkpoint,
    SoundEvent::Clear,
];

pub struct Audio {
    music: Option<Sound>,
    effects: Vec<(SoundEvent, Sound)>,
    pub muted: bool,
    playing: bool,
}

impl Audio {
    pub async fn new(muted: bool) -> Self {
        let music = load(&music_wav()).await;
        let mut effects = Vec::new();
        for event in EVENTS {
            if let Some(sound) = load(&effect_wav(event)).await {
                effects.push((event, sound));
            }
        }
        Self {
            music,
            effects,
            muted,
            playing: false,
        }
    }

    pub fn toggle(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            for (_, sound) in &self.effects {
                stop_sound(sound);
            }
        }
    }

    pub fn sync_music(&mut self, active: bool) {
        let should_play = active && !self.muted;
        if should_play != self.playing {
            if let Some(sound) = &self.music {
                if should_play {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: true,
                            volume: 0.65,
                        },
                    );
                } else {
                    stop_sound(sound);
                }
            }
            self.playing = should_play;
        }
    }

    pub fn play(&self, event: SoundEvent) {
        if !self.muted
            && let Some((_, sound)) = self.effects.iter().find(|(kind, _)| *kind == event)
        {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.55,
                },
            );
        }
    }
}

async fn load(bytes: &[u8]) -> Option<Sound> {
    match load_sound_from_bytes(bytes).await {
        Ok(sound) => Some(sound),
        Err(error) => {
            eprintln!("Could not load a sound; continuing without it: {error}");
            None
        }
    }
}

fn frequency(midi: i32) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty {
        1.0 - duty
    } else {
        -duty
    }
}

fn envelope(t: f32, length: f32) -> f32 {
    (t / 0.004).min(1.0) * ((length - t) / 0.018).clamp(0.0, 1.0)
}

fn wav(samples: &[f32]) -> Vec<u8> {
    let byte_count = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(44 + byte_count as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + byte_count).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&byte_count.to_le_bytes());
    for sample in samples {
        bytes
            .extend_from_slice(&((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    bytes
}

/// An original 16-bar loop: pulse lead, triangle bass, arpeggios and noise drums.
/// Zero is a rest. Everything is synthesized; there are no sampled game assets.
pub fn music_wav() -> Vec<u8> {
    let melody = [
        76, 0, 79, 76, 74, 72, 0, 74, 76, 79, 81, 0, 79, 76, 74, 0, 72, 76, 79, 0, 81, 79, 76, 74,
        72, 0, 69, 72, 74, 0, 76, 0, 77, 0, 81, 79, 77, 76, 74, 0, 77, 81, 84, 0, 81, 79, 77, 0,
        74, 79, 83, 81, 79, 0, 77, 74, 76, 74, 72, 0, 0, 79, 74, 0, 76, 79, 84, 0, 83, 79, 76, 0,
        81, 0, 79, 76, 74, 72, 74, 0, 72, 76, 81, 0, 79, 76, 74, 72, 69, 72, 76, 0, 74, 0, 72, 0,
        77, 81, 84, 81, 79, 0, 77, 76, 74, 77, 81, 0, 79, 77, 74, 0, 79, 0, 83, 81, 79, 77, 74, 0,
        76, 79, 72, 0, 0, 0, 0, 0,
    ];
    let roots = [
        48, 48, 45, 45, 53, 53, 55, 55, 48, 48, 45, 45, 53, 53, 55, 55,
    ];
    let step_seconds = 60.0 / 148.0 / 2.0;
    let count = (melody.len() as f32 * step_seconds * SAMPLE_RATE as f32).round() as usize;
    let mut samples = Vec::with_capacity(count);
    let mut noise = 0xACE1_u32;
    for i in 0..count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let step = ((t / step_seconds) as usize).min(melody.len() - 1);
        let local = t - step as f32 * step_seconds;
        let note = melody[step];
        let lead = if note > 0 {
            pulse(local * frequency(note), 0.25) * envelope(local, step_seconds * 0.88) * 0.31
        } else {
            0.0
        };
        let root = roots[step / 8];
        let bass_note = root + if step % 4 >= 2 { 7 } else { 0 };
        let bass_phase = (local * frequency(bass_note)).fract();
        let bass =
            (1.0 - 4.0 * (bass_phase - 0.5).abs()) * envelope(local, step_seconds * 0.9) * 0.12;
        let chord = root + 24 + [0, if root == 45 { 3 } else { 4 }, 7, 12][step % 4];
        let arp =
            pulse(local * frequency(chord), 0.5) * envelope(local, step_seconds * 0.6) * 0.055;
        noise ^= noise << 13;
        noise ^= noise >> 17;
        noise ^= noise << 5;
        let noise_sample = (noise as i32 as f32) / i32::MAX as f32;
        let hat = noise_sample * (-local * 85.0).exp() * 0.038;
        let snare = if step % 4 == 2 {
            noise_sample * (-local * 34.0).exp() * 0.08
        } else {
            0.0
        };
        let kick = if step.is_multiple_of(4) {
            (std::f32::consts::TAU * (70.0 * local - 20.0 * local * local)).sin()
                * (-local * 24.0).exp()
                * 0.13
        } else {
            0.0
        };
        samples.push(lead + bass + arp + hat + snare + kick);
    }
    wav(&samples)
}

pub fn effect_wav(event: SoundEvent) -> Vec<u8> {
    let duration = match event {
        SoundEvent::Jump => 0.17,
        SoundEvent::Coin => 0.22,
        SoundEvent::Bump => 0.10,
        SoundEvent::Stomp => 0.14,
        SoundEvent::Hurt => 0.70,
        SoundEvent::Checkpoint => 0.45,
        SoundEvent::Clear => 1.6,
    };
    let count = (duration * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / duration;
        let hz = match event {
            SoundEvent::Jump => 180.0 + progress * 640.0,
            SoundEvent::Coin => {
                if t < 0.07 {
                    frequency(88)
                } else {
                    frequency(95)
                }
            }
            SoundEvent::Bump => 135.0 - 75.0 * progress,
            SoundEvent::Stomp => 250.0 - 190.0 * progress,
            SoundEvent::Hurt => frequency(74 - (progress * 27.0) as i32),
            SoundEvent::Checkpoint => frequency([72, 76, 79, 84][(progress * 4.0) as usize % 4]),
            SoundEvent::Clear => {
                frequency([72, 76, 79, 84, 81, 84, 88, 91][(progress * 8.0) as usize % 8])
            }
        };
        phase += hz / SAMPLE_RATE as f32;
        let decay = if matches!(
            event,
            SoundEvent::Coin | SoundEvent::Bump | SoundEvent::Stomp
        ) {
            1.0 - progress
        } else {
            0.9
        };
        samples.push(pulse(phase, 0.5) * envelope(t, duration) * decay * 0.58);
    }
    wav(&samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_audio_is_valid_non_silent_pcm_without_clipping() {
        let mut clips = vec![music_wav()];
        clips.extend(EVENTS.map(effect_wav));
        for bytes in clips {
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(&bytes[8..16], b"WAVEfmt ");
            assert_eq!(
                u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                bytes.len() - 44
            );
            let samples: Vec<i16> = bytes[44..]
                .chunks_exact(2)
                .map(|s| i16::from_le_bytes([s[0], s[1]]))
                .collect();
            let peak = samples.iter().map(|&s| (s as i32).abs()).max().unwrap();
            assert!(peak > 1000 && peak < 32767, "unexpected audio peak: {peak}");
            assert!(samples.first().unwrap().abs() < 1500);
            assert!(samples.last().unwrap().abs() < 1500);
        }
    }
}
