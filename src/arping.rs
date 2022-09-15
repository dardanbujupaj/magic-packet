use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::arp::{
    Arp, ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket, MutableArpPacket,
};
use pnet::packet::ethernet::{EtherType, EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::icmp;
use pnet::packet::icmp::echo_reply::EchoReplyPacket;
use pnet::packet::icmp::echo_request::{IcmpCodes, MutableEchoRequestPacket, EchoRequestPacket};
use pnet::packet::icmp::{
    echo_reply, IcmpCode, IcmpPacket, IcmpType, IcmpTypes, MutableIcmpPacket,
};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{self, Ipv4};
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet};
use pnet::packet::Packet;
use pnet::util::MacAddr;
use std::convert::TryInto;
use std::net::Ipv4Addr;
use std::str::FromStr;

use crate::magic::MagicError;

pub fn arping(mac: MacAddr) -> Result<(), MagicError> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .iter()
        .filter(|interface| interface.ips.iter().any(|ip| ip.is_ipv4()))
        .nth(1)
        .ok_or(MagicError::NoInterfaceFound)?;

    println!("{interface}");

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let interface_mac = interface.mac.ok_or(MagicError::InvalidMac)?;

    let mut echo_buffer = vec![0u8; EchoRequestPacket::minimum_packet_size()];
    let mut ipv4_buffer = vec![0u8; Ipv4Packet::minimum_packet_size() + echo_buffer.len()];
    let ipv4_buffer_len = ipv4_buffer.len() as u16;
    let mut ethernet_buffer = vec![0u8; EthernetPacket::minimum_packet_size() + ipv4_buffer.len()];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(mac);
    ethernet_packet.set_source(interface_mac);
    ethernet_packet.set_ethertype(EtherTypes::Ipv4);

    println!("{}", ipv4_buffer.len());
    let mut ipv4_packet = MutableIpv4Packet::new(&mut ipv4_buffer).unwrap();


    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    ipv4_packet.set_flags(0);

    let addr_string = interface.ips.iter().map(|n| n.ip()).find(|ip| ip.is_ipv4()).unwrap().to_string();
    let source = Ipv4Addr::from_str(addr_string.as_str()).unwrap();

    ipv4_packet.set_source(source);
    ipv4_packet.set_destination(Ipv4Addr::BROADCAST);
    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ipv4_packet.set_total_length(ipv4_buffer_len);
    ipv4_packet.set_ttl(64);

    ipv4_packet.set_checksum(ipv4::checksum(&ipv4_packet.to_immutable()));

    let mut echo_packet = MutableEchoRequestPacket::new(&mut echo_buffer).unwrap();

    echo_packet.set_icmp_type(IcmpTypes::EchoRequest);
    echo_packet.set_checksum(icmp::checksum(&IcmpPacket::new(echo_packet.packet()).unwrap()));
    println!("{:?}", echo_packet);

    ipv4_packet.set_payload(echo_packet.packet());
    ethernet_packet.set_payload(ipv4_packet.packet());

    tx.send_to(ethernet_packet.packet(), None).unwrap().unwrap();

    loop {
        let mut buf: [u8; 1600] = [0u8; 1600];

        let packet = rx.next()?;

        match EthernetPacket::new(packet) {
            Some(ether_packet) => {
                if ether_packet.get_ethertype() == EtherTypes::Ipv4 {
                    let header = Ipv4Packet::new(ether_packet.payload());
                    if let Some(header) = header {
                        if header.get_next_level_protocol() == IpNextHeaderProtocols::Icmp {
                            if let Some(echo_packet) = EchoReplyPacket::new(packet) {
                                println!("{:?} from {}", echo_packet.get_icmp_type(), ether_packet.get_source());
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }

    Ok(())
}
