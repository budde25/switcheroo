<div class="oranda-hide">

# Troubleshooting

</div>


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
If the application is launched without these conditions being met, it will never show `RCM OK`.  

### Windows: Wrong driver error

On windows the rcm connection will only work if the Switch is using the libusbK drivers.  
The easiest way to install them is to plug in the switch in RCM mode and use [zadig](https://zadig.akeo.ie/) to install the correct driver.  