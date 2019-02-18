extern crate pnet;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, DataLinkSender};
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};

use std::env;
use std::thread;
use pnet::util::MacAddr;
use std::net::Ipv6Addr;
use std::collections::HashMap;

mod routing;

fn main() {

    println!("Welcome to Oliver's Software IPv6 Router");

    /*
       TODO plan
       - Add node as gateway to neighours (in python) DONE
       - Add static routing thread DONE - kind of
       - Add thread per interface to forward packets
    */

    let default_route_ip = match Ipv6Addr::from_str(env::args().nth(1).unwrap().as_str()) {
        Ok(x) => x,
        Err(e) => panic!("An error occurred when parsing input default route: {}", e)
    };


    let default_route_mac = match MacAddr::from_str(env::args().nth(2).unwrap().as_str()) {
        Ok(x) => x,
        Err(e) => panic!("An error occurred when parsing input default route: {}", e)
    };

    //for now - setup unchanging routing table, then start data plan threads
    let mut routing_table = routing::Routing::new(default_route_ip, default_route_mac);

    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,2),MacAddr(00,00,00,00,01,00));
    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,3),MacAddr(00,00,00,00,02,00));

    let interfaces = datalink::interfaces();
    let mut rx_threads_mut = HashMap::new();
    let mut tx_threads_mut = HashMap::new();
    for interface in interfaces {
        let (tx,rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(itx, irx)) => (itx, irx),
            Ok(_) => panic!("Unhandled channel type"),
            Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
        };
        let interface_mac = interface.mac_address();
        let rx_thread = thread::spawn()
        let tx_thread
        rx_threads_mut.insert(interface_mac, rx_thread);
        tx_thread.insert(interface_mac, tx_thread);
    }

    //OLD - TO CHANGE

    let left_interface_name = env::args().nth(1).unwrap();
    let right_interface_name = env::args().nth(2).unwrap();
    let left_interface_names_match =
        |iface: &NetworkInterface| iface.name == left_interface_name;
    let right_interface_names_match =
        |iface: &NetworkInterface| iface.name == right_interface_name;

    // Find the network interface with the provided names
    let interfaces = datalink::interfaces();
    let left_interface = interfaces.into_iter()
        .filter(left_interface_names_match)
        .next()
        .unwrap();
    let interfaces = datalink::interfaces();
    let right_interface = interfaces.into_iter()
        .filter(right_interface_names_match)
        .next()
        .unwrap();

    // Create the input channel, dealing with layer 2 packets
    let (ltx, lrx) = match datalink::channel(&left_interface, Default::default()) {
        Ok(Ethernet(itx, irx)) => (itx, irx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    // Create the output channel
    let (rtx, rrx) = match datalink::channel(&right_interface, Default::default()) {
        Ok(Ethernet(otx, orx)) => (otx, orx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    // switch to async
    let diode_thread_l_to_r = thread::spawn(|| run_diode(lrx, rtx, MacAddr(00,00,00,00,02,00),"ltor"));
    let diode_thread_r_to_l = thread::spawn(|| run_diode(rrx, ltx, MacAddr(00,00,00,00,01,00),"rtol"));

    println!("Left: {} and Right: {} channels set up", left_interface_name, right_interface_name);

    println!("running");

    print!("Completed {:?} {:?}",diode_thread_l_to_r.join(), diode_thread_r_to_l.join());
}

fn run_diode(mut rx: Box<DataLinkReceiver>, mut tx: Box<DataLinkSender>, destination_mac_address: MacAddr, tag: &str) {
    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();

                // Constructs a single packet, the same length as the the one received,
                // using the provided closure. This allows the packet to be constructed
                // directly in the write buffer, without copying. If copying is not a
                // problem, you could also use send_to.
                //
                // The packet is sent once the closure has finished executing.
                println!("Tag: {} Sending packet: {:?}", tag, packet);
                tx.build_and_send(1, packet.packet().len(),
                                   &mut | new_packet| {
                                       let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                                       // Create a clone of the original packet
                                       new_packet.clone_from(&packet);

                                       // set the source and destination
                                       new_packet.set_source(packet.get_destination());
                                       new_packet.set_destination(destination_mac_address);
                                   });
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
    }
}

}