use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait BufExt {
    fn get_string(&mut self) -> Bytes;
}

impl BufExt for Bytes {
    fn get_string(&mut self) -> Bytes {
        let len = self.get_u32() as usize;
        self.split_to(len)
    }
}

pub trait BufMutExt {
    fn put_string<T>(&mut self, string: T)
    where
        T: AsRef<[u8]>;
}

impl BufMutExt for BytesMut {
    fn put_string<T>(&mut self, string: T)
    where
        T: AsRef<[u8]>,
    {
        let string = string.as_ref();
        self.put_u32(string.len() as u32);
        self.extend_from_slice(string)
    }
}
