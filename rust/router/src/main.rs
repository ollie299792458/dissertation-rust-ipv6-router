extern crate pnet;

use pnet::datalink::{self};
use pnet::datalink::Channel::Ethernet;
use pnet::util::MacAddr;

use std::net::Ipv6Addr;
use std::collections::HashMap;

mod control;
use control::Routing;
use control::InterfaceMacAddrs;
use std::sync::Arc;

mod forwarding;


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
    let mut routing = Routing::new(Ipv6Addr::new(0xfc00, 0,0,0,0,0,0,0),
                InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,00),MacAddr(0xff,00,00,00,00,00)));

    routing.add_route(Ipv6Addr::new(0xfc00, 0,0,0,0,0,0,1),
                            InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,01),MacAddr(00,00,00,00,01,00)));
    routing.add_route(Ipv6Addr::new(0xfc00, 0,0,0,0,0,0,2),
                            InterfaceMacAddrs::new(MacAddr(00,00,00,00,03,02), MacAddr(00,00,00,00,02,00)));

    println!("Static routing table setup:{:?}",routing);

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
        //println!("Intf mac: {:?}", interface.mac_address());
        tx_channels.insert(interface.mac_address(),tx);
        rx_channels.insert(interface.mac_address(), rx);
    }


    //start all tx threads, getting the tx channel from each
    let mut tx_senders = HashMap::new();
    let mut sender_threads = Vec::new();
    for (adr,tx) in tx_channels {
        let (handle, sender) = forwarding::start_sender(tx);
        tx_senders.insert(adr, sender);
        sender_threads.push(handle);
    }

    let routing_arc = Arc::new(routing);
    //start all the rx threads, giving each one the tx channels
    let mut receiver_threads = Vec::new();
    for (_,rx) in rx_channels {
        let routing_arc = Arc::clone(&routing_arc);
        receiver_threads.push(forwarding::start_receiver(rx, &tx_senders, routing_arc));
    }

    println!("Running");

    for thread in receiver_threads {
        thread.join().unwrap_or_default();
    }

    for thread in sender_threads {
        thread.join().unwrap_or_default();
    }

    println!("Finished");
}