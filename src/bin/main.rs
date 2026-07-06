use std::error::Error;

use emulator::*;
use sdl2::keyboard::Scancode;

#[allow(unused)]
fn main() -> Result<(), Box<dyn Error>> {
    const ROM: &[u8] = include_bytes!("rom/space_invaders/invaders.rom");

    let ctx = sdl2::init()?;
    let video = ctx.video()?;

    {
        let attr = video.gl_attr();
        attr.set_context_profile(sdl2::video::GLProfile::Core);
        attr.set_context_version(4, 3);
        attr.set_context_flags().debug().set();
    }

    let win = video
        .window("CrabInvaders", 672, 768)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = win.into_canvas().accelerated().present_vsync().build()?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGB24,
        video::WIDTH as u32,
        video::HEIGHT as u32,
    )?;

    let mut cpu = cpu::Cpu::new();
    let mut interrupts = int::Int::new();
    let mut memory = memory::Memory::new(Some((0x0000, 0x1FFF)));
    let mut video = video::Video::new();
    let mut audio = audio::Audio::new()?;
    let mut arcade_hw = hardware_impl::SpaceInvadersHardware::new(io::DipSwitches::default());

    memory.load_binary(ROM, 0x0000);

    let mut pump = ctx.event_pump()?;
    'main: loop {
        while let Some(event) = pump.poll_event() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    break 'main;
                }
                _ => {}
            }
        }

        emulator::real_time(|| {
            let mut acc = 0;
            while acc < 16_000 {
                let cycles = cpu.step(&mut cpu::Bus {
                    memory: &mut memory,
                    interrupts: &mut interrupts,
                    video: &mut video,
                    audio: &mut audio,
                    io: &mut arcade_hw,
                    has_mirrors: true,
                    mirror_mask: 0x3FFF,
                });

                let keyboard = pump.keyboard_state();
                let inputs = io::ArcadeInputs {
                    coin: keyboard.is_scancode_pressed(Scancode::Escape),
                    tilt: keyboard.is_scancode_pressed(Scancode::Tab),
                    p1_start: keyboard.is_scancode_pressed(Scancode::V),
                    p2_start: keyboard.is_scancode_pressed(Scancode::B),
                    p1_left: keyboard.is_scancode_pressed(Scancode::Left),
                    p1_right: keyboard.is_scancode_pressed(Scancode::Right),
                    p1_fire: keyboard.is_scancode_pressed(Scancode::Up),
                    p2_left: keyboard.is_scancode_pressed(Scancode::A),
                    p2_right: keyboard.is_scancode_pressed(Scancode::D),
                    p2_fire: keyboard.is_scancode_pressed(Scancode::W),
                };

                arcade_hw.update_input(&inputs);
                video.step(&mut interrupts, cycles);
                if video.img_ready {
                    texture.update(None, &*video.pixel_buffer, video::WIDTH * 3);
                    canvas.copy(&texture, None, None);
                    canvas.present();
                }

                acc += cycles;
            }
            acc
        });
    }

    Ok(())
}
