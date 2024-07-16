<div class="oranda-hide">

# Troubleshooting

</div>


### Linux: Permission denied error

On many linux systems the usb control is blocked by default.
This is also the case for the flatpak version of the application. These rules must be added outside of the sandbox.

This can be fixed by adding the following file on your system:
`/etc/udev/rules.d/99-switch.rules`
with the following content:

```
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="3000", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="0955", ATTRS{idProduct}=="7321", MODE="0666"

```

alternatively run this command command (will prompt for password)
```sh
printf 'SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="3000", MODE="0666"\nSUBSYSTEM=="usb", ATTRS{idVendor}=="0955", ATTRS{idProduct}=="7321", MODE="0666"\n' | sudo tee /etc/udev/rules.d/99-switch.rules
```

then reload the udev rules:
`sudo udevadm control --reload-rules`

then unplug and plug back in the switch and restart the application.

### Windows: Wrong driver error

On windows the rcm connection will only work if the Switch is using the libusbK drivers.
The easiest way to install them is to plug in the switch in RCM mode and use [zadig](https://zadig.akeo.ie/) to install the correct driver.
