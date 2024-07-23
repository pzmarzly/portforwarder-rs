// Copyright (c) 2018 Paweł Zmarzły
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

extern crate ctrlc;
extern crate portforwarder_rs;

use portforwarder_rs::*;
use std::net::Ipv4Addr;

enum InterfaceStrategy {
    Any,
    Explicit(Ipv4Addr),
}

fn main() {
    let mut iter = std::env::args().skip(1);
    let interface = match iter.next() {
        Some(ref i) if i == "any" => InterfaceStrategy::Any,
        Some(i) => {
            if let Ok(x) = i.parse::<Ipv4Addr>() {
                InterfaceStrategy::Explicit(x)
            } else {
                eprintln!("Error! First argument must be a network interface IP or `any`!");
                return;
            }
        }
        _ => {
            eprintln!("Error! First argument must be a network interface IP or `any`!");
            return;
        }
    };
    if iter.len() == 0 {
        eprintln!("Error! No ports specified!");
        return;
    }
    let mut parsed_ports = Vec::with_capacity(iter.len());
    for port in iter {
        let port_split = port.split('/').collect::<Vec<_>>();
        match &port_split[..] {
            [protocol, internal, external] => {
                let proto = match &protocol.to_lowercase()[..] {
                    "tcp" => port_forwarder::PortMappingProtocol::TCP,
                    "udp" => port_forwarder::PortMappingProtocol::UDP,
                    _ => {
                        eprintln!("Error! Unrecognized protocol: {} (in {})", protocol, port);
                        return;
                    }
                };
                let internal = match internal.parse::<u16>() {
                    Ok(num) => num,
                    Err(err) => {
                        eprintln!(
                            "Error! Invalid internal port number: {} (in {}) - {}",
                            internal, port, err
                        );
                        return;
                    }
                };
                let external = match external.parse::<u16>() {
                    Ok(num) => num,
                    Err(err) => {
                        eprintln!(
                            "Error! Invalid external port number: {} (in {}) - {}",
                            external, port, err
                        );
                        return;
                    }
                };
                parsed_ports.push((proto, internal, external));
            }
            _ => {
                eprintln!(
                    "Error! Port not in {{TCP,UDP}}/INTERNAL/EXTERNAL format: {}",
                    port
                );
                return;
            }
        }
    }

    let forwarder = match interface {
        InterfaceStrategy::Any => {
            let interfaces = query_interfaces::get_network_interfaces()
                .expect("Failed to load network interfaces");
            let interface_ips = interfaces.iter().map(|i| i.addr);
            port_forwarder::create_forwarder_from_any(interface_ips)
        }
        InterfaceStrategy::Explicit(i) => port_forwarder::create_forwarder(i).map_err(|e| vec![e]),
    };

    let mut forwarder = match forwarder {
        Ok(f) => f,
        Err(err) => {
            eprintln!(
                "Error! Failed to connect to UPnP-enabled device! List of errors: {:?}",
                err
            );
            return;
        }
    };

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    for port in parsed_ports {
        match forwarder.forward_port(port.1, port.2, port.0, "PortForwardRs") {
            Ok(_) => println!(
                "{} {}:{} -> {}:{}",
                port.0,
                forwarder.gateway.addr.ip(),
                port.2,
                forwarder.network_interface,
                port.1
            ),
            Err(err) => eprintln!(
                "Error! Could map {} {}:{} -> {}:{} - {}",
                port.0,
                forwarder.gateway.addr.ip(),
                port.2,
                forwarder.network_interface,
                port.1,
                err
            ),
        }
    }

    println!("Going to sleep... Press Ctrl-C to close program.");
    while running.load(Ordering::SeqCst) {}
    println!("Shutting down...");
}
