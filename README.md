# LegionGlow

LegionGlow is a Linux desktop GUI for managing keyboard backlight and RGB lighting patterns on Lenovo Legion and Lenovo LOQ laptops.

## Current Status

This repository is an initial Tauri v2 foundation. The app uses a mock Linux keyboard backend by default.

An experimental real Lenovo HID backend is available for known Lenovo Legion/LOQ 4-zone RGB keyboards. It is opt-in and uses userspace HID feature reports through `hidapi` with the Linux **hidraw** backend. A dry-run mode is available for detection and payload validation without hardware writes.

Startup is strictly passive: launching LegionGlow only reads DMI and hidraw metadata from sysfs. It never opens, claims, or probes a HID device automatically, and it never detaches kernel drivers.

The current focus is safe real-hardware validation for the Lenovo LOQ 17IRX10 (`048d:c693`). That product ID uses the standard HID LampArray interface for static/off RGB writes. Real writes still require explicitly launching the `lenovo-hid` backend; they are never the default.

Supported target:

- Linux
- Lenovo Legion 4-zone RGB keyboards with known ITE USB HID IDs
- Lenovo LOQ 4-zone RGB keyboards with known ITE USB HID IDs

Not supported yet:

- Legion 7 per-key RGB
- white-only keyboard backlight models
- unknown Lenovo models
- unknown ITE devices
- reactive effects on the real HID backend
- hardware animation effects on HID LampArray devices

## Features

- Dashboard with detected device, backend status, current effect, brightness, and color.
- RGB controls for static, breathing, wave, reactive, rainbow, and off effects.
- Optional 4-zone color editing with a "use same color for all zones" toggle.
- Capability model so unsupported controls can be disabled per device.
- Built-in profiles with backend-computed compatibility (unsupported profiles disable Apply and explain why).
- Diagnostics report from the Rust backend, including a structured HID access probe, payload hex, write-allowlist source, and an in-app validation checklist.
- Staged Settings setup: detect device → fix permissions → dry-run validation → experimental real mode → safe first write.
- Permission Setup panel: preview/copy the udev rule, install/reload/remove it via an in-app password dialog (the password is piped to `sudo -S`, used once, never stored or logged), and a manual-only "Probe HID access" button gated on safe interface identification.
- Safe first-write test payload, gated behind real writes being enabled.
- Mock backend plus experimental Lenovo HID backend with an explicit experimental product ID allowlist.
- Typed React frontend with centralized Tauri API wrappers.

## Architecture

### Backend Layers

The Rust backend is organized for future production hardware support:

- `app/`: bootstrap, dependency wiring, app state, and centralized error handling.
- `commands/`: thin Tauri command handlers that call services and return frontend-safe data.
- `domain/`: pure serializable models for devices, lighting, profiles, diagnostics, and app info.
- `services/`: business logic, validation flow, profile application, and diagnostics orchestration.
- `drivers/`: keyboard driver trait plus mock and experimental Lenovo HID implementations.
- `infrastructure/`: Linux access placeholders, storage repositories, and logging setup.
- `utils/`: validation and color helpers.

Real hardware access must stay behind the `KeyboardDriver` trait and infrastructure modules. The GUI should never run as root.

### Frontend Layers

The React frontend is organized by responsibility:

- `domain/`: TypeScript types matching Rust models.
- `api/`: all Tauri `invoke` calls.
- `app/`: shell, navigation, and app constants.
- `features/`: dashboard, lighting controls, profiles, diagnostics, and settings pages.
- `shared/`: reusable UI components, hooks, and utilities.
- `state/`: lightweight React hook stores.
- `styles/`: global theme and responsive layout CSS.

## Development Setup

Install dependencies:

```bash
npm install
```

Run the app in development. When `LEGIONGLOW_BACKEND` is not set, debug builds default to the safe Lenovo HID dry-run backend:

```bash
npm run tauri dev
```

Force mock mode:

```bash
npm run tauri:mock dev
```

Explicit dry-run mode (same as the default `npm run tauri dev`):

```bash
npm run tauri:dry-run dev
```

Run the experimental Lenovo HID backend with real writes requested:

```bash
npm run tauri:real dev
```

The backend-specific scripts set `LEGIONGLOW_BACKEND` for you. If your shell
or tooling strips the prefixed environment variable, set it inline instead:

```bash
LEGIONGLOW_BACKEND=mock npm run tauri dev
LEGIONGLOW_BACKEND=lenovo-hid-dry-run npm run tauri dev
LEGIONGLOW_BACKEND=lenovo-hid npm run tauri dev
```

Build the app:

```bash
npm run tauri build
```

The default Tauri config keeps all bundle targets enabled. On minimal or containerized Linux systems, AppImage bundling may require working `linuxdeploy`/FUSE support even when the release binary, deb, and rpm outputs build successfully.

Run frontend build checks:

```bash
npm run build
```

Run backend tests:

```bash
cd src-tauri
cargo test
```

## Keyboard input stops after launching

If keyboard input stops working after launching LegionGlow (and keeps not working after closing it):

1. **Stop using the app immediately.**
2. **Reboot to recover.** Rebooting reattaches the kernel driver to the internal keyboard. (On some systems logging out and back in, or `sudo udevadm trigger`, is enough, but a reboot is the reliable fix.)
3. **Do not run real or dry-run HID mode again until you are on a build with the fix.** As an extra guard, launch with the emergency safety flag:

```bash
LEGIONGLOW_DISABLE_HID=1 npm run tauri dev
```

Why this can happen: if a userspace program uses a **libusb**-based HID backend, opening a device can claim the USB interface and **detach the kernel driver from the keyboard's HID interface**. If the driver is not reattached, keyboard input stays dead even after the program exits. LegionGlow therefore uses passive (sysfs-only) detection, the hidraw backend, and strict interface filtering, and never opens keyboard input interfaces.

With `LEGIONGLOW_DISABLE_HID=1` set, no HID library code that can open or claim devices runs at all: detection falls back to DMI/sysfs-only information, diagnostics still work, and the UI shows "HID access disabled by safety flag."

### Developer notes (do not regress)

- **Never use the libusb hidapi backend** (`linux-static-libusb`, `linux-shared-libusb`, or raw libusb) for internal keyboard HID probing. Only `linux-static-hidraw` is allowed.
- **Never open devices by VID/PID alone.** One USB device exposes multiple HID interfaces (keyboard input, consumer control, vendor/RGB control); `HidApi::open(vid, pid)` picks an arbitrary one. Always select the single eligible RGB-control interface (a vendor-defined usage page ≥ `0xff00`, or the standard HID LampArray page `0x59`) by hidraw path.
- **Never open interfaces identified as keyboard input** (usage page `0x01` + usage `0x06`, or usage page `0x07`), and treat interfaces with missing usage metadata as unsafe to open.
- **Never open HID devices at startup or inside diagnostics.** Opening happens only in explicit user-triggered operations, and the handle is dropped immediately.

## Safety Model

LegionGlow defaults to Lenovo HID dry-run mode during `npm run tauri dev`, and defaults to mock mode in release builds unless a backend is explicitly selected. The real write backend is opt-in and intentionally limited. The app does not:

- write to sysfs,
- open any HID device at startup or during diagnostics,
- open keyboard input or consumer-control HID interfaces, ever,
- use a libusb HID backend or detach kernel drivers,
- keep HID handles open across operations,
- require sudo for normal operation,
- run the GUI as root.

The real backend uses userspace HID feature reports through the Linux hidraw backend. Startup and diagnostics are passive (DMI + hidraw sysfs metadata only). HID devices are opened only in two explicit cases: the manual **Probe HID access** button and real writes — both open only the single eligible RGB-control interface (vendor-defined or HID LampArray) and drop the handle immediately. When the eligible interface is a LampArray, the probe additionally reads its read-only attributes report (lamp count, kind) before closing. The Lenovo vendor write protocol is never sent to a LampArray interface; LampArray devices use standard LampArray control and update reports. Dry-run mode builds and previews payload bytes but does not send feature reports and does not open anything.

Emergency switch: `LEGIONGLOW_DISABLE_HID=1` blocks all active HID access (probe and writes) and limits detection to DMI/sysfs-only information.

## Mock Backend

The mock backend reports a Lenovo Legion device by default. To test LOQ-style reduced capabilities, run with:

```bash
LEGIONGLOW_MOCK_DEVICE=loq npm run tauri dev
```

The mock driver stores lighting state in memory and exposes realistic capabilities for UI development.

## Experimental Lenovo HID Backend

The real backend is selected only when `LEGIONGLOW_BACKEND=lenovo-hid` is set. Use `LEGIONGLOW_BACKEND=lenovo-hid-dry-run` first to verify detection, diagnostics, and payload generation without hardware writes.

The backend detects known ITE HID devices with vendor ID `0x048d` and these initial product IDs:

- `0xc995`: 2024 Pro
- `0xc994`: 2024
- `0xc993`: 2024 LOQ
- `0xc985`: 2023 Pro
- `0xc984`: 2023
- `0xc983`: 2023 LOQ
- `0xc975`: 2022
- `0xc965`: 2021
- `0xc955`: 2020
- `0xc693`: LOQ 17IRX10, HID LampArray static/off path

Known IdeaPad-style IDs and observed unverified IDs are visible in diagnostics but are not enabled for writes in this initial Legion/LOQ backend.

Inspect local device IDs:

```bash
lsusb | grep -i 048d
cat /sys/class/dmi/id/product_name
cat /sys/class/dmi/id/sys_vendor
```

The ITE vendor protocol implementation sends 33-byte feature reports with the public ITE/Lenovo 4-zone facts documented in this repository. The LampArray implementation sends standard HID LampArray control and multi-update feature reports using report IDs parsed from the passive descriptor. The implementation is clean-room from public protocol facts; no GPL project source code was copied.

### State of `048d:c693` (Lenovo LOQ 17IRX10)

`048d:c693` is recognized as a Lenovo LOQ 17IRX10 HID LampArray keyboard. Real writes are enabled only in `LEGIONGLOW_BACKEND=lenovo-hid` mode and only when exactly one safe LampArray interface is identified from passive metadata. Installing the udev rule fixes permissions only; it does not by itself enable writes.

Observed interface layout on real LOQ 17IRX10 hardware (USB device `048d:c693`, 2 HID interfaces):

- **Interface 0** is declared as a USB boot-protocol keyboard, and its report descriptor is composite: vendor-defined collections (`0xff89`, `0xff99`) **plus** the Generic Desktop Keyboard collection that backs the laptop's actual keyboard input. This is the interface a libusb backend would claim, detaching the kernel driver and killing keyboard input. LegionGlow classifies it `is_keyboard_input` and never opens it, even though it also contains vendor collections.
- **Interface 1** exposes the standard HID Lighting & Illumination page (`0x59`, LampArray) — the standardized lighting interface, fully separate from the keyboard interface. It is the single probe-eligible interface on this device.

The manual access probe therefore targets interface 1 only: it opens the LampArray hidraw node, reads the read-only `LampArrayAttributes` feature report (lamp count, kind, update interval), and closes the handle. Interface 0 is never opened. Real writes for `048d:c693` also target interface 1 only: the driver sends `AutonomousMode=false`, then a standard LampArray multi-update report for static colors. Interface 0 remains blocked because it carries keyboard input.

## Experimental product ID override

For advanced manual validation, you can explicitly allow real writes for a specific product ID that is not in the built-in safe write list. This is opt-in, never wildcarded, and intended for users validating their own hardware.

```bash
LEGIONGLOW_BACKEND=lenovo-hid \
LEGIONGLOW_EXPERIMENTAL_ALLOW_PRODUCT_IDS=048d:ffff \
npm run tauri dev
```

Rules:

- Both `048d:ffff` and `0x048d:0xffff` forms are accepted; casing is normalized.
- Multiple IDs are comma-separated: `048d:ffff,048d:c993`.
- Wildcards (`048d:*`), vendor-wide patterns, and `all` are rejected.
- Real writes still require `LEGIONGLOW_BACKEND=lenovo-hid`. The override alone, under dry-run, does not send anything.

Diagnostics will show **Experimental product ID override active**, the **Write allowlist source** (`built-in`, `environment override`, or `blocked`), and whether real writes are effectively enabled for the detected product/interface. When the override is active, diagnostics also surface a strong warning, because real HID feature reports can be sent to an unverified product.

> ⚠️ This sends real HID feature reports to hardware whose protocol has not been verified. Use it only for deliberate, manual validation of your own device, after dry-run and permission checks pass.

## Safe first-write process

Before applying normal effects on a real device, send the lowest-risk payload first. **Settings → Staged setup → Stage 5: Safe first write** has a **Send safe test payload** button (disabled unless real writes are actually enabled for the detected product and interface). It sends a single static, very dim blue frame at the minimum hardware brightness level — no animations.

Recommended order:

1. Run dry-run and confirm payload bytes are generated as expected.
2. Resolve any HID permission issue (Settings → Stage 2: Fix permissions).
3. Switch to real mode. Built-in products use `LEGIONGLOW_BACKEND=lenovo-hid`; experimental products also need the product ID override.
4. Click **Send safe test payload** and confirm the keyboard responds.
5. Only then test the effects the detected capability model enables. For `048d:c693`, that is Static, per-zone Static, brightness, and Off.

## 4-zone support

The hardware is 4-zone RGB. `KeyboardState` carries an optional, backwards-compatible `zone_colors` list (zero-based indices `0..=3`). The Effects page shows an **Advanced zones** section with four color pickers and a **Use same color for all zones** toggle when the device reports `supports_zones` and `zone_count == 4`.

- Static and Breathing honor per-zone colors on the ITE vendor protocol when a complete set of four valid zones is supplied; otherwise they fall back to the primary color for all zones.
- HID LampArray devices currently support Static, per-zone Static, brightness, and Off.
- Wave and Rainbow ignore per-zone colors on the ITE vendor protocol.
- Zone colors are validated for range, duplicates, and zone count; the backend is the final authority.

## Reporting diagnostics

Use the **Copy JSON** button on the diagnostics page to copy the full report (DMI info, HID IDs, access probe, payload hex, write status, allowlist source, warnings, and notes). Include it when reporting issues, along with your product ID, laptop model, kernel version, and desktop session.

## Linux HID Permissions

Do not run the whole Tauri GUI as root. LegionGlow is designed to use userspace HID access from a normal desktop session.

Some Linux systems restrict non-root access to HID devices. If diagnostics show a supported Lenovo HID device but report that it could not be opened, a udev rule is usually needed for your user. The diagnostics page now includes a structured **HID access probe** that classifies the failure (permission denied, device busy, backend unavailable, or unknown) and gives a recommended next action. When the HID backend returns a vague error (for example `hid_error is not implemented yet`), LegionGlow only infers a permission issue when the device was detected, opening failed, and the GUI is running as a normal non-root user.

## Fixing HID permissions from the app

Open **Settings → Staged setup → Stage 2: Fix permissions** (the Permission Setup panel). It lets you fix non-root HID access without leaving the app.

What it does:

- **Preview udev rule** / **Copy udev rule** — show and copy the exact single rule line that will be installed.
- **Copy manual install commands** — copy safe terminal commands if you prefer to do it yourself.
- **Install udev rule** — after a confirmation screen, you enter your system password in an in-app dialog. LegionGlow generates the rule, writes it to a temp file, and runs `sudo -S install -m 0644 <tempfile> /etc/udev/rules.d/99-legionglow-lenovo-rgb.rules`, piping the password to `sudo` over stdin.
- **Reload udev rules** — runs `udevadm control --reload-rules` then `udevadm trigger` through `sudo -S`.
- **Probe HID access** (manual only) — shows a warning first: it briefly opens the single eligible RGB-control interface (vendor-defined or HID LampArray), then closes it immediately. For a LampArray it also reads the read-only attributes report (lamp count, kind) while open. It never sends RGB data, never opens keyboard input interfaces, never enables real writes, and never changes lighting. The button is disabled unless exactly one safe RGB-control interface is identified.
- **Remove LegionGlow udev rule** — after a confirmation screen, removes only `99-legionglow-lenovo-rgb.rules` and reloads rules.
- **Copy diagnostics JSON** — copy the latest report for bug reports.

How the password is handled:

- You type your password into the confirmation dialog. It is sent to the backend, **piped to `sudo -S` over stdin**, used once for that single command, and then dropped. It is **never written to disk, never logged, and never placed on the process command line** (so it does not appear in `ps`). The dialog input is a masked password field and is cleared after use.
- An incorrect password is detected and reported so you can retry without restarting the flow.
- If `sudo` is unavailable, the app returns a clean message and shows the manual commands instead.
- The suggested rule prefers the `uaccess` tag so the active local session is granted access without group management:

```text
SUBSYSTEM=="usb", ATTR{idVendor}=="048d", ATTR{idProduct}=="c693", TAG+="uaccess"
```

After installing, reload rules and reconnect the device (or relog/reboot), then click **Probe HID access**. Stay in dry-run mode until the probe reports `can_open: true`. Fixed permissions mean only that the hidraw node is readable — they do not mean the device is safe to open, and they never enable writes.

## Manual fallback

If you prefer the terminal (or `sudo` is unavailable in the app), the equivalent commands are:

```bash
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="048d", ATTR{idProduct}=="c693", TAG+="uaccess"' \
  | sudo tee /etc/udev/rules.d/99-legionglow-lenovo-rgb.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
# then reconnect the device (or relog/reboot) and restart LegionGlow
```

To remove it again:

```bash
sudo rm -f /etc/udev/rules.d/99-legionglow-lenovo-rgb.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Inspect USB HID IDs and laptop model data:

```bash
lsusb | grep -i 048d
cat /sys/class/dmi/id/product_name
cat /sys/class/dmi/id/sys_vendor
```

## Security model

- **No root GUI.** LegionGlow uses userspace HID access from a normal desktop session; the whole GUI is never run as root. Only the specific `install`/`rm`/`udevadm` commands are elevated via `sudo`.
- **Minimal, transient password handling.** Privileged actions use `sudo -S`. You enter your password in an in-app masked dialog; it is piped to `sudo` over stdin, used once, and then dropped. It is never written to disk, never logged, and never placed on a command line (so it does not appear in `ps`). It is not cached by the app, and an incorrect password is reported for retry. (Note: this is a deliberate trade-off chosen for reliability across desktop environments where a graphical Polkit agent may be absent.)
- **No silent privilege escalation.** Privileged actions never run without an explicit confirmation screen and your password — install, reload, and remove each require confirmation.
- **No shell injection.** Privileged commands are run via `std::process::Command` with an explicit argument vector (no shell, no string interpolation of untrusted input). The password is passed only on stdin, never as an argument. The rule is generated by the backend; the frontend cannot supply rules, IDs, or filenames.
- **Exact, product-specific rule.** Vendor/product IDs are strictly validated (e.g. `048d:c693` or `0x048d:0xc693`). Wildcards, vendor-wide patterns (`048d:*`), and `all` are rejected. The rule grants access to exactly one device, not to all Lenovo/ITE hardware.
- **Constant filename and destination.** The managed rule is always `/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules`; removal only ever targets that file.
- **Permissions are separate from real writes.** Fixing access does not enable real writes. Real writes still require the real backend (`LEGIONGLOW_BACKEND=lenovo-hid`), a product in the built-in allowlist or an explicit `LEGIONGLOW_EXPERIMENTAL_ALLOW_PRODUCT_IDS` override, and the safe test payload.

## Manual Hardware Validation Checklist

1. Start dry-run mode with `npm run tauri dev`.
2. Optionally force mock mode with `LEGIONGLOW_BACKEND=mock npm run tauri dev`.
3. Open diagnostics and verify backend mode, warnings, and notes.
4. Verify detected vendor/product ID, usually `0x048d:<product>`.
5. Verify DMI laptop model and vendor.
6. Apply a safe Static profile in dry-run mode and verify the generated payload bytes.
7. Only then start real mode with `npm run tauri:real dev`.
8. Test Static red, green, and blue.
9. Test brightness low and high.
10. Test Off.
11. Test only the additional effects enabled by the detected capability model.
12. Record product ID, laptop model, kernel version, and desktop session.
13. Report failures with diagnostics JSON and the last payload hex.

## Future Lenovo Integration Plan

- Expand and verify HID protocol handling for more Legion and LOQ generations.
- Add safer model-specific capability detection using DMI and HID identifiers.
- Promote newly validated experimental product IDs into the built-in safe write list once hardware-verified.
- Add optional backend modes such as `linux-sysfs`, `linux-hidraw`, and `external-driver` only if real devices require them.
- Add richer per-device capabilities, software animation support for LampArray devices, and optional per-key RGB support.

## Roadmap

- Persistent custom profiles.
- Profile import and export.
- App settings persistence.
- System tray integration.
- Start-on-login behavior.
- Global shortcuts.
- Logs export and diagnostics bundles.
- Automated frontend and backend tests.
- Multi-vendor driver support.

## Contributing Notes

Keep domain models free of Tauri, filesystem, and hardware access. Keep command handlers thin. Add real hardware code only through driver and infrastructure layers, and prefer read-only detection before any write-capable implementation.
