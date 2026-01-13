//! Mock Message Transmitter for CCTP Forwarder Prototype
//!
//! This is a simplified implementation for testing purposes.
//! It simulates the CCTP MessageTransmitter's receive_message functionality.

#![no_std]
mod contract;
mod message;

pub use contract::*;
pub use message::*;
