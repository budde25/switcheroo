# Switcheroo

[![License](https://flat.badgen.net/badge/license/GPL-2.0/blue)](LICENSE)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/budde25/switcheroo/CI?label=CI)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/budde25/switcheroo/CD?label=CD)

A CLI and GUI for the RCM BootRom expoit (Fusée Gelée exploit for Nintendo Switch)

Only works on unpatched Switches: <https://ismyswitchpatched.com/>

Written in Rust using [clap](https://github.com/clap-rs/clap) for the CLI and [egui](https://github.com/emilk/egui) for the GUI.

## Features

* CLI interface
* GUI inferface
* Works on MacOS and Linux (Windows support is WIP)

</br>
<p align="center">
<img width="" alt="Command Line Interface Example" src="https://raw.githubusercontent.com/budde25/switcheroo/main/extra/images/cli.png">
</p>

</br>
<p align="center">
<img width="" alt="Graphical User Interface Example" src="https://raw.githubusercontent.com/budde25/switcheroo/main/extra/images/gui.png">
</p>

## Usage

The binary name is `switcheroo`  

To display application use `switcheroo help`  
Use `switcheroo <subcommand> help` for help with that subcommand.

### Examples

Execute a payload.  
`switcheroo execute <path>`

Check if the switch is connected.  
`switcheroo device`

Start the GUI.  
`switcheroo gui`

## Troubleshooting

### Linux: Permission denied error

On many linux systems the usb control is blocked by default.  

This can be fixed by adding the following file:  
`/etc/udev/rules.d/99-switch.rules`  
with the following content:  

```
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="3000", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="0955", ATTRS{idProduct}=="7321", MODE="0666"
```

then reload the udev rules:  
`sudo udevadm control --reload-rules`

Finally unplug and plug back in the switch.  

## Similar projects

Here are some other simiar projects

* [TegraRcmGUI](https://github.com/eliboa/TegraRcmGUI) GUI for Windows
* [Fusée Launcher](https://github.com/Cease-and-DeSwitch/fusee-launcher) CLI for (Linux, Windows, MacOS)
* [NXBoot](https://mologie.github.io/nxboot/) (MacOS, iOS)
* [JTegraNX](https://github.com/dylwedma11748/JTegraNX) Java GUI for (Windows, OS X, GNU/Linux)
* [NXLoader](https://github.com/DavidBuchanan314/NXLoader) (Android)
* [Web Fusée Launcher](https://fusee-gelee.firebaseapp.com/) Web App (only Chrome)

## Credit

Implemenation is largely based on the following reference implemenatinon:
[Fusee Launcher](https://github.com/Qyriad/fusee-launcher)

## License

[GPL-2.0 License](LICENSE-APACHE)
