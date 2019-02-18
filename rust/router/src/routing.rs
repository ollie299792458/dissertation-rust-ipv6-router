use std::collections::HashMap;
use std::net::Ipv6Addr;
use pnet::util::MacAddr;
use std::fmt;

pub struct Routing {
    //hash table for now, maybe move to something more complex
    routing_table:HashMap<Ipv6Addr, MacAddr>,
    default_route:Ipv6Addr,
}

impl Routing {
    pub fn new(default_route_ip6: Ipv6Addr, default_route_mac: MacAddr) -> Routing {
        let mut routing_table = HashMap::new();
        routing_table.insert(default_route_ip6,default_route_mac);
        Routing { routing_table, default_route: default_route_ip6}
    }
    pub fn add_route(&mut self, ip6:Ipv6Addr, mac:MacAddr) -> Result<(), WorkError> {
        if ip6 != self.default_route {
            res = self.routing_table.insert(ip6, mac);
            Ok(())
        } else {
            Err("Can't add/update default route with add_route()")
        }
    }


    //todo add routers ip as a special case
    //todo update_default_route - or maybe rethink the whole default route semantics

    pub fn get_route(&self, ip6:Ipv6Addr) -> Option<&MacAddr> {
        match self.routing_table.get(&ip6) {
            Some(mac) => Some(mac),
            None => self.routing_table.get(&self.default_route)
        }

    }
}

impl fmt::Debug for Routing {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Routing Table:\n{:?}", self.routing_table)
    }
}

