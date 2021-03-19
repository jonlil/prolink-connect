#[allow(unused_imports)]

use nom_derive::Nom;

use nom::IResult;
use nom::bytes::complete::tag;
use std::net::{Ipv4Addr, UdpSocket};
use nom::combinator::map_res;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq, Nom)]
struct MacAddr {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
}

#[allow(dead_code)]
const UDP_MAGIC: [u8; 10] = [0x51, 0x73, 0x70, 0x74, 0x31, 0x57, 0x6d, 0x4a, 0x4f, 0x4c];

#[derive(Debug, PartialEq)]
pub struct UdpMagic;

impl UdpMagic {
    pub fn decode(input: &[u8]) -> IResult<&[u8], UdpMagic> {
        let (input, _) = tag(UDP_MAGIC)(input)?;
        Ok((input, UdpMagic))
    }
}

#[derive(Debug, PartialEq, Eq, Nom)]
#[nom(Selector = "u8")]
pub enum MessageType {
    #[nom(Selector = "10")]
    Hello,
    #[nom(Selector="4")]
    Number,
    #[nom(Selector="0")]
    Mac,
    #[nom(Selector="2")]
    Ip,
    #[nom(Selector="6")]
    Status(Status),
    #[nom(Selector="8")]
    Change,
}

fn ip_address_parser(input: &[u8]) -> IResult<&[u8], Ipv4Addr> {
    map_res(
        nom::number::complete::be_u32,
        std::net::Ipv4Addr::try_from,
    )(input)
}

#[derive(Debug, PartialEq, Eq, Nom)]
pub struct Status {
    player_number: u8,
    #[nom(SkipBefore(1))]
    mac_address: MacAddr,
    #[nom(Parse(ip_address_parser))]
    ip_addr: Ipv4Addr,
    device_count: u8,
}

#[derive(Debug, PartialEq, Eq, Nom)]
#[repr(u8)]
pub enum DeviceType {
    Djm = 1,
    Cdj = 2,
    Rekordbox = 3,
}

#[derive(Debug, PartialEq, Eq, Nom)]
#[repr(u8)]
pub enum MessageSubType {
    Hello = 0x25,
    Number = 0x26,
    Mac = 0x2c,
    Ip = 0x32,
    Status = 0x36,
    Change = 0x29,
    StatusMixer = 0x00,
}

fn model_name_parser(input: &[u8]) -> IResult<&[u8], String> {
     match map_res(
        nom::bytes::complete::take(20u8),
        std::str::from_utf8,
    )(input) {
        Ok((input, data)) => {
            Ok((input, data.trim_end_matches('\u{0}').to_string()))
        },
        Err(err) => Err(err),
    }
}

/// An structure containing an enum
#[derive(Debug, PartialEq, Nom)]
pub struct KeepAliveMessage {
    pub msg_type: u8,
    #[nom(SkipBefore(1), Parse(model_name_parser))]
    pub model_name: String,
    #[nom(SkipBefore(1))]
    pub device_type: DeviceType,
    #[nom(SkipBefore(1))]
    pub msg_sub_type: MessageSubType,
    #[nom(Parse = "{ |i| MessageType::parse(i, msg_type) }")]
    pub msg_value: MessageType,
}

#[derive(Debug)]
pub enum Error {
    ParseError,
    MissingHeaderError,
}

pub fn process_keep_alive_message(input: &[u8]) -> Result<KeepAliveMessage, Error> {
    match UdpMagic::decode(&input) {
        Ok((input, _)) => {
            match KeepAliveMessage::parse(&input) {
                Ok((_, message)) => Ok(message),
                Err(_) => Err(Error::ParseError),
            }
        },
        Err(_) => Err(Error::MissingHeaderError),
    }
}

#[test]
fn parse_status_package() {
    let input: &[u8] = &status_package()[10..];

    assert_eq!(KeepAliveMessage::parse(&input), Ok((
        &input[39..],
        KeepAliveMessage {
            msg_type: 6,
            model_name: "XDJ-700".to_string(),
            device_type: DeviceType::Cdj,
            msg_sub_type: MessageSubType::Status,
            msg_value: MessageType::Status(Status {
                player_number: 2,
                mac_address: MacAddr {
                    a: 200,
                    b: 61,
                    c: 252,
                    d: 4,
                    e: 30,
                    f: 196,
                },
                ip_addr: Ipv4Addr::new(192, 168, 10, 78),
                device_count: 1,
            }),
        }
    )));
}

#[test]
fn it_identifies_prolink_header() {
    let data: Vec<u8> = status_package();
    assert_eq!(UdpMagic::decode(&data).unwrap().1, UdpMagic);
    assert_eq!(
        UdpMagic::decode(&vec![82]),
        Err(nom::Err::Error(
            nom::error::Error {
                input: &vec![82][..],
                code: nom::error::ErrorKind::Tag,
            },
        )),
    );
}

#[cfg(test)]
fn status_package() -> Vec<u8> {
    vec![81, 115, 112, 116, 49, 87, 109, 74, 79, 76, 6, 0, 88, 68, 74, 45, 55, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 54, 2, 2, 200, 61, 252, 4, 30, 196, 192, 168, 10, 78, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}
