# Port forwarder

Connects to UPnP-enabled gateways and redirects ports. Inspired by [portmapper](https://github.com/kaklakariada/portmapper) (written in Java). Abstraction over [`igd` crate](https://crates.io/crates/igd). Tested on Zhone Access Point with 2014 firmware (it cannot list opened ports, so I did not try to implement such feature).

This crate consists of binary `pf` and simple library abstracting over port forwarding and network interface listing.

This crate __HAS NOT BEEN TESTED ON WINDOWS OR MACOS.__

## Building

Install the utility using command:

    cargo install portforwarder-rs

or build from source with:

    git clone https://github.com/pzmarzly/portforwarder-rs
    cd portforwarder-rs
    cargo build --release
    mv target/release/pf <some place>

## Usage

Redirect ports on first-matched device:

    pf any TCP/80/80 UDP/3000/2000 TCP/81/82

Format is `{TCP|UDP}/LOCAL_PORT/REMOTE_PORT`.

Redirect ports on a network interface with specific IPv4 address:

    pf 192.168.254.107 TCP/8080/8080
