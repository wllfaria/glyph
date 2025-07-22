#![allow(dead_code)]

mod buffer_manager;
mod command_handler;
pub mod config;
pub mod error;
pub mod event_loop;
pub mod geometry;
pub mod key_mapper;
pub mod renderer;
pub mod startup_options;
pub mod view_manager;

use std::fmt::Debug;

use command_handler::buffer_command_handler::BufferCommandHandler;
use command_handler::{CommandContext, CommandHandler, CommandHandlerChain};

use crate::buffer_manager::{BufferId, BufferManager};
use crate::config::Config;
use crate::error::Result;
use crate::event_loop::EventLoop;
use crate::key_mapper::{Keymapper, KeymapperKind};
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
    key_mapper: KeymapperKind,
    views: ViewManager,
    buffers: BufferManager,
    config: Config,
    command_handler: CommandHandlerChain,
    should_quit: bool,
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
        key_mapper: impl Into<KeymapperKind>,
        options: StartupOptions,
    ) -> Result<Self> {
        let key_mapper = key_mapper.into();
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

        let mut command_handler = CommandHandlerChain::default();
        command_handler.add_handler(BufferCommandHandler::default());

        Ok(Self {
            views,
            config,
            buffers,
            renderer,
            event_loop,
            key_mapper,
            command_handler,
            should_quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.renderer.setup()?;

        while !self.should_quit {
            let event = self.event_loop.maybe_event()?;

            if let Some(commands) = self.key_mapper.parse_event(event) {
                self.command_handler.handle_commands(&mut CommandContext {
                    commands: &commands,
                    buffers: &mut self.buffers.buffers,
                    views: &mut self.views,
                    should_quit: &mut self.should_quit,
                });
            }

            self.render_step()?;
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
            views: &self.views,
            buffers: &buffers,
            layout: &self.views.layout,
        })?;

        Ok(())
    }
}
