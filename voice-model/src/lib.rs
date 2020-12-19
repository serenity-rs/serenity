//! Mappings of objects received from Discord's voice gateway API, with implementations
//! for (de)serialisation.
#![deny(broken_intra_doc_links)]

mod close_code;
pub mod constants;
mod event;
pub mod id;
mod opcode;
pub mod payload;
mod protocol_data;
mod speaking_state;
mod util;

pub use self::{
    close_code::CloseCode,
    event::Event,
    opcode::OpCode,
    protocol_data::ProtocolData,
    speaking_state::SpeakingState,
};

pub use enum_primitive::FromPrimitive;
