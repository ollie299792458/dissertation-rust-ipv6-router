extern crate pnet;

use pnet::datalink::{self};
use pnet::datalink::Channel::Ethernet;

use std::collections::HashMap;

mod control;
use control::Routing;
use std::sync::Arc;
use std::{fs, env};

mod forwarding;

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

    println!("Welcome to Oliver's Software IPv6 Router");

    let config_file = env::args().nth(1).unwrap();

    let contents = fs::read_to_string(config_file).unwrap();

    let routing = Routing::new(contents);

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