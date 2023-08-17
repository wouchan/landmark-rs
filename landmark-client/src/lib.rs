use game_loop::{
    game_loop,
    winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    },
};
use sparsey::prelude::*;

mod rendererer;
use rendererer::*;
mod model;

#[derive(Debug)]
struct Game {
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
}

impl Game {
    pub fn new(window: &Window) -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();

        resources.insert(pollster::block_on(Renderer::new(window)));

        let schedule = Schedule::builder().build();
        schedule.set_up(&mut world);

        Self {
            world,
            resources,
            schedule,
        }
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world, &mut self.resources);
    }

    /// Renders a frame and returns false on exit.
    pub fn render(&mut self) -> bool {
        match sparsey::run_local(&self.world, &self.resources, rendering_sys) {
            Ok(()) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => {
                let size = { self.resources.borrow::<Renderer>().size };
                self.resources.insert(Some(size));
                sparsey::run_local(&mut self.world, &mut self.resources, resize_sys);
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => return false,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        true
    }

    // Handles window events and returns false when CloseRequested is detected.
    pub fn handle_events(&mut self, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    return false;
                }
                WindowEvent::Resized(physical_size) => {
                    self.resources.insert(Some(*physical_size));
                    sparsey::run_local(&mut self.world, &mut self.resources, resize_sys);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    self.resources.insert(Some(**new_inner_size));
                    sparsey::run_local(&mut self.world, &mut self.resources, resize_sys);
                }
                _ => {}
            },
            _ => {}
        }

        true
    }
}

pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Landmark")
        .build(&event_loop)
        .expect("Failed to create a window");

    let game = Game::new(&window);

    game_loop(
        event_loop,
        window,
        game,
        240,
        0.1,
        |g| {
            g.game.update();
        },
        |g| {
            if !g.game.render() {
                g.exit();
            }
        },
        |g, event| {
            if !g.game.handle_events(event) {
                g.exit();
            }
        },
    );
}
