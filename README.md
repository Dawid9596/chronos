# Conservation Editor

Manage documents in standard format including layer characteristics, thickness,
chronological phases, and dating periods.

## Download

Installers for each platform are attached to every
[release](https://github.com/Dawid9596/chronos/releases) (the release page
renders the same instructions with real file names):

| Platform | File | How to install |
|----------|------|----------------|
| Windows | `chronos-windows_x.y.z_x64_en-US.msi` | Run the installer |
| Ubuntu / Debian | `chronos-linux_x.y.z_amd64.deb` | `sudo apt install ./chronos-linux_x.y.z_amd64.deb` |
| Other Linux | `chronos-linux_x.y.z_x86_64.AppImage` | `chmod +x chronos-linux_x.y.z_x86_64.AppImage` and run it |
| Portable Linux | `chronos-linux-x64.tar.gz` | Unpack and run the contained binary |

Notes:

- The .deb targets Ubuntu 24.04 / Debian 13 or newer (it links the GTK 4.14
  runtime).
- AppImages need FUSE (`libfuse2`); on systems without it, run the AppImage
  with `--appimage-extract-and-run` instead.
- The portable archive contains only the binary and expects the GTK 4 runtime
  to be installed through your distribution.
