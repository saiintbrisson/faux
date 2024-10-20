use std::io::{self, Result};

use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

use crate::protocol::FauxPacket;

pub struct Codec {
    delimiter: LengthDelimitedCodec,
}

impl Codec {
    pub fn new() -> Self {
        Self {
            delimiter: Default::default(),
        }
    }
}

impl Decoder for Codec {
    type Item = FauxPacket;

    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let buf = match self.delimiter.decode(src)? {
            Some(buf) => buf,
            None => return Ok(None),
        };

        if buf.len() < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buf len is less than 1",
            ));
        }

        Ok(Some(FauxPacket::decode(buf.freeze())?))
    }
}

impl<T> Encoder<T> for Codec
where
    T: FrameEncoder,
{
    type Error = io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<()> {
        let mut data = BytesMut::with_capacity(128);
        item.encode(&mut data)?;

        self.delimiter.encode(data.freeze(), dst)
    }
}

pub trait FrameDecoder
where
    Self: Sized,
{
    fn decode(src: Bytes) -> Result<Self>;
}

pub trait FrameEncoder {
    fn encode(&self, dst: &mut BytesMut) -> Result<()>;
}
