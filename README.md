# lichen

A heavily in-development, experimental, and early days installer for AerynOS.

## Build & Test

### Running the backend

```bash
$ cargo build -p backend
$ sudo ./target/debug/lichen_backend
0.004011159s  INFO lichen_backend: 🚀 Serving on /run/lichen.sock
```

### Running the frontend

```bash
$ cargo run -p cli
```

To quit the installer, press `ESC` to switch to command mode, then press `q`.

## License

`lichen` is available under the terms of the [MPL-2.0](https://spdx.org/licenses/MPL-2.0.html)
