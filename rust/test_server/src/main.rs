use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::Packet;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv6::Ipv6Packet;

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    println!("Server starting");

    let interfaces = datalink::interfaces();

    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    let interface = interfaces.into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(itx, irx)) => (itx, irx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    println!("Server started");

    loop {
        match rx.next() {
            Ok(packet) => {
                let eth_packet = EthernetPacket::new(packet).unwrap();
                let ipv6_packet = Ipv6Packet::new(eth_packet.payload()).unwrap();
                println!("{:?}:{:?}",ipv6_packet.get_source(), ipv6_packet.packet());
                //println!("{:X?}",packet);
            }
            Err(e)=> {
                println!("Server: {:?}",e);
                continue;
            }
        }
    }
}
