use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::MutablePacket;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ipv6::MutableIpv6Packet;

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    let test = evn::args().nth(2).unwrap();

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

    let mut buffer = vec![0;];
    let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();

    let mut payload = Ipv6Packet::new(&mut packet.payload_mut());

    packet.

    tx.send_to(&packet, None);

    println!("Packets sent");
}
