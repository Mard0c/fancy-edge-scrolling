# Fancy Edge Scrolling
Fancy edge scrolling behaviour for linux touchpads.

Inspired by Asus S16 on windows:
Left edge: increase/decrease volume.
Right edge: increase/decrease brightness.
Top edge: left arrow/right arrow. (very useful when combined with CTRL, skip youtube chapters, jump between words, and probably more!)

Cool additions because I use waybar (could probably also work with waybar alternatives like swaybar):
Pull down from top edge: summon status bar until touch release.

## Very Technical Requirements:
- a touchpad device with touchpad in its device name
- wpctl used for volume
- brightnessctl used for brightness
- ydotool for left/right arrow emulation
- waybar for statusbar

## If you crazy enough to try and set this up for yourself:

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


For those interested (future me who's forgotten how this works) this is the package's high level structure:
main
- persistent data
- main loop
    - x axis processing:
        - vertical (left/right) edge zone detection
        - call horizontal scroll (scrub left/right)
    - y axis processing
        - horizontal (top) edge zone detection
        - call vertical scroll (adjust brightness / adjust volume)
    - touch down event processing
    - touch up event processing
commands
- terminal commands for corresponding gestures
