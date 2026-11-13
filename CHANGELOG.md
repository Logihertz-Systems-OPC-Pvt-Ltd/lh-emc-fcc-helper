# Changelog

All notable changes to `lh-emc-fcc-helper` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-13

### Added
- **Core Maxwell & Dipole Radiated Emission Engine**:
  - Near-field calibrated magnetic H-probe receiver voltage conversion ($V_{\text{dB}\mu\text{V}} = P_{\text{SA}} + 107.0 + L_{\text{cable}} - G_{\text{preamp}}$).
  - Equivalent RF current calculation via Ampere's loop law ($I_{\text{eq}} = 2\pi r_{\text{probe}} H_{\text{near}}$).
  - Small-loop Differential-Mode (DM) radiated electric field extrapolation with $+6\text{ dB}$ ground plane reflection.
  - Short-wire / cable monopole Common-Mode (CM) radiated electric field extrapolation.
  - Enclosure aperture slot shielding attenuation calculation ($SE_{\text{slot}} = 20 \log_{10}(\lambda / 2 l_{\text{slot}})$).
  - Trapezoidal clock pulse Fourier harmonic envelope calculation with Spread-Spectrum Clocking (SSC) peak reduction modeling.
- **FCC Part 15B & CISPR 32 Limit Verifier**:
  - Full piecewise frequency mask definitions for FCC Part 15B Class A ($10\text{ m}$ / $3\text{ m}$) and Class B ($3\text{ m}$).
  - Full piecewise frequency mask definitions for CISPR 32 Class A ($10\text{ m}$ / $3\text{ m}$) and Class B ($10\text{ m}$ / $3\text{ m}$).
  - Minimum engineering margin evaluation ($\ge 6.0\text{ dB}$) with automatic classification (`PASS`, `MARGINAL_WARNING`, `FAIL_OVER_LIMIT`).
- **10-Rule Hardware EMC & TEMPEST Posture Auditor**:
  - Implementation of rules `LH-EMC-001` through `LH-EMC-010` covering radiated emission margin, return current split planes, TEMPEST crypto shielding, common-mode I/O chokes, aperture slots, spread-spectrum clocking, SMPS hot loops, antenna calibration, PDN target impedance, and scan baseline subtraction.
  - Deterministic 0--100 scoring engine with weighted severity deductions.
- **Reporting & Exporters**:
  - ANSI colored terminal tabular report exporter.
  - Canonical RFC 8259 JSON report exporter.
  - OASIS SARIF v2.1.0 report exporter for automated CI/CD code scanning integration.
- **Zero-Dependency CLI Tool (`lh-emc-fcc`)**:
  - Subcommands: `scan`, `extrapolate-scan`, `clock-harmonics`, `check-limit`.
- **Zero External Crate Dependencies**:
  - 100% Rust standard library implementation with `#![forbid(unsafe_code)]`.
