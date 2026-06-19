use std::error::Error;

use emulator::*;

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
    let mut memory = memory::Memory::new(Some((0x0000, 0x1FFF)));

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
                });
                acc += cycles;
            }
            acc
        });
    }

    Ok(())
}
