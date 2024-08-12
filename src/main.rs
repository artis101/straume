mod renderer;
mod utils;
mod vm;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};
use utils::keyboard::keycode_to_vm_input;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Fantasy Console Emulator", 640, 480)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut vm = vm::VM::new();
    vm.load_bios("roms/bios.bin");
    let renderer = renderer::Renderer::new();

    let sample_program = vec![
        vm::Instruction::LoadImmediate(vm::VRAM_START, 72), // 'H'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 1, 101), // 'e'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 2, 108), // 'l'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 3, 108), // 'l'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 4, 111), // 'o'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 5, 32), // ' '
        vm::Instruction::LoadImmediate(vm::VRAM_START + 6, 87), // 'W'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 7, 111), // 'o'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 8, 114), // 'r'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 9, 108), // 'l'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 10, 100), // 'd'
        vm::Instruction::LoadImmediate(vm::VRAM_START + 11, 33), // '!'
        vm::Instruction::RandomNum(65, 90),                 // Random uppercase letter
        vm::Instruction::Load(vm::RANDOM_REGISTER),
        vm::Instruction::Store(vm::VRAM_START + 5),
        vm::Instruction::Halt,
    ];
    vm.load_program(sample_program);

    let mut last_update = Instant::now();
    let mut frame_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let input_value = keycode_to_vm_input(key);
                    vm.set_input(input_value);
                }
                _ => {}
            }
        }

        let current_time = Instant::now();
        let delta = current_time.duration_since(last_update);
        last_update = current_time;

        vm.update_timer(delta.as_millis() as u64);

        for _ in 0..1000 {
            if !vm.halted {
                vm.run_cycle();
            }
        }

        // Simulate a VBlank interrupt every 60th of a second
        frame_counter += 1;
        if frame_counter % 60 == 0 {
            vm.vblank_interrupt();
        }

        // Only render if the screen is dirty
        if vm.screen_dirty {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            renderer.render(
                &mut canvas,
                &vm.memory[vm::VRAM_START..vm::VRAM_START + vm::VRAM_SIZE],
            )?;
            canvas.present();
            vm.screen_dirty = false;
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
