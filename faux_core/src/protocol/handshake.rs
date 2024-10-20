use std::io::Result;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    frame::{FrameDecoder, FrameEncoder},
    io::{BufExt, BufMutExt},
};

use super::BridgeProtocol;

#[derive(Clone, Debug)]
pub struct Handshake {
    pub version: Bytes,
    pub os_name: Bytes,
    pub protocol: BridgeProtocol,
    pub host_port: u16,
    pub preferred_port: u16,
}

impl Handshake {
    pub fn new(version: Bytes, os_name: Bytes, protocol: BridgeProtocol, host_port: u16, preferred_port: u16) -> Self {
        Self {
            version,
            os_name,
            protocol,
            host_port,
			preferred_port
        }
    }
}

impl FrameDecoder for Handshake {
    fn decode(mut src: Bytes) -> Result<Self> {
        Ok(Self {
            version: src.get_string(),
            os_name: src.get_string(),
            protocol: {
                let protocol = BridgeProtocol::decode(src.clone())?;
                src.advance(1);
                protocol
            },
            host_port: src.get_u16(),
			preferred_port: src.get_u16(),
        })
    }
}

impl FrameEncoder for Handshake {
    fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        dst.put_string(&self.version);
        dst.put_string(&self.os_name);
        self.protocol.encode(dst)?;
		dst.put_u16(self.host_port);
        Ok(dst.put_u16(self.preferred_port))
    }
}
