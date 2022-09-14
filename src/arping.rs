use pnet::packet::Packet;
use pnet::packet::arp::{Arp, ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherType, EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::datalink::Channel::{Ethernet};
use pnet::datalink;
use pnet::util::MacAddr;
use std::net::Ipv4Addr;

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
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };
    /*
       let interface = interfaces.into_iter()
                                 .filter(interface_names_match)
                                 .next()
                                 .unwrap();
    */
    

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    let interface_mac = interface.mac.ok_or(MagicError::InvalidMac)?;

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(interface_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(interface_mac);
    arp_packet.set_sender_proto_addr(Ipv4Addr::new(192, 168, 1, 191));
    arp_packet.set_target_hw_addr(mac);
    arp_packet.set_target_proto_addr(Ipv4Addr::BROADCAST);

    ethernet_packet.set_payload(arp_packet.packet());

    tx.send_to(ethernet_packet.packet(), None).unwrap().unwrap();

    loop {
        let mut buf: [u8; 1600] = [0u8; 1600];

        let packet = rx.next()?;

        match EthernetPacket::new(packet) {
            Some(ether_packet) => {
                if ether_packet.get_ethertype() == EtherTypes::Arp {
                    let response = ArpPacket::new(ether_packet.payload()).expect("could not unpack arp");

                    println!("ARP from: {}/{}", response.get_sender_hw_addr(), response.get_sender_proto_addr());
                }
            }
            None => ()
        }
    }

    Ok(())
}


