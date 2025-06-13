# Fancy Edge Scrolling
Fancy edge scrolling behaviour for linux touchpads.

Left edge: increase/decrease volume.
Right edge: increase/decrease brightness.


Very Technical Requirements:
- a touchpad device with touchpad in its device name
- wpctl used for volume
- brightnessctl used for brightness

If you feel motivated to set this up for yourself:

Create edge-scroll.service in /etc/systemd/user/

For example:
```bash
nvim /etc/systemd/user/edge-scroll.service
```
then paste

```bash
[Unit]
Description=Fancy edge scrolling
After=default.target

[Service]
ExecStart=/usr/local/bin/edge-scroll

[Install]
WantedBy=default.target
```

ExecStart is where you specificy the location of your scroll-edge binary.

If you're lucky you can probably just download the binary from this github (target/release/edge-scroll) and put it somewhere like /usr/local/bin/edge-scroll.

Anyway, if this all works just run the following commands:
```
usermod -aG input $USER
```
```
systemctl --user daemon-reload
```
```
systemctl --user enable edge-scroll
```
```
systemctl --user start edge-scroll
```

That's it!

Good luck...

---

Dev commands I keep forgetting:
```bash
libinput list-devices
```
```bash
cp target/release/edge-scroll /usr/local/bin/edge-scroll
```
