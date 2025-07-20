mod buffer_manager;
pub mod error;
pub mod event_loop;
pub mod geometry;
pub mod renderer;
pub mod startup_options;
pub mod view_manager;

use std::fmt::Debug;

use crate::buffer_manager::{BufferId, BufferManager};
use crate::error::Result;
use crate::event_loop::EventLoop;
use crate::renderer::{RenderContext, Renderer};
use crate::startup_options::StartupOptions;
use crate::view_manager::ViewManager;

#[derive(Debug)]
pub struct Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    event_loop: E,
    renderer: R,
    views: ViewManager,
    buffers: BufferManager,
}

impl<E, R> Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    pub fn new(event_loop: E, renderer: R, options: StartupOptions) -> Result<Self> {
        let mut buffers = BufferManager::new();
        let size = renderer.get_size()?;

        if !options.files.is_empty() {
            for file in options.files.iter() {
                buffers.load_buffer(file)?;
            }
        }

        if options.files.is_empty() {
            buffers.load_startup_buffer()?;
        }

        // When the editor starts, it is guaranteed to have at least one buffer. Which will either
        // be a scratch buffer with a welcome message, the first user specified file or the
        // directory view.
        let views = ViewManager::new(BufferId::new(0), size);

        Ok(Self {
            views,
            buffers,
            renderer,
            event_loop,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // self.renderer.setup()?;

        loop {
            self.render_step()?;

            if true {
                break;
            }
        }

        // self.renderer.shutdown()?;

        Ok(())
    }

    fn render_step(&mut self) -> Result<()> {
        let views = self.views.get_visible();
        let buffers = views
            .iter()
            .map(|v| self.buffers.get(v.buffer_id).unwrap())
            .collect::<Vec<_>>();

        self.renderer.render(&mut RenderContext {
            views: &views,
            buffers: &buffers,
            layout: &self.views.layout,
        })?;

        Ok(())
    }
}
