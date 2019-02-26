use std::thread;
use std::collections::hash_map::HashMap;
use pnet::util::MacAddr;
use std::thread::JoinHandle;


pub fn start_rx(mut rx: Box<DataLinkReceiver>, tx_interfaces : HashMap<MacAddr,DataLinkSender>, routing : &Routing) -> JoinHandle<T> {
    let receiver = Receiver::new(routing, rx, tx_interfaces);
    thread::spawn(|| receiver.start())
}

pub struct Receiver {
    routing: Box<Routing>,
    tx_interfaces: HashMap<MacAddr, Channel>,
    rx: Box<DataLinkReceiver>,
}

impl Receiver {
    pub fn new(routing : Box<Routing>, rx : Box<DataLinkReceiver>, tx_interfaces: HashMap<MacAddr, Box<DataLinkSender>>) -> Receiver{
        Receiver{ routing, tx_interfaces, rx}
    }

    pub fn start() {
        loop {
            self::rx();
        }
    }

    fn rx(&mut self) {
        match self.rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                let (packet, mac_address) = transform_packet(packet);
                let tx = match self.tx_interfaces.get(mac_address) {
                    Some(tx) => tx,
                    None => panic!("Transmission interface not found"), //todo deal with this properly
                };
                tx.send_to(packet, None);
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                //todo handle interface errors
                panic!("An error occurred while reading: {}", e);
            }
        }

    }

    fn transform_packet(&self, packet : EthernetPacket ) -> (EthernetPacket, MacAddress) {
        let ipv6_packet = match Packet::new((packet.payload())).from_packet() {
            Ipv6Packet(pkt) => pkt,
            _ => panic!("Unknown packet type"),
        };
        let mut result_packet = MutableEthernetPacker::new(packet);
        let macs = self.routing.get_route(ipv6_packet.get_destination());
        result_packet.set_destination(macs.destination);
        result_packet.set_source(macs.source);
        return (result_packet.to_immutable(), macs.source)
    }
}
