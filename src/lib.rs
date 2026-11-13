//! # lh-emc-fcc-helper
//!
//! Production-grade, zero-dependency Rust Hardware EMC Pre-Compliance Calculator,
//! Near-Field H-Probe to Far-Field Radiated Emission Extrapolator, FCC Part 15B / CISPR 32
//! Limit Verifier, and 10-Rule EMI & TEMPEST Security Posture Auditor.

#![forbid(unsafe_code)]

pub mod auditor;
pub mod emi_physics;
pub mod exporters;
pub mod fcc_cispr_limits;
pub mod json;
