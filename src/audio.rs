use sdl2::{mixer::{Channel, Chunk, LoaderRWops}, rwops::RWops};

pub struct Audio {
    last_out_3: u8,
    last_out_5: u8,

    // Loaded samples
    ufo_flying: Chunk,
    ufo_hit: Chunk,
    player_shot: Chunk,
    player_death: Chunk,
    invader_death: Chunk,
    invader_march: [Chunk; 4],
}

impl Audio {
    pub fn new() -> Result<Self, String> {
        // Initialize sdl2 mixer
        sdl2::mixer::open_audio(44_100, sdl2::mixer::AUDIO_S16LSB, 2, 1024)?;
        sdl2::mixer::allocate_channels(8);

        // Closure function to faciliate the loading of the wav files using RWOps
        let load_wav = |bytes: &[u8]| -> Result<Chunk, String> {
            RWops::from_bytes(bytes)?.load_wav()
        };

        Ok(Self {
            last_out_3: 0,
            last_out_5: 0,

            // Load .wav files
            ufo_flying: load_wav(include_bytes!("bin/rom/space_invaders/audio/0.wav"))?,
            ufo_hit: load_wav(include_bytes!("bin/rom/space_invaders/audio/8.wav"))?,
            player_shot: load_wav(include_bytes!("bin/rom/space_invaders/audio/1.wav"))?,
            player_death: load_wav(include_bytes!("bin/rom/space_invaders/audio/2.wav"))?,
            invader_death: load_wav(include_bytes!("bin/rom/space_invaders/audio/3.wav"))?,
            invader_march: [
                load_wav(include_bytes!("bin/rom/space_invaders/audio/4.wav"))?,
                load_wav(include_bytes!("bin/rom/space_invaders/audio/5.wav"))?,
                load_wav(include_bytes!("bin/rom/space_invaders/audio/6.wav"))?,
                load_wav(include_bytes!("bin/rom/space_invaders/audio/7.wav"))?,
            ],
        })
    }

    pub fn play_sound_port_3(&mut self, data: u8) {
        // Check rising edge
        let rising_edge = data & !self.last_out_3;

        // UFO Flying loops while bit 0 is high
        if rising_edge & 0x01 != 0 {
            // Play on specific sound while looping indefinitely
            Channel(1).play(&self.ufo_flying, -1).unwrap();
        } else if (data & 0x01 == 0) && (self.last_out_3 & 0x01 != 0) {
            // Bit changed from 1 to 0 (falling edge) - stop the UFO sound
            Channel(1).halt();
        }

        if rising_edge & 0x02 != 0 {
            Channel::all().play(&self.player_shot, 0).unwrap();
        }
        if rising_edge & 0x04 != 0 {
            Channel::all().play(&self.player_death, 0).unwrap();
        }
        if rising_edge & 0x08 != 0 {
            Channel::all().play(&self.invader_death, 0).unwrap();
        }

        self.last_out_3 = data;
    }

    pub fn play_sound_port_5(&mut self, data: u8) {
        // Check rising edge
        let rising_edge = data & !self.last_out_5;

        // UFO Flying loops while bit 0 is high
        if rising_edge & 0x01 != 0 {
            Channel::all().play(&self.invader_march[0], 0).unwrap();
        }
        if rising_edge & 0x02 != 0 {
            Channel::all().play(&self.invader_march[1], 0).unwrap();
        }
        if rising_edge & 0x04 != 0 {
            Channel::all().play(&self.invader_march[2], 0).unwrap();
        }
        if rising_edge & 0x08 != 0 {
            Channel::all().play(&self.invader_march[3], 0).unwrap();
        }
        if rising_edge & 0x10 != 0 {
            Channel::all().play(&self.ufo_hit, 0).unwrap();
        }

        self.last_out_5 = data;
    }
}
