use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::{Packet,MutablePacket};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::EtherTypes::Ipv6;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv6::MutableIpv6Packet;
use std::net::Ipv6Addr;
use pnet::util::MacAddr;

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    let test = env::args().nth(2).unwrap();

    let destination_mac = env::args().nth(3).unwrap().parse::<MacAddr>().unwrap();

    let source_ip = env::args().nth(4).unwrap().parse::<Ipv6Addr>().unwrap();

    let destination_ip = env::args().nth(5).unwrap().parse::<Ipv6Addr>().unwrap();

    println!("Client starting");

    let interfaces = datalink::interfaces();

    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    let interface = interfaces.into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let (mut tx, rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(itx, irx)) => (itx, irx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    if test == "11211" {
        let mut buffer:Vec<u8> = vec![0;64];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_11211_packet(&mut payload, 5);
        tx.send_to(packet.packet(), None);
        let mut buffer:Vec<u8> = vec![0;59];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_11211_packet(&mut payload, 10);
        tx.send_to(packet.packet(), None);
    }

    println!("Packets sent");
}

fn get_11211_packet(packet: &mut MutableIpv6Packet, size: u16) {
    packet.set_payload_length(size);
}

fn get_ipv6_packet(packet: &mut MutableIpv6Packet, source:Ipv6Addr, destination:Ipv6Addr) {
    packet.set_version(6);
    packet.set_traffic_class(0);
    packet.set_flow_label(0);
    packet.set_payload_length(0);
    packet.set_next_header(IpNextHeaderProtocol::new(58));
    packet.set_hop_limit(10);
    packet.set_source(source);
    packet.set_destination(destination);
}

fn get_packet(packet: &mut MutableEthernetPacket, source: MacAddr, destination: MacAddr, source_ip: Ipv6Addr, destination_ip: Ipv6Addr) {
    packet.set_source(source);
    packet.set_destination(destination);
    packet.set_ethertype(Ipv6);

    let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
    get_ipv6_packet(&mut payload, source_ip, destination_ip);
}