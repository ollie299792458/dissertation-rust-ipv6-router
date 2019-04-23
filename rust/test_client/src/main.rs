use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet,MutablePacket};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::EtherTypes::Ipv6;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::icmpv6;
use pnet::packet::icmpv6::{MutableIcmpv6Packet, Icmpv6Packet, Icmpv6Types, Icmpv6Code};
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

    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
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

    if test == "11212" {
        let mut buffer:Vec<u8> = vec![0;54];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_11212_packet(&mut payload, 1);
        tx.send_to(packet.packet(), None);
        let mut buffer:Vec<u8> = vec![0;54];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_11212_packet(&mut payload, 10);
        tx.send_to(packet.packet(), None);
    }

    if test == "1211" {
        let mut buffer:Vec<u8> = vec![0;70];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1211_packet(&mut payload);
        tx.send_to(packet.packet(), None);
    }

    println!("Packets sent");
}

fn get_1211_packet(packet: &mut MutableIpv6Packet) {
    packet.set_next_header(IpNextHeaderProtocol::new(58));
    packet.set_payload_length(20);
    {
        let mut icmp_packet = MutableIcmpv6Packet::new(packet.payload_mut()).unwrap();
        icmp_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
        icmp_packet.set_icmpv6_code(Icmpv6Code::new(0));
        let payload = &mut icmp_packet.payload_mut();
        payload[0] = 0;
        payload[1] = 0;
        payload[2] = 0;
        payload[3] = 0;
    }
    let checksum ;
    let payload_length ;
    {
        let icmp_packet = Icmpv6Packet::new(packet.payload()).unwrap();
        checksum = icmpv6::checksum(&icmp_packet, &packet.get_source(), &packet.get_destination());
        payload_length = icmp_packet.packet().len();
    }
    {
        let mut icmp_packet = MutableIcmpv6Packet::new(packet.payload_mut()).unwrap();
        icmp_packet.set_checksum(checksum + 1);
    }
    packet.set_payload_length(payload_length as u16);
}

fn get_11211_packet(packet: &mut MutableIpv6Packet, size: u16) {
    packet.set_payload_length(size);
}

fn get_11212_packet(packet: &mut MutableIpv6Packet, hop_limit: u8) {
    packet.set_hop_limit(hop_limit);
}

fn get_ipv6_packet(packet: &mut MutableIpv6Packet, source:Ipv6Addr, destination:Ipv6Addr) {
    packet.set_version(6);
    packet.set_traffic_class(0);
    packet.set_flow_label(0);
    packet.set_payload_length(0);
    packet.set_next_header(IpNextHeaderProtocol::new(59));
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