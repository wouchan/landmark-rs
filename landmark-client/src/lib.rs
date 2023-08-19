mod camera;
mod color;
mod game_map;
mod input;
mod mesher;
mod model;
mod rendererer;
mod texture;
mod transform;

use camera::update_camera_sys;
use game_loop::{
    game_loop,
    winit::{
        event::{DeviceEvent, Event, WindowEvent},
        event_loop::EventLoop,
        window::{CursorGrabMode, Fullscreen, Window, WindowBuilder},
    },
};
use game_map::{mesh_missing_chunks_sys, GameMap};
use mesher::{chunk_mesher_loop, MeshRequestsSender};
use model::{update_models_sys, ConstructedModelsReceiver};
use shipyard::*;

use input::*;
use rendererer::*;

#[derive(Debug)]
struct Game {
    pub world: World,
}

impl Game {
    pub fn init(window: &Window) -> Self {
        let mut world = World::new();

        let (renderer, camera) = pollster::block_on(Renderer::init(window));

        let game_map = GameMap::new_test(&mut world);

        world.add_unique(renderer);
        world.add_unique(camera);
        world.add_unique(game_map);
        world.add_unique(InputState::default());

        // TODO: simplify meshing
        let (mesh_requests_sender, chunk_mesh_requests_receiver) = MeshRequestsSender::init();
        let (constructed_models_receiver, constructed_chunk_sender) =
            ConstructedModelsReceiver::init();

        world.add_unique_non_sync(mesh_requests_sender);
        world.add_unique_non_sync(constructed_models_receiver);
        std::thread::spawn(|| {
            chunk_mesher_loop(chunk_mesh_requests_receiver, constructed_chunk_sender)
        });

        Workload::new("update")
            .with_system(move_player_sys)
            .with_system(mesh_missing_chunks_sys)
            .add_to_world(&world)
            .unwrap();

        Workload::new("render")
            .with_system(update_camera_sys)
            .with_system(update_models_sys)
            .add_to_world(&world)
            .unwrap();

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.run_workload("update").unwrap();
    }

    /// Renders a frame and returns false on exit.
    pub fn render(&mut self) -> bool {
        self.world.run_workload("render").unwrap();

        match self.world.run(rendering_sys) {
            Ok(()) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => {
                let size = { self.world.borrow::<UniqueView<Renderer>>().unwrap().size };
                self.world.run_with_data(resize_sys, size);
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => return false,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        true
    }

    // Handles window events and returns false when CloseRequested is detected.
    pub fn handle_events(&mut self, window: &Window, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    return false;
                }
                WindowEvent::Resized(physical_size) => {
                    self.world.run_with_data(resize_sys, *physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.world.run_with_data(resize_sys, **new_inner_size);
                }
                WindowEvent::CursorEntered { .. } => {
                    self.world
                        .borrow::<UniqueViewMut<InputState>>()
                        .unwrap()
                        .cursor_in_window = true;
                }
                WindowEvent::CursorLeft { .. } => {
                    self.world
                        .borrow::<UniqueViewMut<InputState>>()
                        .unwrap()
                        .cursor_in_window = false;
                }
                WindowEvent::MouseInput { button, .. } => {
                    self.world.run_with_data(mouse_button_sys, button)
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match *event {
                DeviceEvent::MouseMotion { delta } => {
                    self.world.run_with_data(mouse_input_sys, delta)
                }
                DeviceEvent::Key(event) => self.world.run_with_data(keyboard_input_sys, event),
                _ => {}
            },
            _ => {}
        }

        // Process requests to change the window state.
        let input_state = self.world.borrow::<UniqueView<InputState>>().unwrap();

        // Check if cursor should be captured.
        if input_state.cursor_captured {
            window.set_cursor_visible(false);
            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        } else {
            window.set_cursor_visible(true);
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
        }

        // Check if fullscreen should be enabled.
        if input_state.fullscreen {
            if let None = window.fullscreen() {
                let monitor = window
                    .current_monitor()
                    .expect("Could not get a reference to the current monitor");

                let video_mode = monitor
                    .video_modes()
                    .nth(0)
                    .expect("Could not get a preferred video mode of the current monitor");

                window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
            }
        } else {
            if let Some(_) = window.fullscreen() {
                window.set_fullscreen(None);
            }
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

    let game = Game::init(&window);

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
            if !g.game.handle_events(&g.window, event) {
                g.exit();
            }
        },
    );
}
