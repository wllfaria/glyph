pub mod error;
pub mod event;

pub trait EventLoop {
    fn maybe_event(&self) -> error::Result<Option<event::Event>>;
}
