use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use pnet::util::MacAddr;
use crate::routing::Routing;

pub fn start_rx(mut rx: Box<DataLinkReceiver>, tx_interfaces : HashMap<MacAddr,DataLinkSender>, routing : &Routing) -> JoinHandle<T> {
    let forwarder = Forwarder::new(routing, rx, tx_interfaces)
    thread::spawn(|| forwarder.start())
}

pub struct Forwarder {
    routing: Box<Routing>,
    tx_interfaces: HashMap<MacAddr, Box<DataLinkSender>>,
    rx: Box<DataLinkReceiver>
}

impl Forwarder {
    pub fn new(routing : Box<Routing>, rx : Box<DataLinkReceiver>, tx_interfaces: HashMap<MacAddr, Box<DataLinkSender>>) -> Forwarder{
        Forwarder{ routing, tx_interfaces, rx}
    }

    pub fn rx(&self, mut rx: Box<DataLinkReceiver>, tx_interfaces : HashMap<MacAddr,DataLinkSender>, routing : &Routing) {
        loop {
            match rx.next() {
                Ok(packet) => {
                    let packet = EthernetPacket::new(packet).unwrap();
                    tx.build_and_send(1, packet.packet().len(),
                                      &mut |new_packet| send_packet_continuation(packet));
                },
                Err(e) => {
                    // If an error occurs, we can handle it here
                    //todo handle interface errors
                    panic!("An error occurred while reading: {}", e);
                }
            }
        }
    }
}

fn send_packet_continuation(packet : EthernetPacket ) -> MutableEthernetPacket {{
    //get ip address
    let payload_packet = match Packet::new((packet.get_payload())).from_packet() {
        Ipv6Packet => build_ipv6_packet(),
        _ => panic!("Unknown packet type"),
    };

    let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

    // Create a clone of the original packet
    new_packet.clone_from(&packet);

    new_packet.set_payload(payload_packet.payload);

    // set the source and destination
    new_packet.set_source(new_packet.get_destination());
    new_packet.set_destination(routing.get_route(destination_ip6_address));
    return new_packet
});}