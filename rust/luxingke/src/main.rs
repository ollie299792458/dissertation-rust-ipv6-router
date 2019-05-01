use std::{env, thread};

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::DataLinkReceiver;
use pnet::packet::{Packet,MutablePacket};
use pnet::packet::ethernet::{MutableEthernetPacket, EthernetPacket};
use pnet::packet::ethernet::EtherTypes::Ipv6;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv6::{MutableIpv6Packet,Ipv6Packet};
use pnet::packet::icmpv6;
use pnet::packet::icmpv6::{MutableIcmpv6Packet, Icmpv6Packet, Icmpv6Types, Icmpv6Code, Icmpv6Type};
use std::net::Ipv6Addr;
use pnet::util::MacAddr;
use std::thread::sleep;
use std::time::Duration;

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

//code coverage calculation 30 untested, 736 total ~95% code coverage

fn main() {
    let interface_name = env::args().nth(1).unwrap();

    let test = env::args().nth(2).unwrap();

    let destination_mac = env::args().nth(3).unwrap().parse::<MacAddr>().unwrap();

    let source_ip = env::args().nth(4).unwrap().parse::<Ipv6Addr>().unwrap();

    let destination_ip = env::args().nth(5).unwrap().parse::<Ipv6Addr>().unwrap();

    println!("Luxingke - Client starting");

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
    } else if test == "11212" {
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
    } else if test == "1211" {
        let mut buffer:Vec<u8> = vec![0;70];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1211_packet(&mut payload);
        tx.send_to(packet.packet(), None);
    } else if test == "1212" {
        let mut buffer:Vec<u8> = vec![0;70];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1212_packet(&mut payload);
        tx.send_to(packet.packet(), None);
    } else if test == "1214" {
        let thread = thread::spawn(|| start_server_icmpv6(rx));
        sleep(Duration::from_millis(100));
        println!("Sending packets");
        let mut buffer:Vec<u8> = vec![0;150];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1214_packet(&mut payload);
        tx.send_to(packet.packet(), None);
        println!("Packet sent");
        match thread.join() {_=> ()};
    } else if test == "1215" {
        let thread = thread::spawn(|| start_server_icmpv6(rx));
        sleep(Duration::from_millis(100));
        println!("Sending packets");
        let mut buffer:Vec<u8> = vec![0;54];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_11212_packet(&mut payload, 1);
        tx.send_to(packet.packet(), None);
        println!("Packet sent");
        match thread.join() {_=> ()};
    } else if test == "1216" {
        let thread = thread::spawn(|| start_server_icmpv6(rx));
        sleep(Duration::from_millis(100));
        println!("Sending packets");
        let mut buffer:Vec<u8> = vec![0;70];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1216_packet(&mut payload);
        tx.send_to(packet.packet(), None);
        println!("Packet sent");
        match thread.join() {_=> ()};
    } else if test == "1217" {
        let thread = thread::spawn(|| start_server_icmpv6(rx));
        sleep(Duration::from_millis(100));
        println!("Sending packets");
        let mut buffer:Vec<u8> = vec![0;54];
        let mut packet= MutableEthernetPacket::new(&mut buffer).unwrap();
        get_packet(&mut packet,interface.mac_address(), destination_mac, source_ip, destination_ip);
        let mut payload = MutableIpv6Packet::new(packet.payload_mut()).unwrap();
        get_1217_packet(&mut payload);
        tx.send_to(packet.packet(), None);
        println!("Packet sent");
        match thread.join() {_=> ()};
    }

    println!("Packet(s) sent");
}

fn get_11211_packet(packet: &mut MutableIpv6Packet, size: u16) {
    packet.set_payload_length(size);
}

fn get_11212_packet(packet: &mut MutableIpv6Packet, hop_limit: u8) {
    packet.set_hop_limit(hop_limit);
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

fn get_1212_packet(packet: &mut MutableIpv6Packet) {
    packet.set_next_header(IpNextHeaderProtocol::new(58));
    packet.set_payload_length(20);
    {
        let mut icmp_packet = MutableIcmpv6Packet::new(packet.payload_mut()).unwrap();
        icmp_packet.set_icmpv6_type(Icmpv6Type::new(148));
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
        icmp_packet.set_checksum(checksum);
    }
    packet.set_payload_length(payload_length as u16);
}

fn get_1214_packet(packet: &mut MutableIpv6Packet) {
    packet.set_next_header(IpNextHeaderProtocol::new(59));

    packet.set_payload_length(96);
}

fn get_1216_packet(packet: &mut MutableIpv6Packet) {
    packet.set_next_header(IpNextHeaderProtocol::new(58));
    packet.set_payload_length(20);
    {
        let mut icmp_packet = MutableIcmpv6Packet::new(packet.payload_mut()).unwrap();
        icmp_packet.set_icmpv6_type(Icmpv6Type::new(128));
        icmp_packet.set_icmpv6_code(Icmpv6Code::new(0));
        let payload = &mut icmp_packet.payload_mut();
        payload[0] = 1;
        payload[1] = 2;
        payload[2] = 3;
        payload[3] = 4;
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
        icmp_packet.set_checksum(checksum);
    }
    packet.set_payload_length(payload_length as u16);
}

fn get_1217_packet(packet: &mut MutableIpv6Packet) {
    packet.set_next_header(IpNextHeaderProtocol::new(29));
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

fn start_server_icmpv6(mut rx: Box<DataLinkReceiver>) {
    println!("Client's server started");

    loop {
        match rx.next() {
            Ok(packet) => {
                let eth_packet = EthernetPacket::new(packet).unwrap();
                let ipv6_packet = Ipv6Packet::new(eth_packet.payload()).unwrap();
                let icmpv6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
                    Some(p) => p,
                    None => {
                        println!("No payload");
                        continue;
                    },
                };
                println!("From:{:?}, Details:{:?}, Payload:{:?}",ipv6_packet.get_source(), icmpv6_packet, icmpv6_packet.payload());
                //println!("{:X?}",packet);
            }
            Err(e)=> {
                println!("Client server: {:?}",e);
                continue;
            }
        }
    }
}