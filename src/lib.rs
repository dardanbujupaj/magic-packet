use std::error::Error;
use std::fmt::Display;
use std::net::{Ipv4Addr, UdpSocket};
use std::num::ParseIntError;

#[derive(Debug)]
pub struct MagicPacket([u8; 102]);

type MacAddress = [u8; 6];

#[derive(Debug)]
pub enum MagicError {
    ParseInt(ParseIntError),
    InvalidMac,
    IoError(std::io::Error),
}

impl Error for MagicError {}
impl Display for MagicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Magic Error")
    }
}

impl From<ParseIntError> for MagicError {
    fn from(e: ParseIntError) -> Self {
        MagicError::ParseInt(e)
    }
}

impl From<std::io::Error> for MagicError {
    fn from(e: std::io::Error) -> Self {
        MagicError::IoError(e)
    }
}

impl From<Vec<u8>> for MagicError {
    fn from(_e: Vec<u8>) -> Self {
        MagicError::InvalidMac
    }
}

const PREFIX: [u8; 6] = [0xFF; 6];

impl MagicPacket {
    pub fn new(mac: MacAddress) -> Self {
        MagicPacket(
            [
                PREFIX, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac, mac,
                mac,
            ]
            .concat()
            .try_into()
            .unwrap(),
        )
    }

    pub fn send(&self) -> Result<(), MagicError> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.set_broadcast(true)?;

        socket.send_to(&self.0, (Ipv4Addr::BROADCAST, 9))?;

        Ok(())
    }
}

impl From<[u8; 102]> for MagicPacket {
    fn from(packet: [u8; 102]) -> Self {
        MagicPacket(packet)
    }
}

impl TryFrom<&str> for MagicPacket {
    type Error = MagicError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mac: [u8; 6] = value
            .split(':')
            .map(|e| u8::from_str_radix(e, 16))
            .collect::<Result<Vec<u8>, _>>()?
            .try_into()?;

        Ok(MagicPacket::new(mac))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_packet_from_string() {
        let expected_packet: [u8; 102] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01,
            0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01,
            0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x01,
            0x02, 0x03, 0x04, 0x05,
        ];

        let magic_packet: MagicPacket = "00:01:02:03:04:05".try_into().unwrap();

        assert_eq!(expected_packet, magic_packet.0);
    }

    #[test]
    fn test_invalid_segment() {
        let result: Result<MagicPacket, MagicError> = "GG:00:00:00:00:00".try_into();

        match result {
            Err(MagicError::ParseInt(_)) => (),
            r => unreachable!("Should have been MagicError::ParseInt but was {:?}", r),
        }
    }

    #[test]
    fn test_invalid_length() {
        let result: Result<MagicPacket, MagicError> = "00:00:00:00:00".try_into();

        match result {
            Err(MagicError::InvalidMac) => (),
            r => unreachable!("Should have been MagicError::InvalidMac but was {:?}", r),
        }
    }
}
