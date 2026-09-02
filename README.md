# Async BLDC motor driver

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/target-rp2350-4d8af7" alt="RP2350 target" />
  <img src="https://img.shields.io/badge/embedded-async%20drivers-1abc9c" alt="Embedded async" />
</p>

A compact, async, no_std BLDC control stack for embedded systems. The project is built around a generic driver core, relying on a hardware abstraction layer, that makes the porting to any hardware supported by Embassy, which is the driver core technology, straightforward.

## Table of contents

- [Async BLDC motor driver](#async-bldc-motor-driver)
  - [Table of contents](#table-of-contents)
  - [Project scope](#project-scope)
  - [Driver features](#driver-features)
  - [Architecture overview](#architecture-overview)
    - [Tasks](#tasks)
    - [FOC Algorithm](#foc-algorithm)
  - [Workspace structure](#workspace-structure)
  - [Test hardware](#test-hardware)
  - [The simplest and cheapest driver possible, docs here](#the-simplest-and-cheapest-driver-possible-docs-here)
  - [HAL structure](#hal-structure)
    - [Core traits](#core-traits)
    - [Angle type](#angle-type)
  - [Tests](#tests)
    - [Local PLL tuning in software](#local-pll-tuning-in-software)
    - [Controller tuning on board](#controller-tuning-on-board)
    - [Verified build status](#verified-build-status)
  - [How to use it](#how-to-use-it)
    - [For an already implemented setup](#for-an-already-implemented-setup)
    - [Implement a new board / motor / encoder](#implement-a-new-board--motor--encoder)
  - [Todo](#todo)
    - [Board-specific work](#board-specific-work)
    - [Motor-specific work](#motor-specific-work)
    - [Encoder-specific work](#encoder-specific-work)
    - [Control algorithm work](#control-algorithm-work)
  - [References and docs](#references-and-docs)
    - [External references used in the design](#external-references-used-in-the-design)

---

## Project scope

We aimed at making it possible to control a BLDC motor with simple and cheap hardware: no current sensing is required, only the encoder is needed. The goal is to keep the control logic simple and maintainable by splitting it into cooperating async tasks, which is why we chose to work with Embassy. 

The project was inspired by [SimpleFOC](https://simplefoc.com/), the Arduino library for FOC, and its documentation. We tried to adapt the best ideas from [its control algorithms](https://docs.simplefoc.com/voltage_torque_control) to our technology and architecture, while keeping the implementation lightweight, fixed-point friendly, and suitable for embedded firmware.

In practice, this repo is built around:

- a generic hardware abstraction so that the core logic stays mostly board-agnostic,
- an encoder as the main feedback source,
- a PLL-based observer to estimate angle and velocity,
- a torque-oriented control loop that generates 3-phase PWM outputs,
- telemetry collection and tuning validation on both local simulation and real hardware.

The current implementation is centered on the Raspberry Pi RP2350 and the AS5048A magnetic encoder, but the architecture is intentionally designed to make porting to other boards and motors straightforward.

## Driver features

- Async task orchestration via Embassy `Watch` channels and `Signal`s.
- Torque-controlled estimated-current control mode.
- FOC driven voltage generation, with a basic inverse Park / Clarke conversion in the control loop.
- Generic `Encoder<BITS>` trait for angle sensor implementations.
- Generic `BldcMotor<BITS>` trait for 3-phase PWM motor control.
- Avoid float calculations: `IntAngle<BITS>` fixed-point angle type with integer-safe trigonometric helpers.
- Implementation of a Type-II PLL: `PllObserver` for angle and velocity estimation from encoder samples.
- On-board telemetry capture through flash and CSV export tooling.

## Architecture overview

### Tasks

![FOC controller tasks overview](/docs/tasks.png "FOC controller tasks overview")

The main task initializes the hardware, starts the encoder, PLL, controller, and optional telemetry tasks, and sends the requested torque through a watch channel. The encoder task samples the sensor at high frequency; the PLL task converts those samples into estimated angle and velocity; and the controller task uses those estimates to update the three-phase PWM output. If present, the telemetry task observes the watches at a lower rate and writes a capture to flash when the run is complete.

### FOC Algorithm

![](https://docs.simplefoc.com/extras/Images/torque_control/ec0_b.png)

Most of our FOC algorithms documentation comes from SimpleFOC docs, as it is explained in detail very clearly, like in this diagram for the [estimated-current FOC](https://docs.simplefoc.com/voltage_torque_control#level-3-inductance-lag-compensation-r--kv--l) signal path that we implemented.

The requested q-axis current is limited and converted into a voltage using the phase resistance, estimated back-EMF, and q-axis feed-forward terms. The d-axis current is normally zero, while its voltage path includes the resistance, voltage limit, feed-forward, and estimated lag-voltage compensation from motor inductance per speed. Eventually, inverse Park and Clarke transforms convert the d/q voltages and electrical angle into phase voltages and then PWMs (with SVPWM calculations), which the BLDC driver applies to the motor.

---

## Workspace structure

```text
bldc-driver-async/
├── bldc-driver-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── controller.rs
│       ├── encoder.rs
│       ├── pll.rs
│       └── telemetry.rs
├── bldc-driver-hal/
│   ├── Cargo.toml
│   └── src/
│       ├── angle.rs
│       └── lib.rs
├── bldc-driver-rp2350/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── memory.x
│   └── src/
│       ├── main.rs
│       ├── bldc_motor.rs
│       ├── encoder.rs
│       └── flash.rs
├── test-core/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── csv.rs
│       ├── motor.rs
│       └── angle_iter/
├── Cargo.toml
├── README.md
├── run.sh
├── get-telemetry.sh
├── telemetry-plot.sh
├── tools/
│   └── parse_telemetry.py
└── telemetry_*.csv
```

What each crate is doing:

- `bldc-driver-hal`: hardware abstraction, angle math, generic motor and encoder traits.
- `bldc-driver-core`: async tasks, controller loop, PLL observer, telemetry task, and the control pipeline.
- `bldc-driver-rp2350`: concrete RP2350 board support, SPI encoder wiring, PWM motor driver, and flash-backed telemetry.
- `test-core`: local simulation and tuning harness for controller and PLL behavior without needing a board attached.

---

## Test hardware

- ### RP2350

  - MCU: [Raspberry Pi RP2350](https://www.raspberrypi.com/products/rp2350/)
  - Architecture: dual-core Arm Cortex-M33 / Cortex-M33 plus a secure core model
  - Target: embedded high-speed PWM + DMA + SPI control
  - Used in this repo as the main controller and logger target

- ### AS5048A magnetic encoder

  - Sensor: AS5048A, 14-bit absolute magnetic rotary encoder
  - Interface: SPI
  - Resolution: 14-bit angle, effectively 16384 positions per turn
  - Use in this project: angle measurement for the PLL and control loop

  Reference: [AS5048A datasheet](https://www.mouser.com/datasheet/2/588/AS5048_DS000298_4-00-1100510.pdf?srsltid=AfmBOopVQXd4zM0YdcdOcmUoEoGPYkSAAHg5qVXJu2K3LdYZ0NbKSO3k)

- ### GBM2804H-100T gimbal motor

  This repo targets the [GBM2804H-100T motor](https://it.aliexpress.com/item/4001137970972.html?gatewayAdapt=glo2ita#nav-specification), a compact gimbal motor typically used in lightweight torque-control applications.

  | Parameter                  |         Value |
  | -------------------------- | ------------: |
  | Stator resistance $R$      |         5.6 Ω |
  | Stator inductance $L$      |       0.92 mH |
  | Back-EMF coefficient $k_e$ | 0.047 V·s/rad |
  | Friction coefficient $B$   |    550 nN·m·s |
  | Moment of inertia $J$      |   480 nN·m·s² |
  | Number of poles $P$        |             7 |
  | Rated voltage $V_{dc}$     |          12 V |
  | Torque coefficient $k_t$   |    0.07 N·m/A |

- ### SimpleFOCMini v1.1 driver board

  The simplest and cheapest driver possible, [docs here](https://docs.simplefoc.com/simplefocmini)
---

## HAL structure

The hardware abstraction layer is intentionally generic, to take into consideration the encoder precision (in `BITS`) using fixed-point arithmetic.

### Core traits

- `Encoder<const BITS: usize>`
  - defines how to read an angle sample,
  - exposes sample rate and period,
  - used by the encoder task.

- `BldcMotor<const BITS: usize>`
  - defines winding resistance, inductance, pole pairs, torque constant, PWM top, and limits,
  - exposes the hardware PWM API through `set_pwm` and `wake_to_set_pwm`,
  - used by the controller task.

- `TelemetryFlash<Data, const BUFFER_LEN: usize>`
  - allows to write captured samples to flash memory.

### Angle type

The project uses `IntAngle<BITS>` in `bldc-driver-hal/src/angle.rs`.

It provides:

- fixed-point angle math,
- normalized rotation wrapping,
- trigonometric functions via lookup tables,

This avoids floating-point math in the critical control path of the control loop.

---

## Tests

The project includes both local simulation and real board telemetry.

### Local PLL tuning in software

![PLL tuning at constant speed](/docs/pll-tuning-constant-speed.png "PLL tuning at constant speed")
![PLL tuning with speed ramp](/docs/pll-tuning-ramp-speed.png "PLL tuning with speed ramp")

> The plots represent the simulated angle on the left, and the PLL speed estimation on the right

The `test-core` harness simulates an encoder stream and exercises the PLL observer directly. This is where we adjusted the filter tuning before moving to the real board hardware.

From the local test output, the estimated angle stays very close to the simulated angle while velocity ramps up. This tuning is all done in the integer domain with stable fixed-point gains!

Eventually we used this local test environment also to test the correct PWM output of the controller algorithm, mainly to avoid mathematical errors in the Park / Clarke and SVPWM transformations.

![Controller algorithm PWM generation](/docs/controller-algorithm-debug.png "Controller algorithm PWM generation")

### Controller tuning on board

The on-board target program we used for development testing is in `bldc-driver-rp2350/src/main.rs`. It generates with the macro `generate_bldc_driver_tasks` all the methods and tasks of the driver core. Then it calls the run methods, that launches:

- the SPI encoder task,
- the PLL observer task,
- the controller task,
- and a telemetry capture task (optional, with an independent run method call).

The board is currently configured to run the driver with these frequencies:
- 100 kHz for encoder reads 
- 25 kHz for PWM and controller task
- 100 Hz for telemetry snapshots (written to flash only at the end)

> The telemetry capture is written to a .bin file and then converted by the python script to a .csv

Notable observations from the captured samples:

- startup telemetry shows measured angle and estimated angle remaining tightly aligned at low speed,
- the estimator quickly converges and follows the measured encoder signal,
- the board successfully reaches high PWM duty and high estimated velocities under a target torque condition,
- the telemetry pipeline is working end-to-end, from sensor read → PLL estimation → controller output → flash serialization → CSV export.

![PWM plot at low positive torque](/docs/real-pwm.png "PWM plot at low positive torque")

![Complete run at high positive and negative torque](/docs/bemf.png "Complete run at high positive and negative torque")

https://github.com/user-attachments/assets/2d24976e-bf17-4fcb-9dd1-84e7d065c5d6

![PWM plot at low positive torque](/docs/video-telemetry.png "PWM plot at low positive torque")

Examples from the data:

```text
telemetry_parsed.csv
  angle: 166.79°, est: 166.64°, velocity_est: 54, pwm_b: 2999, pwm_c: 2845
  angle: 171.74°, est: 166.95°, velocity_est: 2602, pwm_b: 2999, pwm_c: 2945
  angle: 176.46°, est: 167.56°, velocity_est: 7273, pwm_b: 2897, pwm_c: 2999
```

```text
telemetry_parsed-max-speed.csv
  angle: 94.46°, est: 77.15°, velocity_est: 29425, pwm_a: 1474, pwm_b: 0, pwm_c: 2999
  angle: 130.41°, est: 118.65°, velocity_est: 223010, pwm_a: 0, pwm_b: 2232, pwm_c: 2999
```

This confirms the system is alive, the PLL is running, and the controller is generating nontrivial phase duty cycles under real motor conditions.

### Verified build status

The workspace was checked with:

```bash
cargo check --workspace
```

and the command completed without errors, confirming the current workspace state builds successfully.

---

## How to use it

### For an already implemented setup

If you already have the RP2350 board, AS5048A encoder, and GBM2804H-100T motor wired and configured, the flow is simple:

1. Build the firmware:

```bash
cargo build --bin bldc-driver-rp2350
```

2. Flash it to the board.

3. Run the telemetry capture script:

```bash
./get-telemetry.sh
```

This saves a binary capture to the repo root and converts it to CSV using `tools/parse_telemetry.py`.

4. Inspect the resulting CSV to validate angle estimation and PWM behavior.

The board entry point is `bldc-driver-rp2350/src/main.rs`. The current example sets a torque target and starts telemetry automatically:

```rust
gimbal_motor::set_torque(100_000);
let telemetry_end =
    gimbal_motor::run_telemetry(spawner, flash, TELEMETRY_FREQUENCY, TELEMETRY_DURATION_US);
```

The firmware is set up to give a working experimental reference, not just a skeleton.

### Implement a new board / motor / encoder

To adapt this project to another MCU or sensor:

1. Implement `Encoder<BITS>` for your sensor.
2. Implement `BldcMotor<BITS>` for your board and motor.
3. Add a flash or telemetry backend implementing `TelemetryFlash<Data, BUFFER_LEN>` if you want on-device traces.
4. Use the macro `generate_bldc_driver_tasks!` to wire the async loop together.
5. Tune the PLL using your actual encoder sample period and expected max speed.
6. Validate the controller using local simulation before on-board tests.

The generic pattern is already visible in the RP2350 implementation:

- `bldc-driver-rp2350/src/encoder.rs`
- `bldc-driver-rp2350/src/bldc_motor.rs`
- `bldc-driver-rp2350/src/flash.rs`
- `bldc-driver-core/src/lib.rs`

This makes the repo reasonable as a starting point for additional boards and motor families.

---

## Todo

### Board-specific work

- split the board support into more explicit platform crates,
- support more MCU targets than RP2350,
- improve startup and fault handling for resets and watchdog conditions,
- make the telemetry logger more robust for long captures.

### Motor-specific work

- separate per-motor parameter packs from the generic controller loop,
- handle more motor geometries and non-gimbal motors,
- improve current/torque model tuning per motor family.

### Encoder-specific work

- support more encoder types: ABI, incremental, Hall, magnetic, resolver-like,
- add calibration and zero-offset routines,
- improve error checking for bad SPI reads and invalid angle packets.

### Control algorithm work

- implement a fully integer-based Park / Clarke and inverse Park / Clarke path,
- reduce float usage in the control loop,
- add explicit current loop and PI tuning for q/d axis control,
- add feed-forward and decoupling terms for higher-speed operation.

---

## References and docs

### External references used in the design

- [Embassy RP2350 examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp235x)
- [RP2350 build script reference](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/build.rs)
- [RP2350 memory.x reference](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/memory.x)
- [AS5048A datasheet](https://www.mouser.com/datasheet/2/588/AS5048_DS000298_4-00-1100510.pdf?srsltid=AfmBOopVQXd4zM0YdcdOcmUoEoGPYkSAAHg5qVXJu2K3LdYZ0NbKSO3k)
- [Estimated current mode theory](https://docs.simplefoc.com/voltage_torque_control#estimated-current-mode-theory)
- [How to Build a Fixed-Point PI Controller](https://www.embeddedrelated.com/showarticle/123.php)
- [Alpha-beta filter](https://en.wikipedia.org/wiki/Alpha_beta_filter)
- [Type-II PLL article](https://www.allaboutcircuits.com/technical-articles/introduction-to-second-order-type-2-plls/)
- [Space Vector Modulation](https://it.mathworks.com/discovery/space-vector-modulation.html)
- [Park transform](https://it.mathworks.com/help/sps/ref/parktransform.html)
- [Clarke transform](https://it.mathworks.com/help/sps/ref/clarketransform.html)
- [Paper that uses the GBM2804H-100T motor](https://arxiv.org/pdf/2505.01740#:~:text=0.92%20(mH).%20Number%20of%20Poles.%20(P).%207.,0.07%20(N.m%2FA).%20NSGA-II%20has%20a%20computational%20%7B%22isScientific%22%3Atrue%2C%22citationCount%22%3A0%2C%22authors%22%3A%5B%5D%2C%22doi%22%3A%22%22%2C%22issuedYear%22%3A0%2C%22publisher%22%3A%22%22%2C%22containerTitle%22%3A%22%22%2C%22title%22%3A%22%22%2C%22page%22%3A%22%22%2C%22volume%22%3A%22%22%2C%22abstract%22%3A%22%22%7D)

![Our test video](/docs/video/bldc-motor-video.mp4 "Our test video")
