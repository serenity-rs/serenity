//! Mappings of objects received from Discord's voice gateway API, with implementations for
//! (de)serialisation.
#![deny(rustdoc::broken_intra_doc_links)]

mod close_code;
pub mod constants;
mod event;
pub mod id;
mod opcode;
pub mod payload;
mod protocol_data;
mod speaking_state;
mod util;

pub use num_traits::FromPrimitive;

pub use self::close_code::CloseCode;
pub use self::event::Event;
pub use self::opcode::Opcode;
pub use self::protocol_data::ProtocolData;
pub use self::speaking_state::SpeakingState;
