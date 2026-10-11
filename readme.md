# `macmon` – Mac Monitor (personal fork)

A personal fork of [vladkens/macmon](https://github.com/vladkens/macmon), the sudoless performance monitor for Apple Silicon Macs by [vladkens](https://github.com/vladkens). All the hard parts (reading CPU / GPU / ANE power, temperatures and memory through private macOS APIs, the same data `powermetrics` exposes, without root) are theirs; go there for the released tool.

The fork makes macmon one screen for the three pillars of a computer: processing, I/O and network. CPU, GPU and RAM sit on top, with power under each processor. Below RAM, disk and network sit side by side, so the row reads from data moving within the machine to its traffic with the outside world. The process list breaks it all down per process. It grew out of running local LLMs next to normal work, where the question is whether memory is under pressure and what is using it:

- **Process list on its own interval** (`-I`, `[` / `]`), so it can refresh slower than the graphs.
- **Memory pressure**, not just “used”: RAM split into wired / app / compressed / cached as Activity Monitor splits it, the history colored by kernel memory pressure, swap activity on the title.
- **Network** download / upload in place of the mostly idle ANE power box, and per process in the process list.
- **Disk** read / write throughput.
- A reworked layout that gives the metric boxes half the screen and degrades gracefully in small windows.

```
╭─ Apple M5 Pro (12P+6S+20GPU 64GB) ─────────────────────────────────────────────────────────────────── macmon v0.9.0 ─╮
│╭─ CPU 2% ──────────────────╮╭─ GPU 2% @ 338 MHz ─────────╮╭─ RAM 25.87 GB (40.4%) · SWAP 2.78 / 4.0 GB ──── normal ─╮│
││                           ││                            ││██████████████████████████████████████                   ││
││                           ││                            ││                                                         ││
││                           ││                            ││                                                         ││
││                           ││                            ││                                             ▅▅▅▅▅▅▅▅▅▅▅▅││
││               ▁▁▂▁▁▁▁▂▂▁▁▁││                ▂▁▁▁▁▁▁▁▁▁▁▁││                                             ████████████││
│╰─ S 2% · P 2% ─────────────╯╰────────────────────────────╯╰─ wired 4.0 · app 20.2 · comp 1.7 ×4.3 · cache 17.3 GB ──╯│
│                                                                                                                      │
│╭─ CPU 0.66W ──────── 39°C ─╮╭─ GPU 0.05W ───────── 39°C ─╮╭─ DISK R 0 B/s W 0 B/s ────╮╭─ NET ↓ 341 B/s ↑ 310 B/s ──╮│
││               █ █    ▄    ││                █           ││                           ││                            ││
││               █ █ ▇  █▁ ▄▄││                █  ▃ ▂▁▂    ││                           ││                            ││
││               █ █▇██▃██▅██││                █▆▆█▄████▇▃▇││                    ▁      ││                 ▁▁▁▁▁▁▁▁▁▁▁││
││               █▅██████████││                ████████████││                       █   ││                            ││
││               ████████████││                ████████████││                 ▁▁▁▁▁▁█   ││                 ▁▁▁▁▁▁▁▁▁▁▁││
│╰───────────────────────────╯╰────────────────────────────╯╰─ R8.1K W9.6M ─────────────╯╰─ ↓5.7K ↑39.3K ─────────────╯│
│                                                                                                                      │
╰─ Power 0.71W (0.71, 1.04) | Fans 0/0 RPM | Total 13.33W (13.96, 16.30) ──────────────────────────────────────────────╯
```

## 🌟 Features

- 🚫 Runs without sudo
- ⚡ Real-time CPU / GPU / ANE power usage
- 📊 CPU frequency-scaled and active ratios per cluster
- 💾 RAM / Swap usage, with wired / app / compressed / cached RAM, memory pressure and swap rates
- 🌐 Network download / upload throughput
- 💽 Disk read / write throughput
- 📈 Historical charts with average and max values
- 🌡️ Average CPU / GPU temperature
- 📋 Process list with per-process CPU, memory, power, GPU usage and network download / upload
- 🎨 Follows your terminal's color scheme
- 🪟 Can be displayed in a small window
- 🦀 Written in Rust

## 📥 Installation

Build from source with a [Rust toolchain](https://rustup.rs/):

```sh
git clone https://github.com/OmerBaddour/macmon.git
cd macmon
cargo install --path .
```

This puts `macmon` in `~/.cargo/bin`. To try it without installing, run `cargo run --release`.

## 🚀 Usage

```sh
Usage: macmon [OPTIONS] [COMMAND]

Commands:
  pipe       Output metrics in JSON format (suitable for piping)
  serve      Serve metrics over HTTP (JSON at /json, Prometheus at /metrics)
  debug      Print debug information
  find-clpc  Find CLPC power counters with CPU/GPU/ANE loads
  stress     Generate load for testing metrics
  help       Print this message or the help of the given subcommand(s)

Options:
  -i, --interval <INTERVAL>            Update interval in milliseconds [default: 1000]
  -I, --proc-interval <PROC_INTERVAL>  Process list refresh interval in milliseconds, independent from `--interval` [default: 5000]
  -h, --help                           Print help
  -V, --version                        Print version
```

### Interactive mode

Run `macmon` without a subcommand to open the terminal UI: the metric boxes on top and the process list below. Press `?` in the app to see the keys.

### JSON output

You can use the `pipe` subcommand to output metrics in JSON format, which makes it suitable for piping into other tools or scripts. For example:

```sh
macmon pipe | jq
```

This command runs `macmon` in "pipe" mode and sends the output to `jq` for pretty-printing.

You can also specify the number of samples to collect using the `-s` or `--samples` parameter (default: `0`, which runs indefinitely), and set the update interval in milliseconds using the `-i` or `--interval` parameter (default: `1000` ms). For example:

```sh
macmon pipe -s 10 -i 500 | jq
```

This will collect 10 samples with an update interval of 500 milliseconds.

### HTTP server

You can use the `serve` subcommand to expose metrics over HTTP. This is useful for integrating with monitoring systems like [Prometheus](https://prometheus.io/) and [Grafana](https://grafana.com/).

```sh
macmon serve                   # default port 9090, interval 1000ms
macmon serve --host 127.0.0.1  # listen on localhost only
macmon serve -p 8080           # custom port
macmon serve -i 500            # sampling interval 500ms
macmon serve &                 # run in background
```

Two endpoints are available:

| Endpoint       | Format     | Description                                                                                       |
| -------------- | ---------- | ------------------------------------------------------------------------------------------------- |
| `GET /json`    | JSON       | Current metrics snapshot (same format as `pipe --soc-info`)                                       |
| `GET /metrics` | Prometheus | Metrics in [Prometheus text format](https://prometheus.io/docs/instrumenting/exposition_formats/) |

#### launchd service

To start `macmon serve` automatically on login and keep it running:

```sh
macmon serve --install                   # install and start (default port 9090)
macmon serve --port 8080 --install       # with custom port
macmon serve --host 127.0.0.1 --install  # listen on localhost only
macmon serve --uninstall                 # stop and remove
```

This creates a launchd agent at `~/Library/LaunchAgents/com.macmon.plist` that auto-starts on login and restarts on crash.

#### Prometheus and Grafana

The `/metrics` endpoint exposes metrics in Prometheus format. See [`examples/grafana`](examples/grafana) for a local demo stack with Prometheus and Grafana.

<details>
<summary>Prometheus output example</summary>

```
# HELP macmon_cpu_temp_celsius Average CPU temperature in Celsius
# TYPE macmon_cpu_temp_celsius gauge
macmon_cpu_temp_celsius{chip="Apple M3 Pro"} 47.3

# HELP macmon_cpu_power_watts CPU power consumption in Watts
# TYPE macmon_cpu_power_watts gauge
macmon_cpu_power_watts{chip="Apple M3 Pro"} 8.42

# HELP macmon_fan_speed_rpm Fan speed in revolutions per minute
# TYPE macmon_fan_speed_rpm gauge
macmon_fan_speed_rpm{chip="Apple M3 Pro",fan="fan0"} 1234

# HELP macmon_cpu_scaled_ratio Combined frequency-scaled CPU ratio (0–1), weighted by core count
# TYPE macmon_cpu_scaled_ratio gauge
macmon_cpu_scaled_ratio{chip="Apple M3 Pro"} 0.037

# HELP macmon_cpu_active_ratio Combined CPU active residency ratio (not frequency-scaled, 0–1), weighted by core count
# TYPE macmon_cpu_active_ratio gauge
macmon_cpu_active_ratio{chip="Apple M3 Pro"} 0.092

# HELP macmon_ecpu_scaled_ratio Efficiency CPU cluster frequency-scaled ratio (0–1)
# TYPE macmon_ecpu_scaled_ratio gauge
macmon_ecpu_scaled_ratio{chip="Apple M3 Pro"} 0.083

# HELP macmon_ecpu_freq_mhz Efficiency CPU cluster frequency in MHz
# TYPE macmon_ecpu_freq_mhz gauge
macmon_ecpu_freq_mhz{chip="Apple M3 Pro"} 1100

# HELP macmon_ecpu_active_ratio Efficiency CPU cluster active residency ratio (not frequency-scaled, 0–1)
# TYPE macmon_ecpu_active_ratio gauge
macmon_ecpu_active_ratio{chip="Apple M3 Pro"} 0.18
```

</details>

### Stress testing

Use `macmon stress` to generate load while checking metric behavior:

```sh
macmon stress
macmon stress pulse --duration 30
macmon stress cpu --duration 30
macmon stress cpu --workers 8 --duration 30
macmon stress gpu --duration 30
macmon stress all --duration 30
```

The default `pulse` mode generates a predictable cyclic CPU load with a fixed 50% duty cycle on half of the logical CPUs. The `cpu` and `gpu` modes continuously load only the selected processor, while `all` loads both. The `cpu` and `all` modes use all logical CPUs unless `--workers` is specified; `--workers` has no effect in `gpu` mode.

## 📊 Metrics

### Power sources and macOS 27

macOS 27 broke the CPU/ANE power source that macmon previously used without root. The IOReport `Energy Model` counters can remain visible but stop updating: refreshing them now requires the private `com.apple.private.pmgr.nrg.reporting` entitlement. Apple's signed `powermetrics` has this entitlement and can keep using the named channels. Running macmon with `sudo` does not grant the same access.

The working rootless workaround we found is to read hidden energy reports from the AppleCLPC driver through IOReport. These reports use opaque IDs instead of discoverable CPU/GPU/ANE channel names, so macmon has to identify the counters and verify their units. This leaves macmon maintaining undocumented mappings for data that Apple's own tool can still read directly.

Both the CLI and the Rust library use known CLPC counters automatically, and `Sampler::new()` retains legacy fallbacks for older systems. New chips or driver versions may need additional mappings. On an unsupported configuration, zero CPU/ANE readings can mean unavailable counters rather than zero consumption. See [docs/clpc-discovery.md](docs/clpc-discovery.md) for details.

### Output format

The `pipe` command and the HTTP `/json` endpoint return the same metrics:

<details>
<summary>JSON example</summary>

```jsonc
{
  "timestamp": "2025-02-24T20:38:15.427569+00:00",
  "temp": {
    "cpu_temp_avg": 43.73614, // Celsius, null when no sensor has a valid reading
    "gpu_temp_avg": 36.95167, // Celsius, null when no sensor has a valid reading
  },
  "memory": {
    "ram_total": 25769803776, // Bytes
    "ram_usage": 20985479168, // Bytes
    "swap_total": 4294967296, // Bytes
    "swap_usage": 2602434560, // Bytes
    "ram_wired": 4232577024, // Bytes, pinned: the kernel, GPU buffers
    "ram_app": 13711769600, // Bytes, anonymous memory of processes
    "ram_compressed": 2587754496, // Bytes of RAM the compressor occupies
    "ram_compressed_data": 10136238080, // Bytes stored in the compressor, uncompressed
    "ram_cached": 7301644288, // Bytes, cached files and purgeable memory
    "pressure": "Normal", // "Normal" / "Warning" / "Critical", null when unavailable
    "swap_in_rate": 0, // Bytes per second
    "swap_out_rate": 0, // Bytes per second
  },
  "network": {
    "rx_rate": 1843200, // Bytes received per second, without loopback, tunnels and bridges
    "tx_rate": 40960, // Bytes sent per second
  },
  "disk": {
    "read_rate": 5242880, // Bytes read per second, without disk images
    "write_rate": 1048576, // Bytes written per second
  },
  "fans": [
    { "name": "fan0", "rpm": 999, "max_rpm": 4900 },
    { "name": "fan1", "rpm": 1200, "max_rpm": 5200 },
  ],
  "cpu_scaled_ratio": 0.036854, // Combined frequency-scaled CPU ratio (weighted by core count, 0–1)
  "cpu_active_ratio": 0.092, // Combined active residency ratio (not frequency-scaled, weighted by core count, 0–1)
  "ecpu_freq_mhz": 1100, // Cluster frequency
  "ecpu_scaled_ratio": 0.082656614, // Frequency-scaled ratio (0–1)
  "ecpu_active_ratio": 0.18, // Active residency (not frequency-scaled, 0–1)
  "pcpu_freq_mhz": 1800, // Cluster frequency
  "pcpu_scaled_ratio": 0.015181795, // Frequency-scaled ratio (0–1)
  "pcpu_active_ratio": 0.04, // Active residency (not frequency-scaled, 0–1)
  "ecpu_cores": [
    { "die_id": 0, "core_id": 0, "freq_mhz": 1600, "scaled_ratio": 0.14, "active_ratio": 0.24 },
    { "die_id": 0, "core_id": 1, "freq_mhz": 1700, "scaled_ratio": 0.12, "active_ratio": 0.2 },
  ],
  "pcpu_cores": [
    { "die_id": 0, "core_id": 0, "freq_mhz": 2100, "scaled_ratio": 0.05, "active_ratio": 0.08 },
    { "die_id": 0, "core_id": 1, "freq_mhz": 2200, "scaled_ratio": 0.07, "active_ratio": 0.06 },
  ],
  "gpu_freq_mhz": 461, // GPU frequency
  "gpu_scaled_ratio": 0.021497859, // Frequency-scaled ratio (0–1)
  "gpu_active_ratio": 0.09, // GPU active residency ratio (not frequency-scaled, 0–1)
  "cpu_power": 0.20486385, // Watts
  "gpu_power": 0.017451683, // Watts
  "ane_power": 0.0, // Watts
  "all_power": 0.22231553, // Watts
  "sys_power": 5.876533, // Watts
  "ram_power": 0.11635789, // Watts
  "gpu_ram_power": 0.0009615385, // GPU SRAM power, Watts
}
```

</details>

### Active and scaled ratios

`active_ratio` is the share of the sampling interval during which the processor was doing any work. `scaled_ratio` is the same measure adjusted for operating frequency, showing the share of the processor's maximum possible capacity that was used.

For interval $T$, $t_i$ is the time spent doing work at frequency $f_i$, and $f_\text{max}$ is the hardware maximum frequency for the CPU cluster or GPU. The product $T f_\text{max}$ is the theoretical maximum frequency-time the processor could deliver at full load without thermal or power throttling:

```math
R_\text{active} = \frac{\sum_i t_i}{T}
\qquad
R_\text{scaled} = \frac{\sum_i t_i f_i}{T f_\text{max}}
```

CPU ratios are calculated per core and averaged across the cluster. A core that did no work contributes zero; combined CPU ratios average all cores from both clusters:

```math
R_\text{cluster} = \frac{1}{N_\text{cores}}\sum_j R_j
```

Examples:

- Full interval at half the maximum frequency → `active = 1.0`, `scaled = 0.5`.
- Half the interval at the maximum frequency → `active = 0.5`, `scaled = 0.5`.
- One of ten cores working at maximum frequency → cluster `active = scaled = 0.1`.

### Disk throughput

Disk rates come from the `Statistics` of each `IOBlockStorageDriver` in the IORegistry, as `iostat` reads them: physical bytes moved to and from every disk, external ones included, so reads served from the file cache don't count. Disk images are left out, because their reads and writes reach the disk holding the image file too.

There is no per-process disk I/O: it needs root. The counters exist (`ri_diskio_bytesread` / `ri_diskio_byteswritten` from `proc_pid_rusage`), but macOS returns `EPERM` for processes of other users, including root daemons such as `kernel_task`, `mds_stores` and `backupd`, which do much of the disk I/O. Unlike CPU time (the setuid `/bin/ps`) and network traffic (`nettop`), no rootless tool reports it for them either: `ps` and `nettop` don't have it, and `fs_usage` and `iotop` need root. Columns for your own processes only would leave the heaviest disk users blank and wouldn't add up to the disk box.

## 📚 Library usage

`macmon` can be used as a Rust library to collect Apple Silicon metrics in your own applications.

Add it to your project:

```sh
cargo add macmon --no-default-features
```

The default `app` feature enables the `macmon` executable and its terminal UI dependencies. Disable default features when using `macmon` only as a library.

Run the standalone [demo app](examples/demo-app):

```sh
cargo run --manifest-path examples/demo-app/Cargo.toml
```

Then use the `Sampler` to collect metrics:

```rust
use macmon::Sampler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sampler = Sampler::new()?;

    // collect metrics over a 1000ms window
    let metrics = sampler.get_metrics(1000)?;

    println!("CPU power:  {:.2} W", metrics.cpu_power);
    println!("GPU power:  {:.2} W", metrics.gpu_power);
    match metrics.temp.cpu_temp_avg {
        Some(value) => println!("CPU temp:   {value:.1} °C"),
        None => println!("CPU temp:   N/A"),
    }
    println!("RAM usage:  {} / {} bytes", metrics.memory.ram_usage, metrics.memory.ram_total);
    println!("eCPU:       {} MHz  {:.1}%", metrics.ecpu_freq_mhz, metrics.ecpu_scaled_ratio * 100.0);
    println!("pCPU:       {} MHz  {:.1}%", metrics.pcpu_freq_mhz, metrics.pcpu_scaled_ratio * 100.0);

    Ok(())
}
```

`get_metrics(duration_ms)` blocks the calling thread while collecting one IOReport delta over the complete interval. For a UI, server, or async application, create the sampler inside a dedicated worker thread and send the completed metrics back through a channel:

```rust
use std::{sync::mpsc, thread};

use macmon::Sampler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut sampler = Sampler::new().expect("failed to create sampler");

        while let Ok(metrics) = sampler.get_metrics(1000) {
            if tx.send(metrics).is_err() {
                break;
            }
        }
    });

    // Use recv() in a consumer thread or try_recv() in a non-blocking event loop.
    let metrics = rx.recv()?;
    println!("CPU power: {:.2} W", metrics.cpu_power);

    Ok(())
}
```

Creating `Sampler` inside the worker keeps its low-level macOS handles on that thread. In an async runtime, use its blocking-thread facility rather than calling `get_metrics` directly from an executor worker.

## 🤝 Contributing

This fork is personal. Issues and improvements for macmon itself belong [upstream](https://github.com/vladkens/macmon).

## 📝 License

Distributed under the [MIT License](LICENSE).

## 🔍 See also

- [tlkh/asitop](https://github.com/tlkh/asitop) – The original tool. Written in Python, requires sudo.
- [dehydratedpotato/socpowerbud](https://github.com/dehydratedpotato/socpowerbud) – Written in Objective-C, sudoless, no TUI.
- [op06072/NeoAsitop](https://github.com/op06072/NeoAsitop) – Written in Swift, sudoless.
- [graelo/pumas](https://github.com/graelo/pumas) – Written in Rust, requires sudo.
- [context-labs/mactop](https://github.com/context-labs/mactop) – Written in Go, requires sudo.
