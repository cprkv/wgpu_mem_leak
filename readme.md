# wgpu memory leak

This is minimal (could be smaller I guess) reproducing example for memory leaking bug in wgpu.

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

| Memory (MB) | Datetime |
| --- | --- |
| 216 | 02.12.2024 22:14:00 |
| 227 | 02.10.2024 22:24:00 |
| 676 | 03.10.2024 06:27:00 |

To summarize: for 493 minutes it leaks 460 MB, which is 955 KB per second, or **16 KB per frame**.
