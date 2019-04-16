use std::collections::HashMap;
use std::net::Ipv6Addr;
use pnet::util::MacAddr;
use std::fmt;

pub struct Routing {
    //hash table for now, maybe move to something more complex
    routing_table:HashMap<Ipv6Addr, InterfaceMacAddrs>,
    default_route:Ipv6Addr,
}

impl Routing {
    /**
    Aim - get this:
    {ff02::1:ff00:2: (00:00:00:00:03:02,00:00:00:00:02:00), fc00::2: (00:00:00:00:03:02,00:00:00:00:02:00)
    , ff02::1:ff00:1: (00:00:00:00:03:01,00:00:00:00:01:00), fc00::1: (00:00:00:00:03:01,00:00:00:00:01:00)
    , fc00::: (00:00:00:00:03:00,ff:00:00:00:00:00)}

    **/

    pub fn new(configuration:String) -> Routing {
        let mut routing_table = HashMap::new();

        let mut lines = configuration.lines();

        //get default route "IPv6" (first line)
        let default_route_string = lines.next().unwrap();

        //todo handle invalid strings better

        let default_route = default_route_string.parse::<Ipv6Addr>().unwrap();

        //get hashmap entries "IPv6-MAC,MAC" (inc a line for default route)
        for line in lines {
            let mut addrs = line.split("@");
            let ipv6_str = addrs.next().unwrap();
            let ipv6 = match ipv6_str.parse::<Ipv6Addr>() {
                Ok(a) => a,
                Err(e) => {println!("Invalid IPv6 Address: {}", ipv6_str); Err(e).unwrap()},
            };
            let mac_addrs_str = addrs.next().unwrap();
            let mut mac_addrs = mac_addrs_str.split(",");
            let source_str = mac_addrs.next().unwrap();
            let destination_str = mac_addrs.next().unwrap();
            let source = match source_str.parse::<MacAddr>() {
                Ok(a) => a,
                Err(e) => {println!("Invalid MAC Address: {}", ipv6_str); Err(e).unwrap()},
            };
            let destination = match destination_str.parse::<MacAddr>(){
                Ok(a) => a,
                Err(e) => {println!("Invalid MAC Address: {}", ipv6_str); Err(e).unwrap()},
            };
            let macs = InterfaceMacAddrs::new(source, destination);
            routing_table.insert(ipv6,macs);
        }

        Routing { routing_table, default_route }
    }

    pub fn add_route(&mut self, ip6:Ipv6Addr, mac:InterfaceMacAddrs) {
        if ip6 != self.default_route {
            self.routing_table.insert(ip6, mac); //todo maybe do something with the result
        } else {
            panic!("Can't add/update default route with add_route()")
        }
    }

    //todo add routers ip as a special case
    //todo update_default_route - or maybe rethink the whole default route semantics

    pub fn get_route(&self, ip6:Ipv6Addr) -> Option<&InterfaceMacAddrs> {
        match self.routing_table.get(&ip6) {
            Some(macs) => {//println!("Address looked up: {:?}, result: {:?}", ip6, macs);
                                            Some(macs)},
            None => self.routing_table.get(&self.default_route)
        }

    }
}

impl fmt::Debug for Routing {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Routing Table:\n{:?}", self.routing_table)
    }
}

//pair of mac addresses to represent an interface

pub struct InterfaceMacAddrs {
    pub source: MacAddr,
    pub destination: MacAddr,
}

impl InterfaceMacAddrs {
    pub fn new(source: MacAddr, destination: MacAddr) -> InterfaceMacAddrs {
        InterfaceMacAddrs{source, destination}
    }
}

impl fmt::Debug for InterfaceMacAddrs {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?},{:?})", self.source, self.destination)
    }
}

