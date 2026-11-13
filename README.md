# lh-emc-fcc-helper (`lh-emc-fcc`)

[![CI](https://github.com/Logihertz-Systems-OPC-Pvt-Ltd/lh-emc-fcc-helper/actions/workflows/ci.yml/badge.svg)](https://github.com/Logihertz-Systems-OPC-Pvt-Ltd/lh-emc-fcc-helper/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](Cargo.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![Dependencies](https://img.shields.io/badge/dependencies-zero-brightgreen.svg)](Cargo.toml)
[![Safety](https://img.shields.io/badge/unsafe_code-forbidden-red.svg)](src/lib.rs)

> **Production-grade, zero-dependency Rust Hardware EMC Pre-Compliance Calculator, Near-Field H-Probe to Far-Field Radiated Emission Extrapolator, FCC Part 15B / CISPR 32 Limit Verifier, and 10-Rule EMI & TEMPEST Security Posture Auditor.**

---

## Table of Contents

- [Overview & Problem Statement](#overview--problem-statement)
- [Architectural Principles & Zero-Dependency Mandate](#architectural-principles--zero-dependency-mandate)
- [Analytical Physics & Mathematical Foundation](#analytical-physics--mathematical-foundation)
  - [1. Spectrum Analyzer to Near-Field Magnetic Flux Conversion](#1-spectrum-analyzer-to-near-field-magnetic-flux-conversion)
  - [2. Ampere's Loop Law Equivalent Conductor Current](#2-amperes-loop-law-equivalent-conductor-current)
  - [3. Differential-Mode (DM) Small-Loop Far-Field Extrapolation](#3-differential-mode-dm-small-loop-far-field-extrapolation)
  - [4. Common-Mode (CM) Cable Dipole Far-Field Extrapolation](#4-common-mode-cm-cable-dipole-far-field-extrapolation)
  - [5. Enclosure Aperture Seam & Slot Shielding Effectiveness](#5-enclosure-aperture-seam--slot-shielding-effectiveness)
  - [6. Trapezoidal Clock Fourier Harmonics & Spread-Spectrum Clocking](#6-trapezoidal-clock-fourier-harmonics--spread-spectrum-clocking)
- [Regulatory Limits & Safety Margin Thresholds](#regulatory-limits--safety-margin-thresholds)
- [10-Rule Hardware EMC, EMI & TEMPEST Security Posture Auditor](#10-rule-hardware-emc-emi--tempest-security-posture-auditor)
- [CLI Reference & Subcommands](#cli-reference--subcommands)
  - [`scan`](#lh-emc-fcc-scan)
  - [`extrapolate-scan`](#lh-emc-fcc-extrapolate-scan)
  - [`clock-harmonics`](#lh-emc-fcc-clock-harmonics)
  - [`check-limit`](#lh-emc-fcc-check-limit)
- [Verification & Test Suite](#verification--test-suite)
- [License & Governance](#license--governance)

---

## Overview & Problem Statement

In modern embedded hardware, high-speed printed circuit boards (PCBs), and sovereign defense edge computers, high-frequency digital clock transitions generate rapid current transients ($di/dt \ge 10^8\text{ A/s}$). These transients radiate electromagnetic interference (EMI) via two primary physical vectors:
1. **Differential-Mode (DM) Currents**: Signal return currents traversing trace loops on the PCB.
2. **Common-Mode (CM) Currents**: Microampere-level RF noise coupled onto external I/O cables and chassis ground loops, forming efficient monopole or dipole antennas.

Unintended electromagnetic radiation causes failure during statutory certification (FCC Part 15 Subpart B, CISPR 32 / EN 55032) and exposes cryptographic hardware (TPM, HSM, secure microcontrollers) to compromising electromagnetic emanations (**TEMPEST / EMSEC side-channel attacks**, CWE-1300).

`lh-emc-fcc-helper` (`lh-emc-fcc`) bridges the gap between laboratory near-field magnetic probe measurements and formal 3m/10m Open Area Test Site (OATS) or Semi-Anechoic Chamber (SAC) pre-compliance verification. It provides analytical physics extrapolation, regulatory limit comparison, and an automated 10-rule hardware layout posture auditor.

---

## Architectural Principles & Zero-Dependency Mandate

`lh-emc-fcc-helper` is built from the ground up under uncompromising engineering constraints:

- **100% Zero External Dependencies**: The `[dependencies]` table in `Cargo.toml` is completely empty. All analytical physics, piecewise limit masks, RFC 8259 JSON parsing, ANSI terminal table rendering, and OASIS SARIF v2.1.0 generation are implemented purely within the Rust standard library (`std`).
- **Memory Safety & Forbid Unsafe Code**: `#![forbid(unsafe_code)]` is enforced across all crates, modules, and tests.
- **Deterministic Math**: Double-precision floating-point arithmetic (`f64`) preserves physical constants ($c = 299,792,458.0\text{ m/s}$, $\eta_0 = 376.730313668\ \Omega$) with high numerical accuracy.

---

## Analytical Physics & Mathematical Foundation

```
+-----------------------------------------------------------------------------+
|                      ELECTROMAGNETIC RADIATION PIPELINE                     |
|                                                                             |
|  [Spectrum Analyzer (dBm)]                                                  |
|              |                                                              |
|              v                                                              |
|  [V_dBuV = P_SA + 107.0 + CableLoss - PreampGain]                           |
|              |                                                              |
|              v                                                              |
|  [H_dBuA/m = V_dBuV + AF_H]  -->  [H_linear = 10^((H - 120)/20) A/m]        |
|              |                                                              |
|              v                                                              |
|  [I_eq = 2 * pi * r_probe * H_near]  (Ampere's Circuital Loop Law)          |
|              |                                                              |
|      +-------+-------+                                                      |
|      |               |                                                      |
|      v               v                                                      |
|  Differential-Mode   Common-Mode                                            |
|  Small Loop (DM):    Cable Dipole (CM):                                     |
|  E_DM ~ f^2 * A_loop E_CM ~ f * L_cable                                     |
|      |               |                                                      |
|      +-------+-------+                                                      |
|              v                                                              |
|  [E_unshielded = sqrt(E_DM^2 + E_CM^2)] (+6 dB Ground Reflection)           |
|              |                                                              |
|              v                                                              |
|  [SE_slot = 20 * log10(lambda / 2*l_slot)] (Enclosure Aperture Cutoff)      |
|              |                                                              |
|              v                                                              |
|  [E_shielded = E_unshielded - min(SE_enclosure, SE_slot)]                   |
|              |                                                              |
|              v                                                              |
|  [Regulatory Limit Check: FCC Part 15B / CISPR 32 (3m / 10m)]               |
|  Margin = Limit - E_shielded (Threshold >= 6.0 dB)                          |
+-----------------------------------------------------------------------------+
```

### 1. Spectrum Analyzer to Near-Field Magnetic Flux Conversion

A 50-Ohm spectrum analyzer measures RF power $P_{\text{SA}}$ in $\text{dBm}$. Across a matched 50-Ohm load, $0\text{ dBm}$ corresponds to $107.0\text{ dB}\mu\text{V}$:

$$V_{\text{dB}\mu\text{V}} = P_{\text{SA}} + 107.0 + L_{\text{cable}} - G_{\text{preamp}}$$

The magnetic field strength $H$ is computed using the calibrated magnetic antenna factor $AF_H(f)$ (in $\text{dB}(\Omega^{-1}/\text{m})$):

$$H_{\text{dB}\mu\text{A/m}} = V_{\text{dB}\mu\text{V}} + AF_H(f)$$

$$H_{\text{near}} = 10^{\frac{H_{\text{dB}\mu\text{A/m}} - 120.0}{20.0}}\quad (\text{A/m})$$

### 2. Ampere's Loop Law Equivalent Conductor Current

From Ampère's circuital law $\oint \vec{H} \cdot d\vec{l} = I_{\text{enc}}$, the equivalent high-frequency RF current enclosed by a loop probe of radius $r_{\text{probe}}$ is:

$$I_{\text{eq}} = 2\pi r_{\text{probe}} H_{\text{near}}\quad (\text{A})$$

### 3. Differential-Mode (DM) Small-Loop Far-Field Extrapolation

For a printed circuit trace loop of area $A_{\text{loop}}\ (\text{m}^2)$ carrying differential-mode current $I_{\text{DM}}$, the radiated electric field at distance $d$ over a semi-anechoic ground plane ($G_{\text{gnd}} = 2.0 = +6\text{ dB}$ reflection) is:

$$E_{\text{DM}}(f, d) = G_{\text{gnd}} \cdot \frac{\eta_0 \pi A_{\text{loop}} I_{\text{DM}} f^2}{c^2 d}\quad (\text{V/m})$$

Where:
- $\eta_0 = 376.730313668\ \Omega$ (intrinsic impedance of free space)
- $c = 299,792,458.0\text{ m/s}$ (speed of light in vacuum)

### 4. Common-Mode (CM) Cable Dipole Far-Field Extrapolation

Common-mode currents flowing out on attached cables or chassis act as short-wire or quarter-wave antennas. For an attached cable of length $L_{\text{cm}}$ carrying common-mode current $I_{\text{CM}}$:

$$E_{\text{CM}}(f, d) = G_{\text{gnd}} \cdot \frac{\eta_0 L_{\text{cm}} I_{\text{CM}} f}{2 c d}\quad (\text{V/m})$$

The unshielded radiated field is combined in quadrature:

$$E_{\text{unshielded}} = \sqrt{E_{\text{DM}}^2 + E_{\text{CM}}^2}\quad (\text{V/m})$$

$$E_{\text{unshielded, dB}\mu\text{V/m}} = 20 \log_{10}\left(E_{\text{unshielded}} \times 10^6\right)$$

### 5. Enclosure Aperture Seam & Slot Shielding Effectiveness

Enclosure seams, display apertures, and connector openings behave as slot antennas. The shielding effectiveness of a slot of length $l_{\text{slot}}$ at wavelength $\lambda = c / f$ is:

$$SE_{\text{slot}}(f) = \max\left(0.0,\ 20 \log_{10}\left(\frac{\lambda}{2 l_{\text{slot}}}\right)\right)\quad (\text{dB})$$

If $l_{\text{slot}} \le 0.0$, the enclosure is seam-welded / continuous ($SE = 100.0\text{ dB}$). At frequencies where $\lambda \le 2 l_{\text{slot}}$, shielding drops to $0\text{ dB}$.

### 6. Trapezoidal Clock Fourier Harmonics & Spread-Spectrum Clocking

A digital clock with supply voltage $V_{\text{dd}}$, fundamental frequency $f_0$, duty cycle $D$, and 10%-90% rise time $t_r$ generates a Fourier harmonic envelope:

$$V(n) = 2 V_{\text{dd}} D \left| \frac{\sin(\pi n D)}{\pi n D} \right| \left| \frac{\sin(\pi n t_r / T)}{\pi n t_r / T} \right|$$

Spread-Spectrum Clocking (SSC) modulates the clock period over a down-spread bandwidth $\delta_{\text{ssc}} n f_0$, spreading spectral energy across the receiver's resolution bandwidth ($\text{RBW} = 120\text{ kHz}$ for CISPR Band B):

$$\Delta A_{\text{SSC}}(n) = 10 \log_{10}\left(1 + \frac{\delta_{\text{ssc}} n f_0}{\text{RBW}}\right)\quad (\text{dB})$$

---

## Regulatory Limits & Safety Margin Thresholds

`lh-emc-fcc` implements piecewise limit lines for FCC Part 15B and CISPR 32:

### FCC Part 15 Subpart B Limits

| Standard | Frequency Range | Distance | Quasi-Peak Limit | Linear Field |
|:---|:---:|:---:|:---:|:---:|
| **FCC Part 15B Class B** | 30 MHz – 88 MHz | 3 m | **40.0 dBµV/m** | 100 µV/m |
| (Consumer / Residential) | 88 MHz – 216 MHz | 3 m | **43.5 dBµV/m** | 150 µV/m |
| | 216 MHz – 960 MHz | 3 m | **46.0 dBµV/m** | 200 µV/m |
| | $\ge$ 960 MHz | 3 m | **54.0 dBµV/m** | 500 µV/m |
| **FCC Part 15B Class A** | 30 MHz – 88 MHz | 10 m / 3 m | **39.1 / 49.5 dBµV/m** | 90 / 300 µV/m |
| (Commercial / Industrial) | 88 MHz – 216 MHz | 10 m / 3 m | **43.5 / 54.0 dBµV/m** | 150 / 500 µV/m |
| | 216 MHz – 960 MHz | 10 m / 3 m | **46.4 / 56.9 dBµV/m** | 210 / 700 µV/m |
| | $\ge$ 960 MHz | 10 m / 3 m | **49.5 / 60.0 dBµV/m** | 300 / 1000 µV/m |

### CISPR 32 / EN 55032 Limits

| Standard | Frequency Range | Distance | Quasi-Peak Limit |
|:---|:---:|:---:|:---:|
| **CISPR 32 Class B** | 30 MHz – 230 MHz | 10 m / 3 m | **30.0 / 40.0 dBµV/m** |
| | 230 MHz – 1000 MHz | 10 m / 3 m | **37.0 / 47.0 dBµV/m** |
| **CISPR 32 Class A** | 30 MHz – 230 MHz | 10 m / 3 m | **40.0 / 50.0 dBµV/m** |
| | 230 MHz – 1000 MHz | 10 m / 3 m | **47.0 / 57.0 dBµV/m** |

### Margin Classification

- **`PASS`**: Margin $M = L_{\text{limit}} - E_{\text{meas}} \ge 6.0\text{ dB}$ (Production engineering tolerance satisfied).
- **`MARGINAL_WARNING`**: $0.0\text{ dB} \le M < 6.0\text{ dB}$ (Passes standard limit, but risks non-compliance due to component tolerances and temperature drift).
- **`FAIL_OVER_LIMIT`**: $M < 0.0\text{ dB}$ (Exceeds statutory regulatory limit).

---

## 10-Rule Hardware EMC, EMI & TEMPEST Security Posture Auditor

The posture auditor evaluates hardware layout and shielding against 10 rules. Posture score is determined by $S_{\text{posture}} = \max\left(0, 100 - \sum \text{Deductions}\right)$:

| Rule ID | Title | Severity | Weight | CWE | Failure Condition |
|:---|:---|:---:|:---:|:---:|:---|
| **LH-EMC-001** | Radiated Emission Engineering Margin | CRITICAL | 25 | CWE-682 | `worst_case_margin_db < 6.0` |
| **LH-EMC-002** | Split Reference Plane Traversal | CRITICAL | 25 | CWE-662 | `high_speed_traces_cross_split_plane == true` OR `unbroken_return_plane_verified == false` |
| **LH-EMC-003** | Cryptographic TEMPEST Shielding | CRITICAL | 25 | CWE-1300 | `tempest_crypto_shielding_enabled == false` OR `enclosure_shielding_effectiveness_db < 30.0` |
| **LH-EMC-004** | External I/O Common-Mode Filtering | HIGH | 10 | CWE-693 | `external_io_cmc_filtered == false` OR `cable_shield_360_terminated == false` |
| **LH-EMC-005** | Enclosure Seam & Via Stitching Pitch | HIGH | 10 | CWE-682 | `max_enclosure_slot_length_mm > 25.0` OR `perimeter_via_pitch_mm > 5.0` |
| **LH-EMC-006** | System Clock Harmonics & Damping | HIGH | 10 | CWE-682 | `ssc_enabled_on_system_clocks == false` AND `clock_series_damping_ohms < 10.0` |
| **LH-EMC-007** | SMPS High di/dt Switching Loop Area | HIGH | 10 | CWE-400 | `max_switching_loop_area_mm2 > 15.0` OR `lisn_conducted_filter_verified == false` |
| **LH-EMC-008** | Near-Field H-Probe Calibration | MEDIUM | 4 | CWE-20 | `probe_antenna_factor_calibrated == false` OR `rbw_120khz_qp_normalized == false` |
| **LH-EMC-009** | PDN Impedance & Decoupling Inductance | MEDIUM | 4 | CWE-693 | `pdn_target_impedance_met == false` OR `decoupling_loop_inductance_nh > 1.5` |
| **LH-EMC-010** | Ambient RF Baseline & Scan Logging | LOW | 1 | CWE-778 | `ambient_noise_baseline_subtracted == false` OR `scan_report_sha256_logged == false` |

---

## CLI Reference & Subcommands

### `lh-emc-fcc scan`

Audits a PCB layout and hardware EMC posture policy JSON file against all 10 rules.

```bash
# ANSI Terminal Table Output
lh-emc-fcc scan fixtures/hardened_emc_policy.json

# Machine-Readable JSON Output
lh-emc-fcc scan fixtures/hardened_emc_policy.json --format json

# OASIS SARIF v2.1.0 for CI/CD Code Scanning Integration
lh-emc-fcc scan fixtures/vulnerable_emc_policy.json --format sarif
```

### `lh-emc-fcc extrapolate-scan`

Converts near-field spectrum analyzer sweep telemetry into calibrated far-field radiated emissions and compares against regulatory limits.

```bash
# Extrapolate near-field scan to FCC Part 15B Class B at 3 meters
lh-emc-fcc extrapolate-scan fixtures/sample_nearfield_hscan.json --standard fcc-b --distance 3

# Extrapolate to CISPR 32 Class A at 10 meters with JSON output
lh-emc-fcc extrapolate-scan fixtures/sample_nearfield_hscan.json --standard cispr-a --distance 10 --format json
```

### `lh-emc-fcc clock-harmonics`

Models Fourier harmonic emission spectrum and Spread-Spectrum Clocking attenuation.

```bash
# Analyze 50 MHz clock with 1.5 ns rise time, 0.5% SSC, and series damping
lh-emc-fcc clock-harmonics --freq-mhz 50.0 --rise-ns 1.5 --vdd 3.3 --duty 0.5 --ssc-pct 0.5 --damping-ohms 22.0

# Add 35 dB enclosure shielding with strict compliance enforcement
lh-emc-fcc clock-harmonics --freq-mhz 50.0 --enclosure-db 35.0 --strict
```

### `lh-emc-fcc check-limit`

Performs an instantaneous verification of an emission data point against a standard limit line.

```bash
lh-emc-fcc check-limit --freq-mhz 100.0 --field-dbuv 30.0 --standard fcc-b --distance 3
```

---

## Verification & Test Suite

The verification suite validates analytical physics, standard limit masks, JSON parsing, SARIF formatting, and CLI subcommands:

```bash
# Run unit and integration tests
cargo test --verbose

# Run formatting and clippy linters with zero warnings
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

---

## License & Governance

`lh-emc-fcc-helper` is open-source software licensed under the **Apache License, Version 2.0**.

Copyright (c) 2026 Logihertz Systems (OPC) Private Limited  
Authored by **Sameer Tawade** `<ops@logihertz.com>`
