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

    let mut cpu = cpu::Cpu::new();
    let mut interrupts = int::Int::new();
    let mut memory = memory::Memory::new(Some((0x0000, 0x1FFF)));
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

        let keyboard = pump.keyboard_state();
        let inputs = io::ArcadeInputs {
            coin: keyboard.is_scancode_pressed(Scancode::KpEnter),
            tilt: keyboard.is_scancode_pressed(Scancode::CapsLock),
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
        // video.step(..., &bus) -> trigger either vblank or half screen interrupt

        emulator::real_time(|| {
            let mut acc = 0;
            /// 16640 cycles between each screen interrupt
            while acc < 16_640 {
                let cycles = cpu.step(&mut cpu::Bus {
                    memory: &mut memory,
                    interrupts: &mut interrupts,
                    io: &mut arcade_hw,
                });
                acc += cycles;
            }
            acc
        });
    }

    Ok(())
}
