use std::io;

use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct RawCodec;

impl Decoder for RawCodec {
    type Error = io::Error;
    type Item = Bytes;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        if len != 0 {
            let b = src.split_to(len);

            return Ok(Some(b.freeze()));
        }

        Ok(None)
    }
}

impl Encoder<Bytes> for RawCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(&item);

        Ok(())
    }
}
