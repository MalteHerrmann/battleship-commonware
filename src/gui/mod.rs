mod actor;
mod ingress;

pub use actor::GuiActor;
pub use ingress::{Log, LogType, Mailbox, Message};