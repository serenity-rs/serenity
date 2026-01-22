//! Binary serialization/deserialization for DAVE protocol opcodes.
//!
//! Per the DAVE protocol specification, opcodes 25-30 use binary WebSocket frames
//! with the following structure:
//! - Server-to-client (25, 27, 29, 30): `[sequence_number: u16][opcode: u8][payload]`
//! - Client-to-server (26, 28): `[opcode: u8][payload]`

use crate::opcode::Opcode;
use crate::payload::*;

/// Error type for binary serialization/deserialization
#[derive(Debug)]
pub enum BinaryError {
    /// Not enough bytes to parse the message
    InsufficientData,
    /// Invalid opcode value
    InvalidOpcode(u8),
    /// Invalid operation type for proposals
    InvalidOperationType(u8),
    /// General parse error
    ParseError(String),
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "Insufficient data to parse binary message"),
            Self::InvalidOpcode(op) => write!(f, "Invalid opcode: {}", op),
            Self::InvalidOperationType(op) => write!(f, "Invalid operation type: {}", op),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for BinaryError {}

/// Parse a u16 from big-endian bytes
fn read_u16(data: &[u8]) -> Result<u16, BinaryError> {
    if data.len() < 2 {
        return Err(BinaryError::InsufficientData);
    }
    Ok(u16::from_be_bytes([data[0], data[1]]))
}

/// Deserialize a binary message from Discord (opcodes 25, 27, 29, 30)
pub fn deserialize_binary_event(data: &[u8]) -> Result<crate::Event, BinaryError> {
    if data.len() < 3 {
        return Err(BinaryError::InsufficientData);
    }

    let sequence_number = read_u16(&data[0..2])?;
    let opcode = data[2];

    // Log raw binary data for debugging (first 16 bytes)
    #[cfg(debug_assertions)]
    eprintln!(
        "[DAVE Binary] Received {} bytes: seq={}, opcode={}, data={:02X?}",
        data.len(),
        sequence_number,
        opcode,
        &data[..data.len().min(16)]
    );

    match opcode {
        25 => {
            // DaveMlsExternalSender
            let external_sender = data[3..].to_vec();
            Ok(crate::Event::DaveMlsExternalSender(DaveMlsExternalSender {
                sequence_number,
                external_sender,
            }))
        },
        27 => {
            // DaveMlsProposals
            if data.len() < 4 {
                return Err(BinaryError::InsufficientData);
            }
            let operation_type = match data[3] {
                0 => DaveMlsProposalsOperationType::Append,
                1 => DaveMlsProposalsOperationType::Revoke,
                other => return Err(BinaryError::InvalidOperationType(other)),
            };
            let proposals = data[4..].to_vec();
            Ok(crate::Event::DaveMlsProposals(DaveMlsProposals {
                sequence_number,
                operation_type,
                proposals,
            }))
        },
        29 => {
            // DaveMlsAnnounceCommitTransition
            if data.len() < 5 {
                return Err(BinaryError::InsufficientData);
            }
            let transition_id = read_u16(&data[3..5])?;
            let commit_message = data[5..].to_vec();
            Ok(crate::Event::DaveMlsAnnounceCommitTransition(DaveMlsAnnounceCommitTransition {
                sequence_number,
                transition_id,
                commit_message,
            }))
        },
        30 => {
            // DaveMlsWelcome
            if data.len() < 5 {
                return Err(BinaryError::InsufficientData);
            }
            let transition_id = read_u16(&data[3..5])?;
            let welcome = data[5..].to_vec();
            Ok(crate::Event::DaveMlsWelcome(DaveMlsWelcome {
                sequence_number,
                transition_id,
                welcome,
            }))
        },
        // Unknown opcodes: Log and skip (might be new Discord protocol extensions)
        other => {
            #[cfg(debug_assertions)]
            eprintln!(
                "[DAVE Binary] Skipping unknown opcode {}: {:02X?}",
                other,
                &data[..data.len().min(32)]
            );

            Err(BinaryError::InvalidOpcode(other))
        },
    }
}

/// Serialize a binary message to Discord (opcodes 26, 28)
pub fn serialize_binary_event(event: &crate::Event) -> Result<Vec<u8>, BinaryError> {
    match event {
        crate::Event::DaveMlsKeyPackage(payload) => {
            // Opcode 26: [opcode: u8][key_package: Vec<u8>]
            let mut data = Vec::with_capacity(1 + payload.key_package.len());
            data.push(Opcode::DaveMlsKeyPackage as u8);
            data.extend_from_slice(&payload.key_package);
            Ok(data)
        },
        crate::Event::DaveMlsCommitWelcome(payload) => {
            // Opcode 28: [opcode: u8][commit: Vec<u8>][welcome: Option<Vec<u8>>]
            // The welcome is included if present, following MLS TLS encoding for optional values
            let mut data = Vec::with_capacity(
                1 + payload.commit.len() + payload.welcome.as_ref().map_or(0, |w| w.len()),
            );
            data.push(Opcode::DaveMlsCommitWelcome as u8);
            data.extend_from_slice(&payload.commit);

            if let Some(welcome) = &payload.welcome {
                data.extend_from_slice(welcome);
            }
            Ok(data)
        },
        _ => Err(BinaryError::ParseError("Event is not a binary DAVE opcode".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_external_sender() {
        let data = vec![
            0x00, 0x01, // sequence_number = 1
            25,   // opcode = 25
            0xDE, 0xAD, 0xBE, 0xEF, // external_sender data
        ];

        let event = deserialize_binary_event(&data).unwrap();
        match event {
            crate::Event::DaveMlsExternalSender(payload) => {
                assert_eq!(payload.sequence_number, 1);
                assert_eq!(payload.external_sender, vec![0xDE, 0xAD, 0xBE, 0xEF]);
            },
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_deserialize_proposals() {
        let data = vec![
            0x00, 0x02, // sequence_number = 2
            27,   // opcode = 27
            0,    // operation_type = Append
            0xCA, 0xFE, // proposals data
        ];

        let event = deserialize_binary_event(&data).unwrap();
        match event {
            crate::Event::DaveMlsProposals(payload) => {
                assert_eq!(payload.sequence_number, 2);
                assert!(matches!(payload.operation_type, DaveMlsProposalsOperationType::Append));
                assert_eq!(payload.proposals, vec![0xCA, 0xFE]);
            },
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_serialize_key_package() {
        let event = crate::Event::DaveMlsKeyPackage(DaveMlsKeyPackage {
            key_package: vec![0xAA, 0xBB, 0xCC],
        });

        let data = serialize_binary_event(&event).unwrap();
        assert_eq!(data, vec![26, 0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_serialize_commit_welcome() {
        let event = crate::Event::DaveMlsCommitWelcome(DaveMlsCommitWelcome {
            commit: vec![0x11, 0x22],
            welcome: Some(vec![0x33, 0x44]),
        });

        let data = serialize_binary_event(&event).unwrap();
        assert_eq!(data, vec![28, 0x11, 0x22, 0x33, 0x44]);
    }
}
