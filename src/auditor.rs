//! 10-Rule Hardware EMC, EMI & TEMPEST Security Posture Auditor.
//!
//! Evaluates printed circuit board layout, enclosure shielding, clock distribution,
//! decoupling, and pre-compliance measurement parameters against 10 deterministic
//! defense-in-depth rules with weighted severity scoring (0 - 100).

#![forbid(unsafe_code)]

use crate::json::JsonValue;

/// Severity level for audit rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl RuleSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
        }
    }

    pub fn default_weight(&self) -> u32 {
        match self {
            Self::Critical => 25,
            Self::High => 10,
            Self::Medium => 4,
            Self::Low => 1,
        }
    }
}

/// Metadata definition of an audit rule.
#[derive(Debug, Clone)]
pub struct AuditRuleMeta {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: RuleSeverity,
    pub weight: u32,
    pub cwe: &'static str,
    pub description: &'static str,
    pub remediation: &'static str,
}

pub const AUDIT_RULES: &[AuditRuleMeta] = &[
    AuditRuleMeta {
        id: "LH-EMC-001",
        title: "Inadequate Radiated Emission Engineering Margin (< 6.0 dB)",
        severity: RuleSeverity::Critical,
        weight: 25,
        cwe: "CWE-682",
        description: "The worst-case pre-compliance radiated emission margin is less than the required 6.0 dB engineering safety threshold.",
        remediation: "Redesign board decoupling, enclose high-speed traces in inner stripline layers, or add common-mode chokes to increase compliance margin to at least 6.0 dB.",
    },
    AuditRuleMeta {
        id: "LH-EMC-002",
        title: "High-Speed Return Current Traversal of Split Reference Plane",
        severity: RuleSeverity::Critical,
        weight: 25,
        cwe: "CWE-662",
        description: "High-speed traces route across ground or power plane splits, or an unbroken reference plane has not been verified, forcing return current into large radiating loops.",
        remediation: "Ensure solid unbroken reference plane under all high-speed signals or bridge splits with low-impedance stitching capacitors.",
    },
    AuditRuleMeta {
        id: "LH-EMC-003",
        title: "Unshielded Cryptographic Subsystem / Sub-30dB TEMPEST Enclosure",
        severity: RuleSeverity::Critical,
        weight: 25,
        cwe: "CWE-1300",
        description: "Cryptographic hardware (TPM, HSM, secure MCU) lacks dedicated physical shielding can or enclosure shielding effectiveness is below 30.0 dB.",
        remediation: "Install board-level soldered RF shield cans over crypto processors and ensure conductive enclosure provides >= 30.0 dB attenuation.",
    },
    AuditRuleMeta {
        id: "LH-EMC-004",
        title: "Unfiltered External I/O Cabling or Non-360° Shield Termination",
        severity: RuleSeverity::High,
        weight: 10,
        cwe: "CWE-693",
        description: "External I/O lines lack common-mode chokes (CMCs) or cable shields lack 360-degree conductive perimeter termination to chassis ground.",
        remediation: "Add series common-mode chokes on differential pairs and ensure metallic backshells provide continuous 360-degree ground contact without pigtails.",
    },
    AuditRuleMeta {
        id: "LH-EMC-005",
        title: "Enclosure Aperture Seam (> 25mm) or Excessive Via Stitching Pitch (> 5mm)",
        severity: RuleSeverity::High,
        weight: 10,
        cwe: "CWE-682",
        description: "Enclosure slot length exceeds 25.0 mm or PCB perimeter ground via stitching pitch exceeds 5.0 mm, leaking high-frequency harmonics.",
        remediation: "Subdivide long enclosure seams using conductive gaskets and reduce perimeter via pitch to <= lambda/20 (typically <= 5.0 mm).",
    },
    AuditRuleMeta {
        id: "LH-EMC-006",
        title: "Unmitigated System Clock Harmonics (No SSC & Low Source Damping)",
        severity: RuleSeverity::High,
        weight: 10,
        cwe: "CWE-682",
        description: "System clock distribution lacks Spread-Spectrum Clocking (SSC) and has series source damping resistance below 10.0 Ohms.",
        remediation: "Enable 0.5% or 1.0% down-spread SSC on clock generators and place 22 - 33 Ohm series damping resistors close to driver pins.",
    },
    AuditRuleMeta {
        id: "LH-EMC-007",
        title: "Excessive SMPS Hot Loop Area (> 15mm²) or Unverified LISN Input Filter",
        severity: RuleSeverity::High,
        weight: 10,
        cwe: "CWE-400",
        description: "Switch-mode power supply high di/dt switching loop area exceeds 15.0 mm² or LISN conducted emissions input filter is unverified.",
        remediation: "Tighten input capacitor to MOSFET/inductor loop layout to <= 15.0 mm² and verify two-stage Pi filter on power input.",
    },
    AuditRuleMeta {
        id: "LH-EMC-008",
        title: "Uncalibrated Near-Field Probe Antenna Factor or Missing QP RBW Normalization",
        severity: RuleSeverity::Medium,
        weight: 4,
        cwe: "CWE-20",
        description: "Near-field H-probe antenna factor is uncalibrated or scan data is not normalized to 120 kHz Quasi-Peak bandwidth.",
        remediation: "Calibrate probe loop factor using TEM cell or reference microstrip fixture and set receiver bandwidth to standard 120 kHz QP.",
    },
    AuditRuleMeta {
        id: "LH-EMC-009",
        title: "Unmet PDN Target Impedance or High Decoupling Loop Inductance (> 1.5nH)",
        severity: RuleSeverity::Medium,
        weight: 4,
        cwe: "CWE-693",
        description: "Power Distribution Network (PDN) target impedance is unmet or decoupling capacitor mounting loop inductance exceeds 1.5 nH.",
        remediation: "Place 0402/0201 decoupling capacitors adjacent to IC power pins with side-by-side vias directly into planes to minimize loop inductance.",
    },
    AuditRuleMeta {
        id: "LH-EMC-010",
        title: "Unsubtracted Ambient RF Baseline or Missing Scan Cryptographic Hash",
        severity: RuleSeverity::Low,
        weight: 1,
        cwe: "CWE-778",
        description: "Pre-compliance scan lacks background ambient baseline subtraction or scan report SHA-256 cryptographic logging.",
        remediation: "Record an ambient RF scan with EUT powered off and log SHA-256 integrity hash of raw scan measurements.",
    },
];

/// Policy input fields extracted from a configuration document.
#[derive(Debug, Clone)]
pub struct EmcPolicyInput {
    pub worst_case_margin_db: f64,
    pub high_speed_traces_cross_split_plane: bool,
    pub unbroken_return_plane_verified: bool,
    pub tempest_crypto_shielding_enabled: bool,
    pub enclosure_shielding_effectiveness_db: f64,
    pub external_io_cmc_filtered: bool,
    pub cable_shield_360_terminated: bool,
    pub max_enclosure_slot_length_mm: f64,
    pub perimeter_via_pitch_mm: f64,
    pub ssc_enabled_on_system_clocks: bool,
    pub clock_series_damping_ohms: f64,
    pub max_switching_loop_area_mm2: f64,
    pub lisn_conducted_filter_verified: bool,
    pub probe_antenna_factor_calibrated: bool,
    pub rbw_120khz_qp_normalized: bool,
    pub pdn_target_impedance_met: bool,
    pub decoupling_loop_inductance_nh: f64,
    pub ambient_noise_baseline_subtracted: bool,
    pub scan_report_sha256_logged: bool,
}

impl Default for EmcPolicyInput {
    fn default() -> Self {
        Self {
            worst_case_margin_db: 8.5,
            high_speed_traces_cross_split_plane: false,
            unbroken_return_plane_verified: true,
            tempest_crypto_shielding_enabled: true,
            enclosure_shielding_effectiveness_db: 36.0,
            external_io_cmc_filtered: true,
            cable_shield_360_terminated: true,
            max_enclosure_slot_length_mm: 12.0,
            perimeter_via_pitch_mm: 3.0,
            ssc_enabled_on_system_clocks: true,
            clock_series_damping_ohms: 22.0,
            max_switching_loop_area_mm2: 8.5,
            lisn_conducted_filter_verified: true,
            probe_antenna_factor_calibrated: true,
            rbw_120khz_qp_normalized: true,
            pdn_target_impedance_met: true,
            decoupling_loop_inductance_nh: 0.85,
            ambient_noise_baseline_subtracted: true,
            scan_report_sha256_logged: true,
        }
    }
}

impl EmcPolicyInput {
    pub fn from_json(json: &JsonValue) -> Result<Self, String> {
        let mut policy = Self::default();

        if let Some(v) = json.get("worst_case_margin_db").and_then(|v| v.as_f64()) {
            policy.worst_case_margin_db = v;
        }
        if let Some(v) = json
            .get("high_speed_traces_cross_split_plane")
            .and_then(|v| v.as_bool())
        {
            policy.high_speed_traces_cross_split_plane = v;
        }
        if let Some(v) = json
            .get("unbroken_return_plane_verified")
            .and_then(|v| v.as_bool())
        {
            policy.unbroken_return_plane_verified = v;
        }
        if let Some(v) = json
            .get("tempest_crypto_shielding_enabled")
            .and_then(|v| v.as_bool())
        {
            policy.tempest_crypto_shielding_enabled = v;
        }
        if let Some(v) = json
            .get("enclosure_shielding_effectiveness_db")
            .and_then(|v| v.as_f64())
        {
            policy.enclosure_shielding_effectiveness_db = v;
        }
        if let Some(v) = json
            .get("external_io_cmc_filtered")
            .and_then(|v| v.as_bool())
        {
            policy.external_io_cmc_filtered = v;
        }
        if let Some(v) = json
            .get("cable_shield_360_terminated")
            .and_then(|v| v.as_bool())
        {
            policy.cable_shield_360_terminated = v;
        }
        if let Some(v) = json
            .get("max_enclosure_slot_length_mm")
            .and_then(|v| v.as_f64())
        {
            policy.max_enclosure_slot_length_mm = v;
        }
        if let Some(v) = json.get("perimeter_via_pitch_mm").and_then(|v| v.as_f64()) {
            policy.perimeter_via_pitch_mm = v;
        }
        if let Some(v) = json
            .get("ssc_enabled_on_system_clocks")
            .and_then(|v| v.as_bool())
        {
            policy.ssc_enabled_on_system_clocks = v;
        }
        if let Some(v) = json
            .get("clock_series_damping_ohms")
            .and_then(|v| v.as_f64())
        {
            policy.clock_series_damping_ohms = v;
        }
        if let Some(v) = json
            .get("max_switching_loop_area_mm2")
            .and_then(|v| v.as_f64())
        {
            policy.max_switching_loop_area_mm2 = v;
        }
        if let Some(v) = json
            .get("lisn_conducted_filter_verified")
            .and_then(|v| v.as_bool())
        {
            policy.lisn_conducted_filter_verified = v;
        }
        if let Some(v) = json
            .get("probe_antenna_factor_calibrated")
            .and_then(|v| v.as_bool())
        {
            policy.probe_antenna_factor_calibrated = v;
        }
        if let Some(v) = json
            .get("rbw_120khz_qp_normalized")
            .and_then(|v| v.as_bool())
        {
            policy.rbw_120khz_qp_normalized = v;
        }
        if let Some(v) = json
            .get("pdn_target_impedance_met")
            .and_then(|v| v.as_bool())
        {
            policy.pdn_target_impedance_met = v;
        }
        if let Some(v) = json
            .get("decoupling_loop_inductance_nh")
            .and_then(|v| v.as_f64())
        {
            policy.decoupling_loop_inductance_nh = v;
        }
        if let Some(v) = json
            .get("ambient_noise_baseline_subtracted")
            .and_then(|v| v.as_bool())
        {
            policy.ambient_noise_baseline_subtracted = v;
        }
        if let Some(v) = json
            .get("scan_report_sha256_logged")
            .and_then(|v| v.as_bool())
        {
            policy.scan_report_sha256_logged = v;
        }

        Ok(policy)
    }
}

/// Result of evaluating an individual audit rule.
#[derive(Debug, Clone)]
pub struct RuleEvaluation {
    pub rule: AuditRuleMeta,
    pub passed: bool,
    pub deduction: u32,
    pub observed_value: String,
}

/// Overall audit result report.
#[derive(Debug, Clone)]
pub struct AuditReport {
    pub evaluations: Vec<RuleEvaluation>,
    pub score: u32,
    pub total_deductions: u32,
    pub passed_rules_count: usize,
    pub failed_rules_count: usize,
    pub is_compliant: bool,
}

/// Evaluates the 10 EMC/TEMPEST posture rules against policy parameters.
pub fn run_emc_posture_audit(policy: &EmcPolicyInput) -> AuditReport {
    let mut evaluations = Vec::new();
    let mut total_deductions = 0u32;

    for rule in AUDIT_RULES {
        let (passed, observed_value) = match rule.id {
            "LH-EMC-001" => {
                let p = policy.worst_case_margin_db >= 6.0;
                let obs = format!(
                    "worst_case_margin_db = {:.2} dB (threshold: >= 6.0 dB)",
                    policy.worst_case_margin_db
                );
                (p, obs)
            }
            "LH-EMC-002" => {
                let p = !policy.high_speed_traces_cross_split_plane
                    && policy.unbroken_return_plane_verified;
                let obs = format!(
                    "split_plane_cross = {}, unbroken_verified = {}",
                    policy.high_speed_traces_cross_split_plane,
                    policy.unbroken_return_plane_verified
                );
                (p, obs)
            }
            "LH-EMC-003" => {
                let p = policy.tempest_crypto_shielding_enabled
                    && policy.enclosure_shielding_effectiveness_db >= 30.0;
                let obs = format!(
                    "crypto_shield = {}, enclosure_se = {:.1} dB (min 30.0 dB)",
                    policy.tempest_crypto_shielding_enabled,
                    policy.enclosure_shielding_effectiveness_db
                );
                (p, obs)
            }
            "LH-EMC-004" => {
                let p = policy.external_io_cmc_filtered && policy.cable_shield_360_terminated;
                let obs = format!(
                    "cmc_filtered = {}, 360_shield_term = {}",
                    policy.external_io_cmc_filtered, policy.cable_shield_360_terminated
                );
                (p, obs)
            }
            "LH-EMC-005" => {
                let p = policy.max_enclosure_slot_length_mm <= 25.0
                    && policy.perimeter_via_pitch_mm <= 5.0;
                let obs = format!(
                    "slot_len = {:.1} mm (max 25.0), via_pitch = {:.1} mm (max 5.0)",
                    policy.max_enclosure_slot_length_mm, policy.perimeter_via_pitch_mm
                );
                (p, obs)
            }
            "LH-EMC-006" => {
                let p =
                    policy.ssc_enabled_on_system_clocks || policy.clock_series_damping_ohms >= 10.0;
                let obs = format!(
                    "ssc_enabled = {}, series_damping = {:.1} Ohms (min 10.0)",
                    policy.ssc_enabled_on_system_clocks, policy.clock_series_damping_ohms
                );
                (p, obs)
            }
            "LH-EMC-007" => {
                let p = policy.max_switching_loop_area_mm2 <= 15.0
                    && policy.lisn_conducted_filter_verified;
                let obs = format!(
                    "smps_loop_area = {:.1} mm² (max 15.0), lisn_filter_verified = {}",
                    policy.max_switching_loop_area_mm2, policy.lisn_conducted_filter_verified
                );
                (p, obs)
            }
            "LH-EMC-008" => {
                let p = policy.probe_antenna_factor_calibrated && policy.rbw_120khz_qp_normalized;
                let obs = format!(
                    "probe_af_calibrated = {}, rbw_qp_normalized = {}",
                    policy.probe_antenna_factor_calibrated, policy.rbw_120khz_qp_normalized
                );
                (p, obs)
            }
            "LH-EMC-009" => {
                let p =
                    policy.pdn_target_impedance_met && policy.decoupling_loop_inductance_nh <= 1.5;
                let obs = format!(
                    "pdn_met = {}, loop_inductance = {:.2} nH (max 1.50 nH)",
                    policy.pdn_target_impedance_met, policy.decoupling_loop_inductance_nh
                );
                (p, obs)
            }
            "LH-EMC-010" => {
                let p =
                    policy.ambient_noise_baseline_subtracted && policy.scan_report_sha256_logged;
                let obs = format!(
                    "ambient_subtracted = {}, scan_sha256_logged = {}",
                    policy.ambient_noise_baseline_subtracted, policy.scan_report_sha256_logged
                );
                (p, obs)
            }
            _ => (false, "unknown rule".to_string()),
        };

        let deduction = if passed { 0 } else { rule.weight };
        total_deductions += deduction;

        evaluations.push(RuleEvaluation {
            rule: rule.clone(),
            passed,
            deduction,
            observed_value,
        });
    }

    let score = 100_u32.saturating_sub(total_deductions);
    let passed_rules_count = evaluations.iter().filter(|e| e.passed).count();
    let failed_rules_count = evaluations.len() - passed_rules_count;
    let is_compliant = score == 100 && failed_rules_count == 0;

    AuditReport {
        evaluations,
        score,
        total_deductions,
        passed_rules_count,
        failed_rules_count,
        is_compliant,
    }
}
