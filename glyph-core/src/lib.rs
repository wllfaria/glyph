#![allow(dead_code)]

pub mod buffer_manager;
pub mod command_handler;
pub mod config;
pub mod cursor;
pub mod error;
pub mod event_loop;
pub mod geometry;
pub mod key_mapper;
pub mod renderer;
pub mod startup_options;
pub mod text_object;
pub mod view_manager;

use std::fmt::Debug;

use command_handler::{CommandContext, CommandHandler, CommandHandlerChain};

use crate::buffer_manager::{BufferId, BufferManager};
use crate::config::Config;
use crate::error::Result;
use crate::event_loop::EventLoop;
use crate::key_mapper::Keymapper;
use crate::renderer::{RenderContext, Renderer};
use crate::startup_options::StartupOptions;
use crate::view_manager::ViewManager;

pub enum EditorKind {
    Modal,
    NonModal,
}

#[derive(Debug)]
pub struct Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    renderer: R,
    event_loop: E,
    config: Config,
    should_quit: bool,
    views: ViewManager,
    buffers: BufferManager,
    key_mapper: Box<dyn Keymapper>,
    command_handler_chain: CommandHandlerChain,
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
        key_mapper: Box<dyn Keymapper>,
        command_handler: Box<dyn CommandHandler>,
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

        let mut command_handler_chain = CommandHandlerChain::default();
        command_handler_chain.add_handler(command_handler);

        Ok(Self {
            views,
            config,
            buffers,
            renderer,
            event_loop,
            key_mapper,
            should_quit: false,
            command_handler_chain,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.renderer.setup()?;

        while !self.should_quit {
            let event = self.event_loop.maybe_event()?;

            if let Some(resolved_keymap) = self.key_mapper.parse_event(event) {
                self.command_handler_chain
                    .handle_commands(&mut CommandContext {
                        resolved_keymap: &resolved_keymap,
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
