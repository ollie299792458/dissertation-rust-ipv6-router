use std::env;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::Packet;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv6::Ipv6Packet;

/*  This file is part of Software IPv6 Router in Rust.

    Software IPv6 Router in Rust is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Software IPv6 Router in Rust is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Software IPv6 Router in Rust.  If not, see <https://www.gnu.org/licenses/>.

    Copyright 2018,2019 Oliver Black
*/

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    println!("Luxingfu - Server starting");

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
