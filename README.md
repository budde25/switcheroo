<div class="oranda-hide">

# Switcheroo

</div>

![GitHub](https://img.shields.io/github/license/budde25/switcheroo?style=for-the-badge)
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/budde25/switcheroo?style=for-the-badge)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/budde25/switcheroo/ci.yml?label=CI&style=for-the-badge&branch=main)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/budde25/switcheroo/release.yml?label=CD&style=for-the-badge)

A cross-platform CLI and GUI for the RCM BootRom exploit (Fusée Gelée exploit for Nintendo Switch).  
Written in Rust using [clap](https://github.com/clap-rs/clap) for the CLI and [egui](https://github.com/emilk/egui) for the GUI.

Only works on unpatched Switches: <https://ismyswitchpatched.com/>

## Features

* CLI interface
* GUI interface
* Works on macOS, Linux, and Windows
* A favorites tab for saving payloads

</br>
<p align="center">
<img width="" alt="Command Line Interface Example" src="https://raw.githubusercontent.com/budde25/switcheroo/main/extra/images/cli.png">
</p>

</br>
<p align="center">
<img width="" alt="Graphical User Interface Example" src="https://raw.githubusercontent.com/budde25/switcheroo/main/extra/images/gui.png">
</p>

## Installation

Download and install the latest release on the [releases](https://github.com/budde25/switcheroo/releases) page  

or install with cargo (binary name is switcheroo)  
`cargo install switcheroo-nx`  

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

See the troubleshooting page

## Similar projects

Here are some other similar projects

* [TegraRcmGUI](https://github.com/eliboa/TegraRcmGUI) GUI for Windows
* [Fusée Launcher](https://github.com/Cease-and-DeSwitch/fusee-launcher) CLI for (Linux, Windows, MacOS)
* [NXBoot](https://mologie.github.io/nxboot/) (macOS, iOS)
* [JTegraNX](https://github.com/dylwedma11748/JTegraNX) Java GUI for (Windows, OS X, GNU/Linux)
* [NXLoader](https://github.com/DavidBuchanan314/NXLoader) (Android)
* [Web Fusée Launcher](https://fusee-gelee.firebaseapp.com/) Web App (only Chrome)

## Credit

Implementation is largely based on the following reference implementation:
[Fusee Launcher](https://github.com/Qyriad/fusee-launcher)  
Gui design inspired from the great:
[TegraRcmGui](https://github.com/eliboa/TegraRcmGUI)  

## License

[GPL-2.0 License](LICENSE)
