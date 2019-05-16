use crate::primitives::hash::Hash;
use hex::encode;

//Our version of Buffer that is implemented in bio - > https://github.com/bcoin-org/bufio
#[derive(Default, Debug, PartialEq)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn new() -> Self {
        Buffer::default()
    }

    //Write u32 in Little Endian format
    //Possibly return the amount of data written //TODO - see if needed anywhere.
    pub fn write_u32(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_u64(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_hash(&mut self, hash: Hash) {
        self.0.extend(&hash.to_array());
    }

    //Return Hex string of the buffer.
    pub fn to_hex(&self) -> String {
        encode(&self.0)
    }

    //Return Hex string of the buffer, Consumes the Hex
    pub fn into_hex(self) -> String {
        encode(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u32() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));
    }

    #[test]
    fn test_write_hash() {
        let hash = Hash::from("bb42edce1895f9a969e81d7371ec113a0966e5d55035a84f87ca098e4f0a1a86");

        let mut buffer = Buffer::new();

        buffer.write_hash(hash);

        dbg!(buffer);
    }

    #[test]
    fn test_to_hex() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));

        let hex = buffer.to_hex();

        assert_eq!(hex, "15cd5b07")
    }

    #[test]
    fn test_into_hex() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));

        let hex = buffer.into_hex();

        assert_eq!(hex, "15cd5b07")
    }

}