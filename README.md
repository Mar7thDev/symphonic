# Symphonic

## Usage
- Install Git LFS, run `git lfs install`, then clone this repository and run `git lfs pull`.
- Compile it with `cargo build --release`.
- Replace `HTGameBase.dll` in the game folder
- That's all

The checked-in payload and generated offsets target the **1.3.10 Windows client**.
The DLL is produced at `target/release/HTGameBase.dll`; the client loads it from
`Client/WindowsNoEditor/HT/Binaries/Win64/HTGameBase.dll`.
Local login uses the matching [fadia-rs services](https://github.com/Mar7thDev/fadia-rs).
Runtime diagnostics are written to `symphonic.log` next to the client executable.

`symphonic/symphony.bin` is a required build input stored in Git LFS, including
the original upstream revision. A source download without LFS objects is not
sufficient to build a usable patch. To regenerate the payload and hook constants
for a client build, use [nte-dumper](https://github.com/Mar7thDev/nte-dumper)'s
`symphonic` command. The generator validates the native login signatures before
writing the profile.

Original upstream: https://git.xeondev.com/fadia-rs/symphonic.

## Platform Support
This revision has been verified on Windows. Upstream also supported Linux
through Wine; that configuration has not been verified with this client profile.
