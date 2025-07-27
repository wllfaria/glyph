#![allow(dead_code)]

pub mod buffer_manager;
pub mod command_handler;
pub mod config;
pub mod cursor;
pub mod editing_plugin;
pub mod error;
pub mod event_loop;
pub mod geometry;
pub mod key_mapper;
pub mod renderer;
pub mod startup_options;
pub mod status_provider;
pub mod text_object;
pub mod view_manager;

use std::fmt::Debug;
use std::sync::Arc;

use command_handler::{CommandContext, CommandHandler, CommandHandlerChain};

use crate::buffer_manager::{BufferId, BufferManager};
use crate::config::Config;
use crate::editing_plugin::EditingPlugin;
use crate::error::Result;
use crate::event_loop::EventLoop;
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
    config: Arc<Config>,
    should_quit: bool,
    views: ViewManager,
    buffers: BufferManager,
    editing_plugin: Box<dyn EditingPlugin>,
    command_handler_chain: CommandHandlerChain,
}

impl<E, R> Glyph<E, R>
where
    E: EventLoop + Debug,
    R: Renderer + Debug,
{
    pub fn new(
        config: Arc<Config>,
        event_loop: E,
        renderer: R,
        editing_plugin: Box<dyn EditingPlugin>,
        options: StartupOptions,
    ) -> Result<Self> {
        let mut buffers = BufferManager::new();
        let size = renderer.get_size(editing_plugin.dock_height())?;

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
        let views = ViewManager::new(config.clone(), BufferId::new(0), size);

        let command_handler = editing_plugin.create_command_handler();
        let file_command_handler = command_handler::FileCommandHandler;
        let mut command_handler_chain = CommandHandlerChain::default();
        command_handler_chain.add_handler(Box::new(file_command_handler));
        command_handler_chain.add_handler(command_handler);

        Ok(Self {
            views,
            config,
            buffers,
            renderer,
            event_loop,
            editing_plugin,
            should_quit: false,
            command_handler_chain,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.renderer.setup()?;

        while !self.should_quit {
            let event = self.event_loop.maybe_event()?;

            if let Some(resolved_keymap) = self.editing_plugin.parse_event(event) {
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
            mode: self.editing_plugin.mode(),
            views: &self.views,
            buffers: &buffers,
            layout: &self.views.layout,
            editing_plugin: self.editing_plugin.as_ref(),
        })?;

        Ok(())
    }
}
