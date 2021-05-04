use anyhow::*;
use cgmath::EuclideanSpace;
use winit::event::*;

use crate::{camera::*, chunk::*, coordinate::*, crosshair::*, framerate::Framerate, overlay_info::*, player::*, renderer::*, world::*, Config};

#[allow(dead_code)]
pub struct Engine {
    pub renderer: Renderer,
    framerate: Framerate,

    camera: Camera,
    coordinate: Coordinate,
    overlay_info: OverlayInfo,
    crosshair: Crosshair,
    player: Player,
    world: World,

    logger: slog::Logger,
    config: Config,
}

impl Engine {
    pub fn new(logger: slog::Logger, config: Config, renderer: Renderer) -> Result<Self> {
        let camera = Camera::new(&renderer, (0.0, SEA_LEVEL as f32 + 5.0, 0.0), cgmath::Deg(90.0), cgmath::Deg(-20.0));
        let coordinate = Coordinate::new(
            &renderer.device,
            &renderer.sc_desc,
            &camera.uniform_bind_group_layout,
            config.display_coordinates,
        )?;

        let overlay_info = OverlayInfo::new(&renderer)?;
        let player = Player::new(&camera);
        let crosshair = Crosshair::new(&renderer);

        info!(logger, "Generating World...");
        let world = World::new(logger.clone(), config, &renderer, &camera.uniform_bind_group_layout)?;
        info!(logger, "World Generated!");

        Ok(Self {
            renderer,
            framerate: Framerate::new(),

            camera,
            coordinate,
            overlay_info,
            crosshair,
            player,
            world,

            logger,
            config,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
        self.camera.resize(new_size);
        self.overlay_info.resize(new_size);
        self.crosshair.resize(&self.renderer.queue, new_size);
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera.update(&self.renderer.queue, dt);
        self.world.set_center(&self.renderer.queue, self.camera.position.to_vec());
        self.overlay_info
            .update(&self.renderer.queue, self.framerate.current_fps, self.camera.position)
            .expect("Overlay update broke.");
        self.player.update2(&self.camera, &mut self.world);
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        self.player.input(event, &self.renderer.queue, &mut self.world);
        self.camera.input(event);
    }

    pub fn input_keyboard(&mut self, event: &WindowEvent) -> bool {
        self.camera.input_keyboard(event)
    }

    pub fn render(&mut self) -> Result<()> {
        self.framerate.fps();
        self.renderer
            .render(&self.camera, &self.world, &self.coordinate, &self.overlay_info, &self.crosshair)?;

        Ok(())
    }
}
