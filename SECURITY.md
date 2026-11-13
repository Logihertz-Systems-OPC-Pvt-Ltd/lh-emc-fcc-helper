# Security Policy

## Supported Versions

Logihertz Systems actively maintains and releases security patches for the following versions of `lh-emc-fcc-helper`:

| Version | Supported          | Security Maintenance Status |
|:-------:|:------------------:|:----------------------------|
| 0.1.x   | :white_check_mark: | Active Current Release      |
| < 0.1.0 | :x:                | Unsupported Pre-release     |

## Threat Model & Electromagnetic Security (EMSEC / TEMPEST)

`lh-emc-fcc-helper` calculates electromagnetic compatibility (EMC) pre-compliance radiated emissions and audits hardware printed circuit board (PCB) layouts against unintentional radio-frequency (RF) radiation and TEMPEST side-channel compromising emanations.

In high-assurance defense, aerospace, and sovereign edge computing deployments:
1. **Unintentional RF Radiation & Side-Channel Emission (CWE-1300)**: Cryptographic processing engines, secure elements, and high-speed memory buses leak microvolt-level high-frequency electromagnetic pulses. When coupled into unshielded traces, split ground planes, or peripheral I/O cables, these signals can be intercepted via sensitive software-defined radios (SDRs).
2. **Computational Verification of Regulatory Limits**: Truncation, integer overflow, or floating-point anomalies in Maxwell/dipole radiation formulas could cause an engineer to misjudge emission margins, resulting in regulatory rejection (FCC Part 15B / CISPR 32) or field susceptibility to high-intensity radiated fields (HIRF).
3. **Memory Safety & Zero Unsafe Code**: `lh-emc-fcc-helper` enforces `#![forbid(unsafe_code)]` across 100% of its Rust codebase, preventing buffer overflows, memory corruption, and undefined behavior during automated parsing of untrusted pre-compliance scan telemetry.

## Reporting a Vulnerability

If you discover a security vulnerability, algorithmic flaw, or posture rule bypass in `lh-emc-fcc-helper`, please disclose it responsibly:

1. **Email**: Send an encrypted or plain-text report directly to `ops@logihertz.com` with the subject line `[SECURITY] lh-emc-fcc-helper Vulnerability Report`.
2. **Details to Include**:
   - Description of the issue (e.g. calculation discrepancy, parsing crash, or rule logic bypass).
   - Minimal reproduction sample or test fixture (e.g. JSON scan file or policy definition).
   - Expected behavior versus observed behavior.
   - Potential impact on EMC compliance determination or hardware physical security.
3. **Response Timeline**:
   - **Initial Acknowledgement**: Within 24 hours.
   - **Assessment & Triage**: Within 48 hours.
   - **Remediation & Patch Release**: Within 7 business days for high-severity issues.

Please do not report security vulnerabilities via public GitHub issues.
