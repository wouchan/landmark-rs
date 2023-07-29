use game_loop::{
    game_loop,
    winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    },
};
use sparsey::prelude::*;

#[derive(Debug)]
struct Game {
    pub world: World,
    pub resources: Resources,
    pub update_schedule: Schedule,
    pub render_schedule: Schedule,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::default();
        let resources = Resources::default();

        let update_schedule = Schedule::builder().build();
        update_schedule.set_up(&mut world);

        let render_schedule = Schedule::builder().build();
        render_schedule.set_up(&mut world);

        Self {
            world,
            resources,
            update_schedule,
            render_schedule,
        }
    }

    pub fn update(&mut self) {
        self.update_schedule
            .run(&mut self.world, &mut self.resources);
    }

    pub fn render(&mut self) {
        self.render_schedule
            .run(&mut self.world, &mut self.resources);
    }

    // A very simple handler that returns false when CloseRequested is detected.
    pub fn handle_events(&self, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    return false;
                }
                _ => {}
            },
            _ => {}
        }

        true
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Landmark")
        .build(&event_loop)
        .expect("Failed to create a window");

    let game = Game::new();

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
            g.game.render();
        },
        |g, event| {
            if !g.game.handle_events(event) {
                g.exit();
            }
        },
    );
}
