//! Exporters for EMC Pre-Compliance and Audit Reports.
//!
//! Formats evaluation results into ANSI colored terminal tables, canonical RFC 8259 JSON,
//! or OASIS SARIF v2.1.0 format for automated CI/CD security scanning.

#![forbid(unsafe_code)]

use crate::auditor::AuditReport;
use crate::fcc_cispr_limits::ComplianceStatus;
use crate::json::JsonValue;

/// An extrapolated measurement point.
#[derive(Debug, Clone)]
pub struct ExtrapolatedPoint {
    pub freq_mhz: f64,
    pub measured_psa_dbm: f64,
    pub receiver_voltage_dbuv: f64,
    pub magnetic_field_dbuam: f64,
    pub equivalent_current_ma: f64,
    pub unshielded_field_dbuvm: f64,
    pub shielding_eff_db: f64,
    pub shielded_field_dbuvm: f64,
    pub regulatory_limit_dbuvm: f64,
    pub margin_db: f64,
    pub status: ComplianceStatus,
}

/// An evaluated clock harmonic point.
#[derive(Debug, Clone)]
pub struct ClockHarmonicPoint {
    pub harmonic_n: u32,
    pub freq_mhz: f64,
    pub envelope_volts: f64,
    pub base_field_dbuvm: f64,
    pub ssc_attenuation_db: f64,
    pub net_field_dbuvm: f64,
    pub limit_dbuvm: f64,
    pub margin_db: f64,
    pub status: ComplianceStatus,
}

/// Formats an audit report as an ANSI colored table.
pub fn format_audit_table(report: &AuditReport) -> String {
    let mut out = String::new();
    out.push_str("========================================================================================================\n");
    out.push_str("                        LOGIHERTZ EMC / EMI & TEMPEST POSTURE AUDIT REPORT                      \n");
    out.push_str("========================================================================================================\n");
    out.push_str(&format!(
        " Overall Score: {} / 100  |  Status: {}  |  Deductions: -{}\n",
        report.score,
        if report.is_compliant {
            "\x1b[32mCOMPLIANT\x1b[0m"
        } else {
            "\x1b[31mNON-COMPLIANT\x1b[0m"
        },
        report.total_deductions
    ));
    out.push_str(&format!(
        " Rules Evaluated: {}  |  Passed: \x1b[32m{}\x1b[0m  |  Failed: \x1b[31m{}\x1b[0m\n",
        report.evaluations.len(),
        report.passed_rules_count,
        report.failed_rules_count
    ));
    out.push_str("--------------------------------------------------------------------------------------------------------\n");
    out.push_str(
        " RULE ID    | SEVERITY | DEDUCT | STATUS  | DETAILS                                                     \n",
    );
    out.push_str("--------------------------------------------------------------------------------------------------------\n");

    for ev in &report.evaluations {
        let status_str = if ev.passed {
            "\x1b[32mPASS\x1b[0m   "
        } else {
            "\x1b[31mFAIL\x1b[0m   "
        };
        let sev_str = match ev.rule.severity.as_str() {
            "CRITICAL" => "\x1b[31mCRITICAL\x1b[0m",
            "HIGH" => "\x1b[33mHIGH    \x1b[0m",
            "MEDIUM" => "\x1b[36mMEDIUM  \x1b[0m",
            _ => "\x1b[37mLOW     \x1b[0m",
        };
        out.push_str(&format!(
            " {:<10} | {} | {:>6} | {} | {:<58}\n",
            ev.rule.id, sev_str, ev.deduction, status_str, ev.observed_value
        ));
    }
    out.push_str("========================================================================================================\n");
    out
}

/// Formats an audit report as canonical JSON.
pub fn format_audit_json(report: &AuditReport) -> String {
    let mut ev_json = Vec::new();
    for ev in &report.evaluations {
        let rule_obj = JsonValue::Object(vec![
            ("id".to_string(), JsonValue::String(ev.rule.id.to_string())),
            (
                "title".to_string(),
                JsonValue::String(ev.rule.title.to_string()),
            ),
            (
                "severity".to_string(),
                JsonValue::String(ev.rule.severity.as_str().to_string()),
            ),
            (
                "weight".to_string(),
                JsonValue::Number(ev.rule.weight as f64),
            ),
            (
                "cwe".to_string(),
                JsonValue::String(ev.rule.cwe.to_string()),
            ),
            (
                "description".to_string(),
                JsonValue::String(ev.rule.description.to_string()),
            ),
            (
                "remediation".to_string(),
                JsonValue::String(ev.rule.remediation.to_string()),
            ),
        ]);

        let ev_obj = JsonValue::Object(vec![
            ("rule".to_string(), rule_obj),
            ("passed".to_string(), JsonValue::Bool(ev.passed)),
            (
                "deduction".to_string(),
                JsonValue::Number(ev.deduction as f64),
            ),
            (
                "observed_value".to_string(),
                JsonValue::String(ev.observed_value.clone()),
            ),
        ]);
        ev_json.push(ev_obj);
    }

    let root = JsonValue::Object(vec![
        ("score".to_string(), JsonValue::Number(report.score as f64)),
        (
            "total_deductions".to_string(),
            JsonValue::Number(report.total_deductions as f64),
        ),
        (
            "passed_rules_count".to_string(),
            JsonValue::Number(report.passed_rules_count as f64),
        ),
        (
            "failed_rules_count".to_string(),
            JsonValue::Number(report.failed_rules_count as f64),
        ),
        (
            "is_compliant".to_string(),
            JsonValue::Bool(report.is_compliant),
        ),
        ("evaluations".to_string(), JsonValue::Array(ev_json)),
    ]);

    root.to_pretty_string()
}

/// Formats an audit report as OASIS SARIF v2.1.0 JSON.
pub fn format_audit_sarif(report: &AuditReport) -> String {
    let mut rules_sarif = Vec::new();
    let mut results_sarif = Vec::new();

    for ev in &report.evaluations {
        let rule_id = ev.rule.id;
        let rule_level = match ev.rule.severity.as_str() {
            "CRITICAL" | "HIGH" => "error",
            "MEDIUM" => "warning",
            _ => "note",
        };

        rules_sarif.push(JsonValue::Object(vec![
            ("id".to_string(), JsonValue::String(rule_id.to_string())),
            (
                "name".to_string(),
                JsonValue::String(ev.rule.title.to_string()),
            ),
            (
                "shortDescription".to_string(),
                JsonValue::Object(vec![(
                    "text".to_string(),
                    JsonValue::String(ev.rule.title.to_string()),
                )]),
            ),
            (
                "fullDescription".to_string(),
                JsonValue::Object(vec![(
                    "text".to_string(),
                    JsonValue::String(ev.rule.description.to_string()),
                )]),
            ),
            (
                "help".to_string(),
                JsonValue::Object(vec![(
                    "text".to_string(),
                    JsonValue::String(format!(
                        "Remediation: {}\nCWE: {}",
                        ev.rule.remediation, ev.rule.cwe
                    )),
                )]),
            ),
        ]));

        if !ev.passed {
            results_sarif.push(JsonValue::Object(vec![
                ("ruleId".to_string(), JsonValue::String(rule_id.to_string())),
                (
                    "level".to_string(),
                    JsonValue::String(rule_level.to_string()),
                ),
                (
                    "message".to_string(),
                    JsonValue::Object(vec![(
                        "text".to_string(),
                        JsonValue::String(format!(
                            "{}: {}. Observed: {}",
                            ev.rule.title, ev.rule.description, ev.observed_value
                        )),
                    )]),
                ),
                (
                    "locations".to_string(),
                    JsonValue::Array(vec![JsonValue::Object(vec![(
                        "physicalLocation".to_string(),
                        JsonValue::Object(vec![(
                            "artifactLocation".to_string(),
                            JsonValue::Object(vec![(
                                "uri".to_string(),
                                JsonValue::String("hardware/pcb-layout/policy.json".to_string()),
                            )]),
                        )]),
                    )])]),
                ),
            ]));
        }
    }

    let driver = JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonValue::String("lh-emc-fcc".to_string()),
        ),
        (
            "version".to_string(),
            JsonValue::String(env!("CARGO_PKG_VERSION").to_string()),
        ),
        (
            "informationUri".to_string(),
            JsonValue::String(
                "https://github.com/Logihertz-Systems-OPC-Pvt-Ltd/lh-emc-fcc-helper".to_string(),
            ),
        ),
        ("rules".to_string(), JsonValue::Array(rules_sarif)),
    ]);

    let tool = JsonValue::Object(vec![("driver".to_string(), driver)]);

    let run = JsonValue::Object(vec![
        ("tool".to_string(), tool),
        ("results".to_string(), JsonValue::Array(results_sarif)),
    ]);

    let root = JsonValue::Object(vec![
        (
            "version".to_string(),
            JsonValue::String("2.1.0".to_string()),
        ),
        (
            "$schema".to_string(),
            JsonValue::String("https://json.schemastore.org/sarif-2.1.0.json".to_string()),
        ),
        ("runs".to_string(), JsonValue::Array(vec![run])),
    ]);

    root.to_pretty_string()
}

/// Formats extrapolated scan results as an ANSI table.
pub fn format_scan_table(
    points: &[ExtrapolatedPoint],
    standard_name: &str,
    distance_m: f64,
) -> String {
    let mut out = String::new();
    out.push_str("========================================================================================================\n");
    out.push_str(&format!(
        " NEAR-FIELD H-PROBE TO FAR-FIELD RADIATED EMISSION EXTRAPOLATION ({}, {:.1}m) \n",
        standard_name, distance_m
    ));
    out.push_str("========================================================================================================\n");
    out.push_str(
        " FREQ (MHz) | P_SA (dBm) | I_EQ (mA) | UNSHIELD (dBµV/m) | SE (dB) | FIELD (dBµV/m) | LIMIT | MARGIN (dB) | STATUS\n",
    );
    out.push_str("--------------------------------------------------------------------------------------------------------\n");

    for p in points {
        let status_color = match p.status {
            ComplianceStatus::Pass => "\x1b[32mPASS\x1b[0m            ",
            ComplianceStatus::MarginalWarning => "\x1b[33mMARGINAL_WARNING\x1b[0m",
            ComplianceStatus::FailOverLimit => "\x1b[31mFAIL_OVER_LIMIT \x1b[0m",
        };
        out.push_str(&format!(
            " {:>10.2} | {:>10.1} | {:>9.2} | {:>17.2} | {:>7.1} | {:>14.2} | {:>5.1} | {:>11.2} | {}\n",
            p.freq_mhz,
            p.measured_psa_dbm,
            p.equivalent_current_ma,
            p.unshielded_field_dbuvm,
            p.shielding_eff_db,
            p.shielded_field_dbuvm,
            p.regulatory_limit_dbuvm,
            p.margin_db,
            status_color
        ));
    }
    out.push_str("========================================================================================================\n");
    out
}

/// Formats extrapolated scan results as JSON.
pub fn format_scan_json(
    points: &[ExtrapolatedPoint],
    standard_name: &str,
    distance_m: f64,
) -> String {
    let mut pts_json = Vec::new();
    for p in points {
        let obj = JsonValue::Object(vec![
            ("freq_mhz".to_string(), JsonValue::Number(p.freq_mhz)),
            (
                "measured_psa_dbm".to_string(),
                JsonValue::Number(p.measured_psa_dbm),
            ),
            (
                "receiver_voltage_dbuv".to_string(),
                JsonValue::Number(p.receiver_voltage_dbuv),
            ),
            (
                "magnetic_field_dbuam".to_string(),
                JsonValue::Number(p.magnetic_field_dbuam),
            ),
            (
                "equivalent_current_ma".to_string(),
                JsonValue::Number(p.equivalent_current_ma),
            ),
            (
                "unshielded_field_dbuvm".to_string(),
                JsonValue::Number(p.unshielded_field_dbuvm),
            ),
            (
                "shielding_eff_db".to_string(),
                JsonValue::Number(p.shielding_eff_db),
            ),
            (
                "shielded_field_dbuvm".to_string(),
                JsonValue::Number(p.shielded_field_dbuvm),
            ),
            (
                "regulatory_limit_dbuvm".to_string(),
                JsonValue::Number(p.regulatory_limit_dbuvm),
            ),
            ("margin_db".to_string(), JsonValue::Number(p.margin_db)),
            (
                "status".to_string(),
                JsonValue::String(p.status.as_str().to_string()),
            ),
        ]);
        pts_json.push(obj);
    }

    let min_margin = points
        .iter()
        .map(|p| p.margin_db)
        .fold(f64::INFINITY, f64::min);
    let all_passed = points.iter().all(|p| p.status == ComplianceStatus::Pass);

    let root = JsonValue::Object(vec![
        (
            "standard".to_string(),
            JsonValue::String(standard_name.to_string()),
        ),
        ("distance_m".to_string(), JsonValue::Number(distance_m)),
        (
            "worst_case_margin_db".to_string(),
            JsonValue::Number(min_margin),
        ),
        ("compliant".to_string(), JsonValue::Bool(all_passed)),
        ("points".to_string(), JsonValue::Array(pts_json)),
    ]);

    root.to_pretty_string()
}

/// Formats clock harmonic analysis results as an ANSI table.
pub fn format_harmonics_table(
    points: &[ClockHarmonicPoint],
    f0_mhz: f64,
    vdd: f64,
    tr_ns: f64,
    ssc_pct: f64,
) -> String {
    let mut out = String::new();
    out.push_str("========================================================================================================\n");
    out.push_str(&format!(
        " CLOCK HARMONIC FOURIER SPECTRUM & SSC REDUCTION (f0={:.2}MHz, Vdd={:.2}V, tr={:.2}ns, SSC={:.2}%) \n",
        f0_mhz, vdd, tr_ns, ssc_pct
    ));
    out.push_str("========================================================================================================\n");
    out.push_str(
        " HARM # | FREQ (MHz) | ENVELOPE (V) | BASE (dBµV/m) | ΔSSC (dB) | NET (dBµV/m) | LIMIT | MARGIN (dB) | STATUS\n",
    );
    out.push_str("--------------------------------------------------------------------------------------------------------\n");

    for p in points {
        let status_color = match p.status {
            ComplianceStatus::Pass => "\x1b[32mPASS\x1b[0m            ",
            ComplianceStatus::MarginalWarning => "\x1b[33mMARGINAL_WARNING\x1b[0m",
            ComplianceStatus::FailOverLimit => "\x1b[31mFAIL_OVER_LIMIT \x1b[0m",
        };
        out.push_str(&format!(
            " {:>6} | {:>10.2} | {:>12.4} | {:>13.2} | {:>9.2} | {:>12.2} | {:>5.1} | {:>11.2} | {}\n",
            p.harmonic_n,
            p.freq_mhz,
            p.envelope_volts,
            p.base_field_dbuvm,
            p.ssc_attenuation_db,
            p.net_field_dbuvm,
            p.limit_dbuvm,
            p.margin_db,
            status_color
        ));
    }
    out.push_str("========================================================================================================\n");
    out
}

/// Formats clock harmonic analysis results as JSON.
pub fn format_harmonics_json(
    points: &[ClockHarmonicPoint],
    f0_mhz: f64,
    vdd: f64,
    tr_ns: f64,
    ssc_pct: f64,
) -> String {
    let mut pts_json = Vec::new();
    for p in points {
        let obj = JsonValue::Object(vec![
            (
                "harmonic_n".to_string(),
                JsonValue::Number(p.harmonic_n as f64),
            ),
            ("freq_mhz".to_string(), JsonValue::Number(p.freq_mhz)),
            (
                "envelope_volts".to_string(),
                JsonValue::Number(p.envelope_volts),
            ),
            (
                "base_field_dbuvm".to_string(),
                JsonValue::Number(p.base_field_dbuvm),
            ),
            (
                "ssc_attenuation_db".to_string(),
                JsonValue::Number(p.ssc_attenuation_db),
            ),
            (
                "net_field_dbuvm".to_string(),
                JsonValue::Number(p.net_field_dbuvm),
            ),
            ("limit_dbuvm".to_string(), JsonValue::Number(p.limit_dbuvm)),
            ("margin_db".to_string(), JsonValue::Number(p.margin_db)),
            (
                "status".to_string(),
                JsonValue::String(p.status.as_str().to_string()),
            ),
        ]);
        pts_json.push(obj);
    }

    let root = JsonValue::Object(vec![
        ("f0_mhz".to_string(), JsonValue::Number(f0_mhz)),
        ("vdd".to_string(), JsonValue::Number(vdd)),
        ("tr_ns".to_string(), JsonValue::Number(tr_ns)),
        ("ssc_pct".to_string(), JsonValue::Number(ssc_pct)),
        ("harmonics".to_string(), JsonValue::Array(pts_json)),
    ]);

    root.to_pretty_string()
}
