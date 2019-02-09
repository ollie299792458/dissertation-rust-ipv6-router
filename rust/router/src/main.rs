extern crate pnet;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};

use std::env;

fn main() {
    let input_interface_name = env::args().nth(1).unwrap();
    let output_interface_name = env::args().nth(2).unwrap();
    let input_interface_names_match =
        |iface: &NetworkInterface| iface.name == input_interface_name;
    let output_interface_names_match =
        |iface: &NetworkInterface| iface.name == output_interface_name;

    // Find the network interface with the provided names
    let interfaces = datalink::interfaces();
    let input_interface = interfaces.into_iter()
        .filter(input_interface_names_match)
        .next()
        .unwrap();
    let interfaces = datalink::interfaces();
    let output_interface = interfaces.into_iter()
        .filter(output_interface_names_match)
        .next()
        .unwrap();

    // Create the input channel, dealing with layer 2 packets
    let (_, mut irx) = match datalink::channel(&input_interface, Default::default()) {
        Ok(Ethernet(itx, irx)) => (itx, irx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    // Create the output channel
    let (mut otx, _) = match datalink::channel(&output_interface, Default::default()) {
        Ok(Ethernet(otx, orx)) => (otx, orx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    loop {
        match irx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();

                // Constructs a single packet, the same length as the the one received,
                // using the provided closure. This allows the packet to be constructed
                // directly in the write buffer, without copying. If copying is not a
                // problem, you could also use send_to.
                //
                // The packet is sent once the closure has finished executing.
                otx.build_and_send(1, packet.packet().len(),
                                  &mut | new_packet| {
                                      let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                                      // Create a clone of the original packet
                                      new_packet.clone_from(&packet);

                                      // Switch the source and destination
                                      new_packet.set_source(packet.get_destination());
                                      new_packet.set_destination(packet.get_source());
                                  });
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}