# RMK for ZSA Moonlander Mark I rev A

Firmware for the 72-key [ZSA Moonlander Mark I](https://www.zsa.io/moonlander),
adapted from rmk-zsa-voyager using [RMK](https://rmk.rs) and [Embassy](https://embassy.dev).

This build supports **rev A**, identified in DFU mode as STM32 Bootloader /
STM Device in DFU Mode (0483:df11). The application starts at 0x08000000;
the bootloader resides in STM32 system ROM. Rev B (Moonlander Bootloader,
3297:2003) requires a different build.

## Features

- Live keymap editing and persistent settings through [Vial](https://get.vial.today/).
- Per-key RGB on all 72 switches, with six effects and 16 palettes from
  [rmk-palettefx](https://github.com/jpds/rmk-palettefx).
- Six status LEDs showing the active layer as a mirrored 3-bit value.
- Chordal hold: same-hand key presses resolve mod-taps as taps; thumbs are excluded.

## Keymap

Five layers are editable through Vial:

- 0 EN: QWERTY, with B/N on the fourth alpha row and modifiers/arrows below.
- 1 RU: ЙЦУКЕН on the same keys as EN.
- 2 SYM: symbols, brackets, function keys and a right-hand numpad.
- 3 NAV: navigation, media, RGB and ROM DFU.
- 4 SYM-RU: SYM for the RU layer.

### Layout-independent symbols

Every symbol is on the same physical key in EN and RU. The layout key
(right thumb) presses Caps Lock and switches layer 0 ↔ 1 together, so the
firmware always knows which host layout is active. On the RU layers:

- symbols that exist in ЙЦУКЕН (`" ; : / .`) are sent at their Russian positions;
- symbols missing in ЙЦУКЕН (`@ # $ ^ & [ ] { } ' \``) are typed by a macro that
  switches the host to EN, types the EN key, and switches back (`src/layout_switch.rs`);
- Х Ъ Ё Э are vertical combos: Y+H, J+M, F+R and H+N. Hold Shift for capitals.

Host requirements: Caps Lock must toggle EN/RU, and the Russian layout must be the
standard ЙЦУКЕН ("Russian" on Windows, `ru` on Linux, "Russian – PC" on macOS).
If the host layout is changed another way, the layers drift; press the layout key
to resync.

Caps Lock is held for 80 ms per switch (`CAPS_HOLD_MS`), because macOS ignores
shorter presses, and each switch gets 80 ms to settle (`LAYOUT_SETTLE_MS`).
Raise these if a host drops or mistypes wrapped symbols. Keys pressed while a
symbol is being typed are queued and sent afterwards, in order.

Holding a wrapped symbol repeats it, as on the EN layer: the host stays in EN
with the key held until any key is released or pressed. After a long hold,
pause briefly before typing on; a key pressed within ~160 ms of the release can
still come out in EN.

The additional right-hand column is the inner column: 6/Y/H start at column 1.
Thumb assignments, in matrix column order:

| Half | Coordinate | Tap | Hold |
|------|------------|-----|------|
| Left | (5,0) | Enter | SYM |
| Left | (5,1) | Tab | Left Ctrl |
| Left | (5,2) | Delete | Delete |
| Left, red key | (5,3) | Left GUI (Win/Cmd) | Left GUI |
| Right, red key | (11,3) | Escape | Escape |
| Right | (11,4) | Tab | Tab |
| Right | (11,5) | Backspace | Right Shift |
| Right | (11,6) | Space | NAV |

On SYM, the Space thumb emits 0; other thumbs inherit BASE on SYM/NAV.
Hold Space (NAV) and release the top-right key to enter ROM DFU. The DFU action
is User 0 (VIA code 0x7E00), named DFU in Vial. Use it instead of RMK's generic
Bootloader action, which has no STM32 ROM backend in RMK 0.9.

## Build and flash

Enter the Nix development shell with `nix develop`, or install Rust,
the `thumbv7em-none-eabihf` target, cargo-binutils, llvm-tools and dfu-util.
Ruby is needed for the checks below.

```sh
cargo objcopy --release -- -O binary moonlander-reva.bin
```

Put the keyboard into DFU using its physical reset button or the NAV DFU key.
Confirm **0483:df11** with `dfu-util -l`, then flash:

```sh
dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D moonlander-reva.bin
```

Do not flash this image to rev B. Both the binary's link address and the flashing
address must be 0x08000000. After flashing, reconnect USB and check both halves.
Normal USB identity is 3297:1969; the serial includes `moonlander:reva`.

## Storage

The firmware keeps no settings in flash: the compiled keymap, combos and macros
are used on every boot, so all 128 KB of flash is available to the application.
To change the layout, edit the source and reflash.

## Hardware reference

The left matrix uses direct GPIO, the right uses an MCP23018, and RGB uses two
IS31FL3731 drivers. Coordinate and RGB channel tables follow
[QMK's Moonlander definition](https://github.com/qmk/qmk_firmware/tree/master/keyboards/zsa/moonlander).

## License

Licensed under [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

This project is not affiliated with or endorsed by ZSA Technology Labs, Inc.
