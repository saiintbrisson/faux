use std::io::Result;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::frame::{FrameDecoder, FrameEncoder};

#[derive(Clone, Debug)]
pub struct Response {
    pub port: u16,
}

impl FrameDecoder for Response {
    fn decode(mut src: Bytes) -> Result<Self> {
        Ok(Self {
            port: src.get_u16(),
        })
    }
}

impl FrameEncoder for Response {
    fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        Ok(dst.put_u16(self.port))
    }
}
