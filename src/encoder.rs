use pancake::Vec;

pub struct Encoder {
    bytes: Vec<u8, 15>,
}

impl Encoder {
    #[inline]
    pub const fn new() -> Self {
        let bytes = Vec::new();

        Self { bytes }
    }

    #[inline]
    pub const unsafe fn write_u8(&mut self, value: u8) {
        self.bytes.push_unchecked(value);
    }

    #[inline]
    pub const unsafe fn write_i32(&mut self, value: i32) {
        let bytes = value.to_le_bytes();

        self.write_bytes(&bytes);
    }

    #[inline]
    pub const unsafe fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice_unchecked(bytes);
    }

    #[inline]
    pub const fn into_vec(self) -> Vec<u8, 15> {
        self.bytes
    }
}
