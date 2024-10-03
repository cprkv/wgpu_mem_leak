# wgpu memory leak

This is minimal (could be smaller I guess) reproducing example for memory leaking bug in wgpu ([issue](https://github.com/gfx-rs/wgpu/issues/6143)).

## Environment

|  |  |
| --- | --- |
| GPU | NVIDIA GeForce RTX 3090 Ti |
| OS | Windows 10 Pro 22H2 (19045.4894) |
| Driver | Nvidia 560.94 |
| Rust | 1.81.0 (2dbb1af80 2024-08-20) |
| WGPU | 22.1.0 |
| Backend | Vulkan, DX12 |

## How to run

```powershell
cargo build
./target/debug/wgpu_mem_leak.exe
```

## How bad is leakage?

Table for (Windows 10 / Vulkan) from Task Manager:

60 mb/hour ~ 1 mb/minute ~ 17 kb/sec ~ **290 bytes/frame**
