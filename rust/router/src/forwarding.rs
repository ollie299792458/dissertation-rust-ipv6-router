use std::thread;
use std::collections::hash_map::HashMap;
use std::thread::JoinHandle;

use pnet::util::MacAddr;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use pnet::packet::ipv6::{Ipv6Packet, MutableIpv6Packet};
use pnet::datalink::{DataLinkReceiver,DataLinkSender};

use crate::control::Routing;
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
                //println!("Received packet: {:?}", EthernetPacket::new(packet));
                let packet = packet.to_vec();
                let packet = MutableEthernetPacket::owned(packet).unwrap();
                let (mac_address,packet) = transform_packet_and_get_address(packet, Arc::clone(&routing));
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

fn transform_packet_and_get_address(mut packet: MutableEthernetPacket, routing: Arc<Routing>) -> (MacAddr, MutableEthernetPacket) {
    let ipv6_packet = MutableIpv6Packet::owned(packet.payload().to_vec()).unwrap();
    //println!("Received ipv6 packet: source: {:?} destination: {:?}", ipv6_packet.get_source(), ipv6_packet.get_destination());
    let macs = routing.get_route(ipv6_packet.get_destination()).unwrap();
    let ipv6_packet = transform_ipv6_packet(ipv6_packet);
    packet.set_payload(ipv6_packet.packet());
    packet.set_destination(macs.destination);
    packet.set_source(macs.source);
    //println!("Sent packet (ip, mac): to {:?}, from: {:?}, on interface {:?} to {:?}", ipv6_packet.get_destination(), ipv6_packet.get_source(), macs.source, macs.destination);
    (macs.source, packet)
}

fn transform_ipv6_packet(packet: MutableIpv6Packet) -> (MutableIpv6Packet) {
    //todo handle icmpv6 - next hops, etc
    return packet;
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