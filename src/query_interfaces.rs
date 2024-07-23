// Copyright (c) 2018 Paweł Zmarzły
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use get_if_addrs::{get_if_addrs, IfAddr};

use std::io;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub addr: Ipv4Addr,
}

pub fn get_network_interfaces() -> Result<Vec<Interface>, io::Error> {
    get_if_addrs().map(|interfaces| {
        interfaces
            .into_iter()
            .filter_map(|interface| {
                if let IfAddr::V4(ref addr) = interface.addr {
                    if addr.is_loopback() {
                        return None;
                    }
                    Some(Interface {
                        name: interface.name,
                        addr: addr.ip,
                    })
                } else {
                    None
                }
            })
            .collect()
    })
}
