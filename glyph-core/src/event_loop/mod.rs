pub mod error;
pub mod event;

pub use event::Event;

pub trait EventLoop {
    fn maybe_event(&self) -> error::Result<Option<event::Event>>;
}