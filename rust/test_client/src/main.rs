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

    let size = 64;
    let mut buffer:Vec<u8> = vec![0;size];
    let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();

    let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
    get_ipv6_packet(&mut payload, source_ip, destination_ip);
    if test == "11211" {
        get_11211_packet(&mut payload)
    }

    packet.set_source(interface.mac_address());
    packet.set_destination(destination_mac);
    packet.set_ethertype(Ipv6);

    tx.send_to(&packet.packet(), None);

    println!("Packets sent");
}

fn get_11211_packet(packet: &mut MutableIpv6Packet) {
    packet.set_payload_length(4);
    let payload:Vec<u8> = vec![1,2,3,4];
    packet.set_payload(&payload);
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
