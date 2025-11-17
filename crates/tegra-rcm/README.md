# Tegra RCM

A library to help exploit the bootROM exploit for the Tegra X1's RCM mode.  

Currently compatible with Linux, macOS, and Windows.

## Example

```rust
use tegra_rcm::{Payload, Switch};

// Load a payload from a file
let payload = Payload::read("payload.bin")?;

// Find and connect to a Switch in RCM mode
let switch = Switch::find()?;

// Execute the payload on the Switch
switch.execute(&payload)?;

println!("Done!");
```
