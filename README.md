# PhoshFileSelector

A widget for selecting files.

## Building

```sh
meson setup _build
meson compile -C _build
```

## Running

Running the Rust demo

```sh
G_MESSAGES_DEBUG=pfs _build/src/pfs-demo
```

Running the C demo

```sh
G_MESSAGES_DEBUG=pfs LD_LIBRARY_PATH=_build/src/ _build/src/examples/pfs-c-demo
```
