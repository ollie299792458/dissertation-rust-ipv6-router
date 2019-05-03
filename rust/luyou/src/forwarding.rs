use std::thread;
use std::collections::hash_map::HashMap;
use std::thread::JoinHandle;

use pnet::util::MacAddr;
use pnet::packet::{MutablePacket,Packet};
use pnet::packet::ethernet::{MutableEthernetPacket, EthernetPacket, EtherTypes::Ipv6};
use pnet::packet::ip::IpNextHeaderProtocols::{Icmpv6, Hopopt};
use pnet::packet::ipv6::{MutableIpv6Packet,Ipv6Packet};
use pnet::packet::icmpv6;
use pnet::packet::icmpv6::{MutableIcmpv6Packet, Icmpv6Packet, Icmpv6Type, Icmpv6Code};
use pnet::packet::icmpv6::Icmpv6Types;
use pnet::datalink::{DataLinkReceiver,DataLinkSender};

use crate::control::{Routing};
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::net::Ipv6Addr;

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

//RECEIVER

pub fn start_receiver(rx: Box<DataLinkReceiver>, tx_senders :  &HashMap<MacAddr,Sender<Box<[u8]>>>, routing : Arc<Routing>) -> JoinHandle<()> {
    let tx_senders_clone = tx_senders.clone();
    thread::spawn(move|| receiver_loop(rx, tx_senders_clone, routing))
}

fn receiver_loop(mut rx: Box<DataLinkReceiver>, tx_senders : HashMap<MacAddr,Sender<Box<[u8]>>>, routing : Arc<Routing>) {
    loop {
        match rx.next() {
            Ok(packet) => { //todo lots of copies here, and locked routing struct, potential performance bottleneck
                let old_packet = EthernetPacket::new(packet).unwrap();
                //println!("Received packet: {:?}, data: {:?}", old_packet, old_packet.payload());
                //buffer is 8 octets longer to account for icmpv6 headers
                let mut buffer:Vec<u8> = vec![0;old_packet.packet().len()+60];
                let mut new_packet= MutableEthernetPacket::new(&mut buffer).unwrap();
                let mac_address = match transform_ethernet_packet(old_packet, &mut new_packet, Arc::clone(&routing)) {
                    Ok(ma) => ma,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                let tx = match tx_senders.get(&mac_address) {
                    Some(tx) => tx,
                    None => panic!("Transmission interface not found for: {:?}", mac_address),
                };
                tx.send(Box::from(new_packet.packet())).unwrap_or_default();
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                //todo handle interface errors
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn transform_ethernet_packet(old_packet: EthernetPacket, new_packet: &mut MutableEthernetPacket, routing: Arc<Routing>) -> Result<(MacAddr), String> {
    let old_ipv6_packet = match Ipv6Packet::new(old_packet.payload()) {
        Some(p) => p,
        None => return Err(format!("Invalid Packet")),
    };

    let mut buffer = vec![0;new_packet.payload().len()];
    let mut new_ipv6_packet = MutableIpv6Packet::new(&mut buffer).unwrap();
    new_ipv6_packet.set_payload_length((new_packet.payload().len()-40) as u16);

    //println!("Received ipv6 packet: source: {:?} destination: {:?}", old_ipv6_packet.get_source(), old_ipv6_packet.get_destination());
    let (source, destination) = match transform_ipv6_packet(old_ipv6_packet, &mut new_ipv6_packet, routing) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    new_packet.set_destination(destination);
    new_packet.set_source(source);
    new_packet.set_ethertype(Ipv6);
    new_packet.set_payload(new_ipv6_packet.packet());
    //println!("Sent packet (ip, mac): to {:?}, from: {:?}, on interface {:?} to {:?}", ipv6_packet.get_destination(), ipv6_packet.get_source(), macs.source, macs.destination);
    return Ok(source);
}

fn transform_ipv6_packet(old_packet: Ipv6Packet, new_packet: & mut MutableIpv6Packet, routing: Arc<Routing>) -> Result<(MacAddr, MacAddr), String> {
    //packet length
    let reported_length = old_packet.get_payload_length();
    let actual_length = old_packet.packet().len() as u16 - 40; //40 is the length of the first header
    //println!("Reported length: {}, actual length: {:?}", reported_length, actual_length);
    if reported_length != actual_length {
        return Err(format!("Incorrect payload length, reported: {}, actual: {}", reported_length, actual_length));
    }
    //hop limit
    let hop_limit = old_packet.get_hop_limit();
    let destination = old_packet.get_destination();
    if (hop_limit <= 1) && destination != routing.get_router_address() {
        let mut buffer = vec![0;new_packet.payload().len()];
        let mut new_icmp_packet = MutableIcmpv6Packet::new(&mut buffer).unwrap();
        let (macs, (source, destination)) = match transform_icmp6_packet((Icmpv6Types::TimeExceeded,0),
        old_packet,&mut new_icmp_packet, routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_payload_length((new_icmp_packet.packet().len()) as u16);
        new_packet.set_payload(new_icmp_packet.packet());
        new_packet.set_version(6);
        new_packet.set_traffic_class(0);
        new_packet.set_flow_label(0);
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        println!("Packet dropped, hop limit, and imcpv6 response sent");
        return Ok(macs);
    }

    //respond if this is destination
    if destination == routing.get_router_address() {
        //go through headers
        //todo support more headers - breakout here - fix following code
        if old_packet.get_next_header() != Icmpv6 {
            let payload_length = old_packet.get_payload_length();
            let mut buffer = vec![0;new_packet.payload().len()];
            let mut new_icmp_packet = MutableIcmpv6Packet::new(&mut buffer).unwrap();
            let (macs, (source, destination)) = match transform_icmp6_packet((Icmpv6Types::ParameterProblem, 40),
                                                                             old_packet,&mut new_icmp_packet, routing) {
                Ok(r) => r,
                Err(s) => return Err(s),
            };
            new_packet.set_payload_length((new_icmp_packet.packet().len()) as u16);
            new_packet.set_payload(new_icmp_packet.packet());
            new_packet.set_source(source);
            new_packet.set_destination(destination);
            new_packet.set_next_header(Icmpv6);
            new_packet.set_payload_length(payload_length+8);//todo fix this, can result in miscalculated mtu's

            return Ok(macs);
        }

        //get packet
        let payload_length = old_packet.get_payload_length();
        let old_icmp_packet = match Icmpv6Packet::new(old_packet.payload()) {
            Some(p) => p,
            None => return Err(format!("Packet's destination is router, unknown type")),
        };

        let mut buffer = vec![0;new_packet.payload().len()];
        let mut new_icmp_packet = MutableIcmpv6Packet::new(&mut buffer).unwrap();
        let (macs, (source, destination)) = match transform_icmp6_packet((old_icmp_packet.get_icmpv6_type(),0),
                                          old_packet,&mut new_icmp_packet, routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_payload_length((new_icmp_packet.packet().len()) as u16);
        new_packet.set_payload(new_icmp_packet.packet());
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        new_packet.set_next_header(Icmpv6);
        new_packet.set_payload_length(payload_length);

        return Ok(macs);
    }

    let (mac_source, mac_destination) = routing.get_route(old_packet.get_destination());

    let length = (old_packet.get_payload_length() + 40) as u32;
    let mtu = routing.get_mtu(mac_source);
    if length > mtu {
        let mut buffer = vec![0;new_packet.payload().len()];
        let mut new_icmp_packet = MutableIcmpv6Packet::new(&mut buffer).unwrap();
        let (macs, (source, destination)) = match transform_icmp6_packet((Icmpv6Types::PacketTooBig,0),
                                                                       old_packet,
                                                                       &mut new_icmp_packet,
                                                                       routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_payload_length((new_icmp_packet.packet().len()) as u16);
        new_packet.set_payload(new_icmp_packet.packet());
        new_packet.set_version(6);
        new_packet.set_traffic_class(0);
        new_packet.set_flow_label(0);
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        return Ok(macs);
    }

    //if standard packet get to here
    new_packet.clone_from(&old_packet);


    if old_packet.get_next_header() == Hopopt {
        //todo implement hop-by-hop - extension
    }

    let new_hop_limit:u8 = hop_limit - 1_u8;
    //todo fix this, actually decrementing hop limit works, but results in inexplicable packet drops
    new_packet.set_hop_limit(new_hop_limit);

    return Ok((mac_source, mac_destination));
}

//todo may need source and destination here
fn transform_icmp6_packet((icmpv6_type, parameter): (Icmpv6Type, u32), old_ipv6_packet: Ipv6Packet, new_packet: &mut MutableIcmpv6Packet, routing: Arc<Routing>) -> Result<((MacAddr, MacAddr), (Ipv6Addr, Ipv6Addr)), String> {
    let source = old_ipv6_packet.get_source();
    let destination = old_ipv6_packet.get_destination();
    if old_ipv6_packet.get_next_header() == Icmpv6{
        let old_packet = match Icmpv6Packet::new(old_ipv6_packet.payload()) {
            Some(p) => p, //todo deal with erroneous next header - maybe move checksum elsewher
            None => return Err(format!("Erroneous next header - unimplemented"))
        };
        let checksum = icmpv6::checksum(&old_packet, &source, &destination);
        if checksum != old_packet.get_checksum() {
            return Err(format!("Invalid ICMPv6 Checksum, packet dropped"));
        }
    }

    match icmpv6_type {
        Icmpv6Types::EchoRequest => {
            let new_source = routing.get_router_address();
            let new_destination = source;
            shuffle_icmpv6_payload(&old_ipv6_packet, new_packet, new_source, Arc::clone(&routing));
            new_packet.set_icmpv6_type(Icmpv6Types::EchoReply);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let payload = &mut new_packet.payload_mut();
            let old_packet = Icmpv6Packet::new(old_ipv6_packet.payload()).unwrap();
            let old_payload = old_packet.payload();
            payload[0] = old_payload[0];
            payload[1] = old_payload[1];
            payload[2] = old_payload[2];
            payload[3] = old_payload[3];
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok((routing.get_route(new_destination), (new_source, new_destination)));
        }
        Icmpv6Types::ParameterProblem => {
            let new_source = routing.get_router_address();
            let new_destination = source;
            shuffle_icmpv6_payload(&old_ipv6_packet, new_packet,new_source, Arc::clone(&routing));
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(1)); //todo support more codes
            let payload = &mut new_packet.payload_mut(); //set 4 octets to mtu
            let (mac_source, mac_destination) = routing.get_route(new_destination);
            let pointer = parameter;
            payload[0] = (pointer / (1 << 24)) as u8;
            payload[1] = (pointer / (1 << 16)) as u8;
            payload[2] = (pointer / (1 << 8)) as u8;
            payload[3] = (pointer % (1 << 8)) as u8;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok(((mac_source, mac_destination), (new_source, new_destination)))
        },
        Icmpv6Types::PacketTooBig => {
            let new_source = routing.get_router_address();
            let new_destination = source;
            shuffle_icmpv6_payload(&old_ipv6_packet, new_packet,new_source, Arc::clone(&routing));
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let payload = &mut new_packet.payload_mut(); //set 4 octets to mtu
            let (mac_source, mac_destination) = routing.get_route(new_destination);
            let (old_mac_source,_) = routing.get_route(destination);
            let mtu = routing.get_mtu(old_mac_source);
            payload[0] = (mtu / (1 << 24)) as u8;
            payload[1] = (mtu / (1 << 16)) as u8;
            payload[2] = (mtu / (1 << 8)) as u8;
            payload[3] = (mtu % (1 << 8)) as u8;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok(((mac_source, mac_destination), (new_source, new_destination)))
        },
        Icmpv6Types::TimeExceeded => {
            let new_source = routing.get_router_address();
            let new_destination = source;
            shuffle_icmpv6_payload(&old_ipv6_packet, new_packet,new_source, Arc::clone(&routing));
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let payload = &mut new_packet.payload_mut(); //set 4 octets to 0
            payload[0] = 0;
            payload[1] = 0;
            payload[2] = 0;
            payload[3] = 0;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok((routing.get_route(new_destination), (new_source, new_destination)))
        },
        _ => return Err(format!("Unhandled/Unknown ICMP message type")),
    };
}

fn shuffle_icmpv6_payload(old_packet: &Ipv6Packet, new_packet: &mut MutableIcmpv6Packet, source: Ipv6Addr, routing: Arc<Routing>) {
    let (source_mac, _) = routing.get_route(source);
    let packet_size = old_packet.get_payload_length() as u32;
    let mtu = routing.get_mtu(source_mac);
    let buffer = old_packet.packet();
    if (packet_size+8) < mtu { //todo fix this so it deals with min mtu not actual mtu (1280)
        let mut new_buffer = vec![0; buffer.len() + 4];
        let new_buffer_slice = &mut new_buffer[4..];
        new_buffer_slice.clone_from_slice(&buffer);
        new_packet.set_payload(&new_buffer); //due to set_payload() only need to shuffle by 4
    } else {
        let buffer = &buffer[0..(mtu as usize-8)];
        let mut new_buffer = vec![0; buffer.len() + 4];
        new_buffer.clone_from_slice(&buffer[4..]);
        new_packet.set_payload(&new_buffer);
    }
}

//SENDER
pub fn start_sender(tx : Box<DataLinkSender>) -> (JoinHandle<()>, Sender<Box<[u8]>>) {
    let (sender, receiver) = channel();
    let handle = thread::spawn(move || sender_loop(tx, receiver));
    (handle, (sender))
}

fn sender_loop(mut sender: Box<DataLinkSender>, receiver: Receiver<Box<[u8]>>) {
    loop {
        let packet = receiver.recv().unwrap();
        //println!("Sent packet: {:?}, data: {:?}", EthernetPacket::new(&packet).unwrap(), EthernetPacket::new(&packet).unwrap().payload());
        /*if EthernetPacket::new(&packet).unwrap().get_destination() != MacAddr(00, 00, 00, 00, 00, 00) {
            println!("Sent packet: {:?}", EthernetPacket::new(&packet));
        }*/
        sender.send_to(&packet,None);
    }
}
