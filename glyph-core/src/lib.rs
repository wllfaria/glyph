#![allow(dead_code)]

mod buffer_manager;
pub mod config;
pub mod error;
pub mod event_loop;
pub mod geometry;
pub mod key_mapper;
pub mod renderer;
pub mod startup_options;
pub mod view_manager;

use std::fmt::Debug;

use crate::buffer_manager::{BufferId, BufferManager};
use crate::config::Config;
use crate::error::Result;
use crate::event_loop::EventLoop;
use crate::key_mapper::{KeyMapper, KeyMapperKind};
use crate::renderer::{RenderContext, Renderer};
use crate::startup_options::StartupOptions;
use crate::view_manager::ViewManager;

#[derive(Debug)]
pub struct Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    renderer: R,
    event_loop: E,
    key_mapper: KeyMapperKind,
    views: ViewManager,
    buffers: BufferManager,
    config: Config,
}

impl<E, R> Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    pub fn new(
        config: Config,
        event_loop: E,
        renderer: R,
        key_mapper: impl Into<KeyMapperKind>,
        options: StartupOptions,
    ) -> Result<Self> {
        let mut buffers = BufferManager::new();
        let size = renderer.get_size()?;

        if !options.files.is_empty() {
            for file in options.files.iter() {
                buffers.load_buffer(file)?;
            }
        }

        if options.files.is_empty() {
            buffers.load_startup_buffer(size)?;
        }

        // When the editor starts, it is guaranteed to have at least one buffer. Which will either
        // be a scratch buffer with a welcome message, the first user specified file or the
        // directory view.
        let views = ViewManager::new(BufferId::new(0), size);

        Ok(Self {
            views,
            config,
            buffers,
            renderer,
            event_loop,
            key_mapper: key_mapper.into(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.renderer.setup()?;

        let mut i = 0;
        loop {
            let event = self.event_loop.maybe_event()?;
            let _command = self.key_mapper.parse_event(event);

            self.render_step()?;

            std::thread::sleep(std::time::Duration::from_millis(16));

            i += 1;
            if i == 600 {
                break;
            }
        }

        self.renderer.shutdown()?;

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
