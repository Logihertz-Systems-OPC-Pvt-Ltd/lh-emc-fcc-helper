# Contributing to Logihertz EMC & FCC Pre-Compliance Helper

Thank you for your interest in contributing to `lh-emc-fcc-helper` (`lh-emc-fcc`), part of the Logihertz 60-Tool Open-Source Ecosystem.

## Architectural Principles & Hard Constraints

`lh-emc-fcc-helper` is engineered for mission-critical hardware pre-compliance lab testing, automated CI/CD PCB layout verification, and defense-grade EMSEC/TEMPEST auditing. All contributions must adhere to the following strict architectural rules:

1. **100% Zero External Dependencies**:
   - The `[dependencies]` section in `Cargo.toml` MUST remain completely empty.
   - All physics calculations (Maxwell loop equations, monopole cable models, aperture slot attenuation, Fourier trapezoidal harmonic series), limit verifiers, JSON parsers, SARIF emitters, and posture auditors must be implemented exclusively using the Rust standard library (`std`).
2. **Zero Unsafe Code**:
   - `#![forbid(unsafe_code)]` is strictly mandated across all crates and source files. Pull requests containing `unsafe` blocks will be rejected automatically.
3. **Deterministic Math & Numerical Robustness**:
   - Calculations must guard against division-by-zero, negative logarithms, and NaN propagations. Standard SI physical constants ($c = 299,792,458.0\text{ m/s}$, $\eta_0 = 376.730313668\ \Omega$) must be maintained with double-precision floating-point accuracy (`f64`).
4. **Comprehensive Unit & Integration Testing**:
   - Every physical equation, regulatory limit line, and posture audit rule must have corresponding unit and integration test coverage.

## Local Development Workflow

### Prerequisites
- Rust stable toolchain (Rust 2021 Edition, 1.75+ recommended)
- `cargo`, `rustfmt`, and `clippy`

### Building the Project
```bash
cargo build
```

### Running Tests
```bash
cargo test --verbose
```

### Formatting & Linting
All submissions must pass strict formatting and clippy checks without warnings:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Pull Request Guidelines

1. **Commit Messages**: Follow the Conventional Commits specification (e.g. `feat(physics): add multi-aperture array shielding factor`, `fix(limits): correct CISPR 32 Class A 3m band boundary`).
2. **Documentation**: Update `README.md` and inline doc comments (`///`) when introducing or modifying CLI commands or mathematical models.
3. **Fixtures**: Include test fixtures in `fixtures/` if adding new audit rules or scan schema options.
