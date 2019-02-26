extern crate pnet;

use pnet::datalink::{self};
use pnet::datalink::Channel::Ethernet;

use std::env;
use pnet::util::MacAddr;
use std::net::Ipv6Addr;
use std::collections::HashMap;

fn main() {

    println!("Welcome to Oliver's Software IPv6 Router");

    /*
       TODO plan
       - Add node as gateway to neighours (in python) DONE
       - Add static routing thread DONE - kind of
       - Add thread per interface to forward packets - done
       - Kill threads - not needed
       - Use channels and add tx threads - done
    */

    //for now - setup unchanging routing table, then start forwarder plan threads
    let mut routing_table = control::Routing::new(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,0),
                InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,00),MacAddr(00,00,00,00,00,00)));

    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,2),
                            InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,01),MacAddr(00,00,00,00,01,00)));
    routing_table.add_route(Ipv6Addr::new(0xfc, 0,0,0,0,0,0,3),
                            InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,02), MacAddr(00,00,00,00,02,00)));


    //start all the tx threads, collecting all their input channels in a HashMap
    let interfaces = datalink::interfaces();
    let mut tx_channels = HashMap::new();
    let mut rx_channels = HashMap::new();
    for interface in interfaces {
        let (tx,rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(itx, irx)) => (itx, irx),
            Ok(_) => panic!("Unhandled channel type"),
            Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
        };
        tx_channels.insert(interface.mac_address(),tx);
        rx_channels.insert(interface.mac_address(), rx);
    }

    //start all the rx threads, giving each one the tx channels
    let tx_channels = tx_channels;
    let mut receiver_threads = Vec::new();
    for rx_channel in rx_channels {
        receiver_threads.push(forwarding::start_receiver_thread())
    }

    println!("Running");
}