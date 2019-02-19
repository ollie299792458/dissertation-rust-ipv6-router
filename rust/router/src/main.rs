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
mod data;

fn main() {

    println!("Welcome to Oliver's Software IPv6 Router");

    /*
       TODO plan
       - Add node as gateway to neighours (in python) DONE
       - Add static routing thread DONE - kind of
       - Add thread per interface to forward packets
       - Kill threads
       - Use channels and add tx threads
    */

    let default_route_ip = match Ipv6Addr::from_str(env::args().nth(1).unwrap().as_str()) {
        Ok(x) => x,
        Err(e) => panic!("An error occurred when parsing input default route: {}", e)
    };


    let default_route_mac = match MacAddr::from_str(env::args().nth(2).unwrap().as_str()) {
        Ok(x) => x,
        Err(e) => panic!("An error occurred when parsing input default route: {}", e)
    };

    //for now - setup unchanging routing table, then start forwarder plan threads
    let mut routing_table = routing::Routing::new(default_route_ip, default_route_mac);

    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,2),MacAddr(00,00,00,00,01,00));
    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,3),MacAddr(00,00,00,00,02,00));


    //start all the tx threads, collecting all their input channels in a HashMap
    let interfaces = datalink::interfaces();
    let mut tx_channels = HashMap::new();
    for interface in interfaces {
        let (tx,rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(itx, irx)) => (itx, irx),
            Ok(_) => panic!("Unhandled channel type"),
            Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
        };
    }


    //start all the rx threads, giving each one the tx channels



    //OLD - TO CHANGE
    let mut rx_threads = HashMap::new();
    for interface in interfaces {
        let (tx,rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(itx, irx)) => (itx, irx),
            Ok(_) => panic!("Unhandled channel type"),
            Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
        };
        let interface_mac = interface.mac_address();
        let rx_thread = data::start_rx(rx,&tx_channels, &routing_table);
        rx_threads.insert(interface_mac, rx_thread);
        tx_channels.insert(interface_mac, tx);
    }
}