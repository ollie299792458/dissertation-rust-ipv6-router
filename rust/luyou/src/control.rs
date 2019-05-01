use std::collections::HashMap;
use std::net::Ipv6Addr;
use pnet::util::MacAddr;
use std::fmt;

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

pub struct Routing {
    //hash table for now, maybe move to something more complex
    routing_table:HashMap<Ipv6Addr, (MacAddr, MacAddr)>,
    mtu_table:HashMap<MacAddr, u32>,
    default_route:Ipv6Addr,
    router_address:Ipv6Addr,
}

impl Routing {
    pub fn new(configuration:String) -> Routing {
        let mut routing_table = HashMap::new();
        let mut mtu_table = HashMap::new();

        let mut lines = configuration.lines();

        //get default route "IPv6" (first line)
        let ipv6_str = lines.next().unwrap();
        let default_route =  match ipv6_str.parse::<Ipv6Addr>() {
            Ok(a) => a,
            Err(e) => {println!("Invalid default IPv6 Address: {}", ipv6_str); Err(e).unwrap()},
        };

        //get router address (second line)
        let ipv6_str = lines.next().unwrap();
        let router_address = match ipv6_str.parse::<Ipv6Addr>() {
            Ok(a) => a,
            Err(e) => {println!("Invalid router IPv6 Address: {}", ipv6_str); Err(e).unwrap()},
        };

        //get hashmap entries "IPv6@MAC,MAC" (inc a line for default route) - lines starting with "mtu" are mtuMTU@ROUTER_MAC lines
        for line in lines {
            if line =="" {
                continue;
            }
            if line.starts_with("mtu") {
                if line == "" {
                    break;
                }
                let mut mtu_lines = line.split("mtu");
                mtu_lines.next();
                let mtu_line = mtu_lines.next().unwrap();
                let mut addrs = mtu_line.split("@");
                let mtu_str = addrs.next().unwrap();
                let mac_str = addrs.next().unwrap();
                let mtu = match mtu_str.parse::<u32>() {
                    Ok(mtu) => mtu,
                    Err(e) => {println!("Invalid mtu: {}", mtu_str); Err(e).unwrap()}
                };
                let mac = match mac_str.parse::<MacAddr>() {
                    Ok(mac) => mac,
                    Err(e) => {println!("Invalid MAC Addresss: {}", mac_str); Err(e).unwrap()}
                };
                mtu_table.insert(mac, mtu);
            } else {
                let mut addrs = line.split("@");
                let ipv6_str = addrs.next().unwrap();
                let ipv6 = match ipv6_str.parse::<Ipv6Addr>() {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Invalid IPv6 Address: {}", ipv6_str);
                        Err(e).unwrap()
                    },
                };
                let mac_addrs_str = addrs.next().unwrap();
                let mut mac_addrs = mac_addrs_str.split(",");
                let source_str = mac_addrs.next().unwrap();
                let destination_str = mac_addrs.next().unwrap();
                let source = match source_str.parse::<MacAddr>() {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Invalid MAC Address: {}", source_str);
                        Err(e).unwrap()
                    },
                };
                let destination = match destination_str.parse::<MacAddr>() {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Invalid MAC Address: {}", destination_str);
                        Err(e).unwrap()
                    },
                };
                let macs = (source, destination);
                routing_table.insert(ipv6, macs);
            }
        }

        Routing { routing_table, mtu_table, default_route, router_address}
    }

    //source, destination
    /*
    pub fn add_route(&mut self, ip6:Ipv6Addr, macs: (MacAddr, MacAddr)) {
        if ip6 != self.default_route {
            self.routing_table.insert(ip6, macs); //todo maybe do something with the result
        } else {
            panic!("Can't add/update default route with add_route()")
        }
    }*/

    //todo add routers ip as a special case
    //todo update_default_route - or maybe rethink the whole default route semantics

    pub fn get_route(&self, ip6:Ipv6Addr) -> (MacAddr, MacAddr) {
        match self.routing_table.get(&ip6) {
            Some((source, destination)) => {//println!("Address looked up: {:?}, result: {:?}", ip6, macs);
                (source.clone(), destination.clone())},
            None => self.routing_table.get(&self.default_route).unwrap().clone()
        }

    }

    pub fn get_router_address(&self) -> Ipv6Addr {
        return self.router_address;
    }

    pub fn get_mtu(&self, mac:MacAddr) -> u32{
        match self.mtu_table.get(&mac) {
            Some(mtu) => mtu.clone(),
            None => u32::max_value(),
        }
    }
}

impl fmt::Debug for Routing {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Routing Table:\n{:?}", self.routing_table)
    }
}