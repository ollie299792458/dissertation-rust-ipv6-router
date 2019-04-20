use std::thread;
use std::collections::hash_map::HashMap;
use std::thread::JoinHandle;

use pnet::util::MacAddr;
use pnet::packet::Packet;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::datalink::{DataLinkReceiver,DataLinkSender};

use crate::control::Routing;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;


//RECEIVER

pub fn start_receiver(rx: Box<DataLinkReceiver>, tx_senders :  &HashMap<MacAddr,Sender<Box<[u8]>>>, routing : Arc<Routing>) -> JoinHandle<()> {
    let tx_senders_clone = tx_senders.clone();
    thread::spawn(move|| receiver_loop(rx, tx_senders_clone, routing))
}

fn receiver_loop(mut rx: Box<DataLinkReceiver>, tx_senders : HashMap<MacAddr,Sender<Box<[u8]>>>, routing : Arc<Routing>) {
    loop {
        match rx.next() {
            Ok(packet) => { //todo lots of copies here, and locked routing struct, potential performance bottleneck
                let packet = packet.to_vec();
                let packet = MutableEthernetPacket::owned(packet).unwrap();
                println!("Received packet: {:?}, data: {:?}", packet, packet.packet());
                let (mac_address,packet) = match transform_packet_and_get_address(packet, Arc::clone(&routing)) {
                    Ok(p) => p,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                let tx = match tx_senders.get(&mac_address) {
                    Some(tx) => tx,
                    None => panic!("Transmission interface not found for: {:?}", mac_address),
                };
                tx.send(Box::from(packet.packet())).unwrap_or_default();
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                //todo handle interface errors
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn transform_packet_and_get_address(mut packet: MutableEthernetPacket, routing: Arc<Routing>) -> Result<(MacAddr, MutableEthernetPacket), String> {
    let ipv6_packet = match MutableIpv6Packet::owned(packet.payload().to_vec()) {
        Some(p) => p,
        None => return Err(format!("Invalid Packet")),
    };
    //println!("Received ipv6 packet: source: {:?} destination: {:?}", ipv6_packet.get_source(), ipv6_packet.get_destination());
    let macs = routing.get_route(ipv6_packet.get_destination()).unwrap();
    let ipv6_packet = match transform_ipv6_packet(ipv6_packet, Arc::clone(&routing)) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    packet.set_payload(ipv6_packet.packet());
    packet.set_destination(macs.destination);
    packet.set_source(macs.source);
    //println!("Sent packet (ip, mac): to {:?}, from: {:?}, on interface {:?} to {:?}", ipv6_packet.get_destination(), ipv6_packet.get_source(), macs.source, macs.destination);
    return Ok((macs.source, packet));
}

fn transform_ipv6_packet(mut packet: MutableIpv6Packet, routing: Arc<Routing>) -> Result<MutableIpv6Packet, String> {
    //packet length
    let reported_length = packet.get_payload_length();
    let actual_length = packet.packet().len() as u16 - 40; //40 is the length of the first header
    //println!("Reported length: {}, actual length: {:?}", reported_length, actual_length);
    if reported_length != actual_length {
        return Err(format!("Incorrect payload length, reported: {}, actual: {}", reported_length, actual_length));
    }
    //hop limit
    let hop_limit = packet.get_hop_limit();
    let destination = packet.get_destination();
    if hop_limit < 0 || ((hop_limit <= 1) && destination != routing.get_router_address()) {
        return Err(format!("Hop limit reached, packet dropped"));
    } else {
        packet.set_hop_limit(hop_limit-1);
    }

    //todo do ICMPv6 if for this node - destination (general breakout) and next header split

    return Ok(packet);
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
        /*if EthernetPacket::new(&packet).unwrap().get_destination() != MacAddr(00, 00, 00, 00, 00, 00) {
            println!("Sent packet: {:?}", EthernetPacket::new(&packet));
        }*/
        sender.send_to(&packet,None);
    }
}