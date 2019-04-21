use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::MutablePacket;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::EtherTypes::Ipv6;
use pnet::packet::ipv6::MutableIpv6Packet;
use std::net::Ipv6Addr;

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    let test = evn::args().nth(2).unwrap();

    let destination_mac = env::args().nth(3).unwrap();

    let source_ip = env::args().nth(4).unwrap();

    let destination_ip = env::args().nth(5).unwrap();

    println!("Client starting");

    let interfaces = datalink::interfaces();

    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    let interface = interfaces.into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let (tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(itx, irx)) => (itx, irx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    let size = 64;
    let mut buffer:Vec<u8> = vec![0;size];
    let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();

    let mut payload = Ipv6Packet::new(&mut packet.payload_mut());
    payload.set_version(6);
    payload.set_traffic_class(0);
    payload.set_flow_label(0);
    payload.set_payload_length();
    payload.set_next_header(58);
    payload.set_hop_limit(10);
    payload.set_source_address(source_ip.parse::<Ipv6Addr>());
    payload.set_destination_address(destination_ip.parse::<Ipv6Addr>());

    packet.set_source_address(interface.mac_address);
    packet.set_destination_address(destination_mac);
    packet.set_ethertype(Ipv6);

    tx.send_to(&packet, None);

    println!("Packets sent");
}

fn
