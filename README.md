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

Three layers are editable through Vial:

- BASE: QWERTY, with B/N on the fourth alpha row and modifiers/arrows below.
- SYM: symbols, brackets, function keys and a right-hand numpad.
- NAV: navigation, media, RGB and ROM DFU.

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

## Storage and recovery

The conservative STM32F303CB target uses 128 KB of flash and 32 KB of main RAM.
The last 12 KB of flash contain a 2 KB layout-version page followed by a 10 KB
journal. Keep `memory.x` and `src/storage_layout.rs` consistent.

A layout-version mismatch resets saved Vial configuration to compiled defaults
once. Subsequent boots preserve edits. Export settings before changing firmware;
an old Voyager layout cannot be imported unchanged because its coordinates differ.
RMK can also reset settings when its own build hash changes.

To explicitly restore defaults, build a recovery image:

```sh
cargo objcopy --release --features reset-layout -- -O binary moonlander-reset-layout.bin
```

The recovery image resets the layout on every boot. Boot it once, then flash a
normal image.

## Checks

Build the normal image first so the ELF and binary match, then run:

```sh
ruby tests/check_keymap.rb
ruby tests/check_reva_image.rb
cargo test --manifest-path tests/storage-capacity/Cargo.toml --target "$(rustc -vV | sed -n 's/^host: //p')"
```

The checks cover physical key placement, vector addresses, storage boundaries,
journal capacity, updates and reopening. The firmware build also checks Vial
geometry and RGB channel uniqueness.

## Hardware reference

The left matrix uses direct GPIO, the right uses an MCP23018, and RGB uses two
IS31FL3731 drivers. Coordinate and RGB channel tables follow
[QMK's Moonlander definition](https://github.com/qmk/qmk_firmware/tree/master/keyboards/zsa/moonlander).

## License

Licensed under [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

This project is not affiliated with or endorsed by ZSA Technology Labs, Inc.
