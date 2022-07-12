# Switcheroo

[![License](https://flat.badgen.net/badge/license/GPL-2.0/blue)](LICENSE)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/budde25/switcheroo/CI?label=CI&style=flat-square)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/budde25/switcheroo/CD?label=CD%20MacOS%20Linux-amd64&style=flat-square)
![CircleCI](https://img.shields.io/circleci/build/github/budde25/switcheroo/main?label=CD%20Linux-arm64&style=flat-square)

A cross platform CLI and GUI for the RCM BootRom exploit (Fusée Gelée exploit for Nintendo Switch)

Only works on unpatched Switches: <https://ismyswitchpatched.com/>

Written in Rust using [clap](https://github.com/clap-rs/clap) for the CLI and [egui](https://github.com/emilk/egui) for the GUI.

## Features

* CLI interface
* GUI interface
* Works on MacOS, Linux, and Windows
* A favorites tab for saving payloads

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

### Linux: Flatpak not detecting Switch

Due to a limitation of flatpak not allowing access to udev, the flatpak version only works if the Switch is already in rcm mode and plugged in when it is launched.
If the application is launched without these conditions being met, it will never show `RCM OK`

### Windows: Wrong driver error

On windows the rcm connection will only work if the Switch is using the libusbK drivers.  
The easiest way to install them is to plug in the switch in RCM mode and use [zadig](https://zadig.akeo.ie/) to install the correct driver  

## Similar projects

Here are some other similar projects

* [TegraRcmGUI](https://github.com/eliboa/TegraRcmGUI) GUI for Windows
* [Fusée Launcher](https://github.com/Cease-and-DeSwitch/fusee-launcher) CLI for (Linux, Windows, MacOS)
* [NXBoot](https://mologie.github.io/nxboot/) (MacOS, iOS)
* [JTegraNX](https://github.com/dylwedma11748/JTegraNX) Java GUI for (Windows, OS X, GNU/Linux)
* [NXLoader](https://github.com/DavidBuchanan314/NXLoader) (Android)
* [Web Fusée Launcher](https://fusee-gelee.firebaseapp.com/) Web App (only Chrome)

## Credit

Implementation is largely based on the following reference implementation:
[Fusee Launcher](https://github.com/Qyriad/fusee-launcher)  
Gui design inspired from the great:
[TegraRcmGui](https://github.com/eliboa/TegraRcmGUI)  

## License

[GPL-2.0 License](LICENSE-APACHE)
