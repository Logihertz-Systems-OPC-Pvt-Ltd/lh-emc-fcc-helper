//! FCC Part 15B and CISPR 32 Radiated Emission Limits & Margin Evaluator.
//!
//! Provides piecewise regulatory limit lines for Class A and Class B digital devices
//! across 3-meter and 10-meter measurement distances, as well as margin classification
//! against engineering safety thresholds (default 6.0 dB).

#![forbid(unsafe_code)]

use std::fmt;

/// Supported EMC regulatory standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmcStandard {
    /// FCC Part 15, Subpart B, Class B (Consumer / Residential).
    FccPart15BClassB,
    /// FCC Part 15, Subpart B, Class A (Commercial / Industrial).
    FccPart15BClassA,
    /// CISPR 32 / EN 55032 Class B (Residential).
    Cispr32ClassB,
    /// CISPR 32 / EN 55032 Class A (Commercial).
    Cispr32ClassA,
}

impl EmcStandard {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "fcc-b" | "fcc_b" | "fcc-part-15b-b" | "fcc15b" => Some(Self::FccPart15BClassB),
            "fcc-a" | "fcc_a" | "fcc-part-15b-a" | "fcc15a" => Some(Self::FccPart15BClassA),
            "cispr-b" | "cispr_b" | "cispr32-b" | "cispr32b" => Some(Self::Cispr32ClassB),
            "cispr-a" | "cispr_a" | "cispr32-a" | "cispr32a" => Some(Self::Cispr32ClassA),
            _ => None,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::FccPart15BClassB => "FCC-Part15B-ClassB",
            Self::FccPart15BClassA => "FCC-Part15B-ClassA",
            Self::Cispr32ClassB => "CISPR-32-ClassB",
            Self::Cispr32ClassA => "CISPR-32-ClassA",
        }
    }
}

impl fmt::Display for EmcStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Compliance status of a frequency point against a regulatory limit line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceStatus {
    /// Radiated field is below the limit by at least the engineering safety margin (>= 6.0 dB).
    Pass,
    /// Radiated field is below the regulatory limit, but margin is < 6.0 dB.
    MarginalWarning,
    /// Radiated field exceeds the regulatory limit (margin < 0.0 dB).
    FailOverLimit,
}

impl ComplianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::MarginalWarning => "MARGINAL_WARNING",
            Self::FailOverLimit => "FAIL_OVER_LIMIT",
        }
    }
}

impl fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Evaluates the regulatory limit in dBµV/m for a specific standard, distance (m), and frequency (MHz).
pub fn get_regulatory_limit_dbuvm(standard: EmcStandard, distance_m: f64, freq_mhz: f64) -> f64 {
    let d: f64 = if distance_m <= 5.0 { 3.0 } else { 10.0 };

    match standard {
        EmcStandard::FccPart15BClassB => {
            // Standard FCC 15.109(a) limit is specified at 3 meters
            let limit_3m = if freq_mhz < 88.0 {
                40.0
            } else if freq_mhz < 216.0 {
                43.5
            } else if freq_mhz < 960.0 {
                46.0
            } else {
                54.0
            };
            if (d - 3.0_f64).abs() < 0.5 {
                limit_3m
            } else {
                // Extrapolate from 3m to 10m using 20*log10(3/10) = -10.45757 dB
                limit_3m - 10.457_57
            }
        }
        EmcStandard::FccPart15BClassA => {
            if (d - 10.0_f64).abs() < 0.5 {
                // FCC 15.109(a) Class A at 10 meters
                if freq_mhz < 88.0 {
                    39.1
                } else if freq_mhz < 216.0 {
                    43.5
                } else if freq_mhz < 960.0 {
                    46.4
                } else {
                    49.5
                }
            } else {
                // FCC 15.109(b) Class A at 3 meters
                if freq_mhz < 88.0 {
                    49.5
                } else if freq_mhz < 216.0 {
                    54.0
                } else if freq_mhz < 960.0 {
                    56.9
                } else {
                    60.0
                }
            }
        }
        EmcStandard::Cispr32ClassB => {
            if (d - 10.0_f64).abs() < 0.5 {
                // CISPR 32 Class B at 10m
                if freq_mhz < 230.0 {
                    30.0
                } else {
                    37.0
                }
            } else {
                // CISPR 32 Class B at 3m
                if freq_mhz < 230.0 {
                    40.0
                } else {
                    47.0
                }
            }
        }
        EmcStandard::Cispr32ClassA => {
            if (d - 10.0_f64).abs() < 0.5 {
                // CISPR 32 Class A at 10m
                if freq_mhz < 230.0 {
                    40.0
                } else {
                    47.0
                }
            } else {
                // CISPR 32 Class A at 3m
                if freq_mhz < 230.0 {
                    50.0
                } else {
                    57.0
                }
            }
        }
    }
}

/// Evaluates an emission data point against a regulatory limit and safety margin threshold.
pub fn evaluate_emission_margin(
    field_dbuvm: f64,
    limit_dbuvm: f64,
    safety_margin_db: f64,
) -> (f64, ComplianceStatus) {
    let margin = limit_dbuvm - field_dbuvm;
    let status = if margin < 0.0 {
        ComplianceStatus::FailOverLimit
    } else if margin < safety_margin_db {
        ComplianceStatus::MarginalWarning
    } else {
        ComplianceStatus::Pass
    };
    (margin, status)
}
