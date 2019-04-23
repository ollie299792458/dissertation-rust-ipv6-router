use std::thread;
use std::collections::hash_map::HashMap;
use std::thread::JoinHandle;

use pnet::util::MacAddr;
use pnet::packet::{MutablePacket,Packet};
use pnet::packet::ethernet::{MutableEthernetPacket, EthernetPacket, EtherTypes::Ipv6};
use pnet::packet::ip::IpNextHeaderProtocols::Icmpv6;
use pnet::packet::ipv6::{MutableIpv6Packet,Ipv6Packet};
use pnet::packet::icmpv6;
use pnet::packet::icmpv6::{MutableIcmpv6Packet, Icmpv6Packet, Icmpv6Type, Icmpv6Code};
use pnet::packet::icmpv6::Icmpv6Types::{TimeExceeded, EchoReply, PacketTooBig};
use pnet::datalink::{DataLinkReceiver,DataLinkSender};

use crate::control::{Routing, InterfaceMacAddrs};
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::net::Ipv6Addr;


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
                let mut buffer = vec![0;old_packet.packet().len()];
                let mut new_packet= MutableEthernetPacket::new(&mut buffer).unwrap();
                let (mac_address) = match transform_ethernet_packet(old_packet, &mut new_packet, Arc::clone(&routing)) {
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

    let mut new_ipv6_packet = MutableIpv6Packet::new(new_packet.payload_mut()).unwrap();

    //println!("Received ipv6 packet: source: {:?} destination: {:?}", ipv6_packet.get_source(), ipv6_packet.get_destination());
    let macs = match transform_ipv6_packet(old_ipv6_packet, &mut new_ipv6_packet, routing) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    new_packet.set_destination(macs.destination);
    new_packet.set_source(macs.source);
    new_packet.set_ethertype(Ipv6);
    //println!("Sent packet (ip, mac): to {:?}, from: {:?}, on interface {:?} to {:?}", ipv6_packet.get_destination(), ipv6_packet.get_source(), macs.source, macs.destination);
    return Ok(macs.source);
}

fn transform_ipv6_packet<'a>(old_packet: Ipv6Packet, new_packet: &'a mut MutableIpv6Packet, routing: Arc<Routing>) -> Result<(&'a InterfaceMacAddrs), String> {
    new_packet.clone_from(&old_packet);
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
        let old_icmp_packet = Icmpv6Packet::new(old_packet.payload()).unwrap();
        let mut new_icmp_packet = MutableIcmpv6Packet::new(new_packet.payload_mut()).unwrap();
        let (macs, source, destination) = match transform_icmp6_packet(TimeExceeded,
                                                                       old_packet.get_source(),
                                                                       old_packet.get_destination(),
                                                                       old_icmp_packet,
                                                                       &mut new_icmp_packet,
                                                                       routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_version(6);
        new_packet.set_traffic_class(0);
        new_packet.set_flow_label(0);
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        return Ok(macs);
    } else {
        let new_hop_limit:u8 = hop_limit - 1_u8;
        //todo fix this, actually decrementing hop limit works, but results in inexplicable packet drops
        new_packet.set_hop_limit(new_hop_limit);
    }

    if destination == routing.get_router_address() {
        //todo get to last header to check if ipv6


        //get packet
        let old_icmp_packet = match Icmpv6Packet::new(old_packet.payload()) {
            Some(p) => p,
            None => return Err(format!("Packet's destination is router, unknown type")),
        };

        let mut new_icmp_packet = MutableIcmpv6Packet::new(new_packet.payload_mut()).unwrap();
        let (macs, source, destination) = match transform_icmp6_packet(old_icmp_packet.get_icmpv6_type(), old_packet.get_source(),
                                          old_packet.get_destination(),old_icmp_packet,
                                          &mut new_icmp_packet, routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        new_packet.set_next_header(Icmpv6);
        new_packet.set_payload_length(old_packet.get_payload_length());

        return Ok(macs);
    }

    let macs = routing.get_route(old_packet.get_destination());

    let length = (old_packet.get_payload_length() + 40) as u32;
    let mtu = routing.get_mtu(macs.source);
    if (length > mtu) {
        let old_icmp_packet = Icmpv6Packet::new(old_packet.payload()).unwrap();
        let mut new_icmp_packet = MutableIcmpv6Packet::new(new_packet.payload_mut()).unwrap();
        let (macs, source, destination) = match transform_icmp6_packet(PacketTooBig,
                                                                       old_packet.get_source(),
                                                                       old_packet.get_destination(),
                                                                       old_icmp_packet,
                                                                       &mut new_icmp_packet,
                                                                       routing) {
            Ok(r) => r,
            Err(s) => return Err(s),
        };
        new_packet.set_version(6);
        new_packet.set_traffic_class(0);
        new_packet.set_flow_label(0);
        new_packet.set_source(source);
        new_packet.set_destination(destination);
        return Ok(macs);
    }

    //if standard packet get to here
    return Ok(macs);
}

fn transform_icmp6_packet<'a>(icmpv6_type: Icmpv6Type, source: Ipv6Addr, destination: Ipv6Addr, old_packet: Icmpv6Packet, new_packet: &'a mut MutableIcmpv6Packet, routing: Arc<Routing>) -> Result<(&'a InterfaceMacAddrs, Ipv6Addr, Ipv6Addr), String> {
    let checksum = icmpv6::checksum(&old_packet, &source, &destination);
    if checksum != old_packet.get_checksum() {
        return Err(format!("Invalid ICMPv6 Checksum, packet dropped"));
    }
    match icmpv6_type {
        EchoReply => {
            new_packet.clone_from(&old_packet);
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let new_source = routing.get_router_address();
            let new_destination = source;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok((routing.get_route(source), new_source, new_destination));
        }
        PacketTooBig => {
            new_packet.clone_from(&old_packet);
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let mut payload = new_packet.payload_mut(); //set 4 octets to mtu
            let mtu = routing.get_mtu(routing.get_route(destination).source);
            payload[0] = (mtu / (2 ^ 24)) as u8;
            payload[1] = (mtu / (2 ^ 16)) as u8;;
            payload[2] = (mtu / (2 ^ 8)) as u8;;
            payload[3] = (mtu % (2 ^ 8)) as u8;;
            let new_source = routing.get_router_address();
            let new_destination = source;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok((routing.get_route(source), new_source, new_destination))
        },
        TimeExceeded => {
            new_packet.clone_from(&old_packet);
            new_packet.set_icmpv6_type(icmpv6_type);
            new_packet.set_icmpv6_code(Icmpv6Code::new(0));
            let mut payload = new_packet.payload_mut(); //set 4 octets to 0
            payload[0] = 0;
            payload[1] = 0;
            payload[2] = 0;
            payload[3] = 0;
            let new_source = routing.get_router_address();
            let new_destination = source;
            new_packet.set_checksum(icmpv6::checksum(&Icmpv6Packet::new(new_packet.payload()).unwrap(), &new_source, &new_destination));
            return Ok((routing.get_route(source), new_source, new_destination))
        },
        _ => return Err(format!("Unhandled ICMP message type")),
    };
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