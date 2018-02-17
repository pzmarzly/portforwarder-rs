// Copyright (c) 2018 Paweł Zmarzły
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use igd::{ self, SearchError, AddAnyPortError, AddPortError, RemovePortError };
use std::net::{ SocketAddrV4, Ipv4Addr };

pub use igd::PortMappingProtocol;

#[derive(Debug)]
pub struct Port {
    pub proto: PortMappingProtocol,
    pub num: u16
}

impl PartialEq for Port {
    fn eq(&self, other: &Port) -> bool {
        self.proto == other.proto && self.num == other.num
    }
}

#[derive(Debug)]
pub struct Forwarder {
    pub gateway: igd::Gateway,
    pub network_interface: Ipv4Addr,
    pub open_ports: Vec<Port>
}

pub fn create_forwarder(interface_ip: Ipv4Addr) -> Result<Forwarder, SearchError> {
    igd::search_gateway_from(interface_ip).map(|gateway|
        Forwarder {
            gateway: gateway,
            network_interface: interface_ip,
            open_ports: Vec::new()
        }
    )
}

pub fn create_forwarder_from_any<I>(interface_ips: I) -> Result<Forwarder, Vec<SearchError>>
    where
        I: IntoIterator<Item = Ipv4Addr>, {
    let mut errors = Vec::new();
    for interface_ip in interface_ips {
        match create_forwarder(interface_ip) {
            Ok(forwarder) => return Ok(forwarder),
            Err(error) => errors.push(error)
        }
    }
    Err(errors)
}

impl Forwarder {
    pub fn forward_any_port(&mut self, local_port: u16, proto: PortMappingProtocol, name: &str) -> Result<u16, AddAnyPortError> {
        self.gateway
            .add_any_port(proto, SocketAddrV4::new(self.network_interface, local_port), 0, name)
            .map(|port| {
                self.open_ports.push(Port { proto: proto, num: port });
                port
            })
    }
    pub fn forward_port(&mut self, local_port: u16, remote_port: u16, proto: PortMappingProtocol, name: &str) -> Result<(), AddPortError> {
        self.gateway
            .add_port(proto, remote_port, SocketAddrV4::new(self.network_interface, local_port), 0, name)
            .map(|()| {
                self.open_ports.push(Port { proto: proto, num: remote_port });
            })
    }
    pub fn remove_port(&mut self, remote_port: u16, proto: PortMappingProtocol) -> Result<(), RemovePortError> {
        if let Some(pos) = self.open_ports.iter().position(|el| *el == Port { proto: proto, num: remote_port }) {
            self.open_ports.remove(pos);
        } else {
            println!("Remote port {} {} was not opened by this Forwarder! Removing anyway...", proto, remote_port);
        }
        self.gateway.remove_port(proto, remote_port)
    }
}

impl Drop for Forwarder {
    fn drop(&mut self) {
        println!("Closing open ports...");
        for port in &self.open_ports {
            let num = port.num;
            let proto = port.proto;
            println!("Closing port {} {}...", proto, num);
            if self.gateway.remove_port(proto, num).is_err() {
                println!("Failed to close port {} {} on exit!", proto, num);
            }
        }
    }
}
