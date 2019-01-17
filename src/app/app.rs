extern crate sdl2;

use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

use std::time::Duration;

use super::super::types::Byte;
use super::super::x68k::X68k;

pub struct App {
    sdl_context: Sdl,
    canvas: WindowCanvas,

    x68k: X68k,
}

impl App {
    pub fn new(ipl: Vec<Byte>) -> App {
        let x68k = X68k::new(ipl);

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("X68000", 800, 600)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        App {
            sdl_context,
            canvas,

            x68k,
        }
    }

    pub fn run(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 255, 255));
        self.canvas.clear();
        self.canvas.present();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut i = 0;
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }

            self.update();

            i = (i + 1) % 255;
            self.canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            self.canvas.clear();

            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }

    fn update(&mut self) {
        // TODO: Proceed appropriate cycles.
        self.x68k.update(10000)
    }
}
