# switch-rcm

### Troubleshooting

`/etc/udev/rules.d/99-switch.rules`

```
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="3000", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="0955", ATTRS{idProduct}=="7321", MODE="0666"
```

`udevadm control --reload-rules`
