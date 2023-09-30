# Tegra RCM

A library to help exploit the bootROM exploit for the Tegra X1's RCM mode.  

Currently compatible with Linux and macOS.  
Windows support is WIP.

## Example

```rust
    use std::fs;
    use tegra_rcm::{Payload, Rcm};

    let payload_bytes = fs::read(&payload).unwrap();
    let payload = Payload::new(&payload_bytes).unwrap();
    let mut switch = Rcm::new(wait).unwrap();
    // Init the switch device (should only be done once)
    switch.init().unwrap();
    // We MUST to read the device id first
    let _ = switch.read_device_id().unwrap();
    switch.execute(&payload).unwrap();
    println!("Done!");

```
