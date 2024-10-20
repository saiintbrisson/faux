use std::io::{self, Error, ErrorKind, Result};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::frame::{FrameDecoder, FrameEncoder};

use self::{handshake::Handshake, response::Response};

pub mod handshake;
pub mod response;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum BridgeProtocol {
    TCP = 0x00,
    UDP = 0x01,
}

impl FrameDecoder for BridgeProtocol {
    fn decode(mut src: Bytes) -> Result<Self> {
        Ok(match src.get_u8() {
            0 => BridgeProtocol::TCP,
            1 => BridgeProtocol::UDP,
            res => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid network protocol {}", res),
            ))?,
        })
    }
}

impl FrameEncoder for BridgeProtocol {
    fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        Ok(dst.put_u8(*self as u8))
    }
}

#[derive(Clone, Debug)]
pub enum FauxPacket {
    Handshake(Handshake),
    Response(Response),
}

impl FauxPacket {
    pub fn id(&self) -> u8 {
        match self {
            FauxPacket::Handshake(_) => 0x00,
            FauxPacket::Response(_) => 0x01,
        }
    }
}

impl FrameDecoder for FauxPacket {
    fn decode(mut src: Bytes) -> Result<Self> {
        Ok(match src.get_u8() {
            0x00 => FauxPacket::Handshake(Handshake::decode(src)?),
            0x01 => FauxPacket::Response(Response::decode(src)?),
            id => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("packet id {} is unknown", id),
                ))
            }
        })
    }
}

impl FrameEncoder for FauxPacket {
    fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u8(self.id());
        match self {
            FauxPacket::Handshake(handshake) => handshake.encode(dst),
            FauxPacket::Response(response) => response.encode(dst),
        }
    }
}
