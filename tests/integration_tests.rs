//! Comprehensive Integration and Verification Tests for `lh-emc-fcc-helper`.

#![forbid(unsafe_code)]

use std::fs;
use std::process::Command;

use lh_emc_fcc::auditor::{run_emc_posture_audit, EmcPolicyInput};
use lh_emc_fcc::emi_physics::{
    common_mode_radiated_field_vm, diff_mode_radiated_field_vm, electric_field_dbuvm,
    equivalent_rf_current, magnetic_field_dbuam, magnetic_field_linear_am, receiver_voltage_dbuv,
    slot_shielding_effectiveness_db, ssc_attenuation_db, total_electric_field_vm,
    trapezoidal_harmonic_voltage,
};
use lh_emc_fcc::exporters::{format_audit_json, format_audit_sarif, format_audit_table};
use lh_emc_fcc::fcc_cispr_limits::{
    evaluate_emission_margin, get_regulatory_limit_dbuvm, ComplianceStatus, EmcStandard,
};
use lh_emc_fcc::json::JsonParser;

#[test]
fn test_receiver_voltage_conversion() {
    // 0 dBm = 107 dBuV into 50 Ohms
    let v0 = receiver_voltage_dbuv(0.0, 0.0, 0.0);
    assert!((v0 - 107.0).abs() < 1e-6);

    // -50 dBm + 1.5 dB cable loss - 10 dB preamp gain = 48.5 dBuV
    let v1 = receiver_voltage_dbuv(-50.0, 1.5, 10.0);
    assert!((v1 - 48.5).abs() < 1e-6);
}

#[test]
fn test_magnetic_field_dbuam_and_linear() {
    let v_dbuv = 60.0;
    let af_h = 20.0;
    let h_dbuam = magnetic_field_dbuam(v_dbuv, af_h);
    assert!((h_dbuam - 80.0).abs() < 1e-6);

    // 80 dBuA/m = 10 ^ ((80 - 120)/20) = 10 ^ (-2) = 0.01 A/m
    let h_am = magnetic_field_linear_am(h_dbuam);
    assert!((h_am - 0.01).abs() < 1e-8);
}

#[test]
fn test_equivalent_rf_current() {
    // Probe radius 5 mm (0.005 m), H = 0.01 A/m
    // I_eq = 2 * pi * 0.005 * 0.01 = 0.000314159 A = ~0.314 mA
    let h_am = 0.01;
    let i_eq = equivalent_rf_current(h_am, 0.005);
    let expected = 2.0 * std::f64::consts::PI * 0.005 * 0.01;
    assert!((i_eq - expected).abs() < 1e-9);
}

#[test]
fn test_diff_mode_radiated_field_physics() {
    let f = 100_000_000.0; // 100 MHz
    let d = 3.0; // 3 m
    let a_loop = 10.0 * 1e-6; // 10 mm²
    let i_dm = 0.001; // 1 mA

    let e_dm = diff_mode_radiated_field_vm(f, d, a_loop, i_dm);
    assert!(e_dm > 0.0);

    // Verify f² frequency dependence: doubling frequency quadruples field (approx +12 dB)
    let e_dm_2f = diff_mode_radiated_field_vm(f * 2.0, d, a_loop, i_dm);
    let ratio = e_dm_2f / e_dm;
    assert!((ratio - 4.0).abs() < 1e-4);

    // Verify 1/d distance scaling: doubling distance halves field
    let e_dm_2d = diff_mode_radiated_field_vm(f, d * 2.0, a_loop, i_dm);
    assert!((e_dm_2d / e_dm - 0.5).abs() < 1e-4);
}

#[test]
fn test_common_mode_radiated_field_physics() {
    let f = 100_000_000.0; // 100 MHz
    let d = 3.0; // 3 m
    let l_cm = 0.5; // 0.5 m cable
    let i_cm = 0.00005; // 50 uA

    let e_cm = common_mode_radiated_field_vm(f, d, l_cm, i_cm);
    assert!(e_cm > 0.0);

    // Verify linear f dependence: doubling frequency doubles field
    let e_cm_2f = common_mode_radiated_field_vm(f * 2.0, d, l_cm, i_cm);
    assert!((e_cm_2f / e_cm - 2.0).abs() < 1e-4);
}

#[test]
fn test_total_field_and_dbuvm_conversion() {
    let e1 = 0.0001; // 100 uV/m = 40 dBuV/m
    let e2 = 0.0;
    let etot = total_electric_field_vm(e1, e2);
    assert!((etot - 0.0001).abs() < 1e-12);

    let dbuvm = electric_field_dbuvm(etot);
    assert!((dbuvm - 40.0).abs() < 1e-4);
}

#[test]
fn test_slot_shielding_effectiveness() {
    // Slot length = 0 returns 100 dB
    assert_eq!(slot_shielding_effectiveness_db(100e6, 0.0), 100.0);

    // At 100 MHz, lambda = c / 100e6 = ~2.9979 m
    // For slot length = 0.05 m (50 mm), cutoff = 2 * 0.05 = 0.1 m
    // ratio = 2.9979 / 0.1 = ~29.979
    // SE = 20 * log10(29.979) = ~29.53 dB
    let se = slot_shielding_effectiveness_db(100e6, 0.05);
    assert!(se > 28.0 && se < 31.0);

    // If slot length > lambda / 2, SE drops to 0 dB
    let se_res = slot_shielding_effectiveness_db(100e6, 2.0); // 2m > 1.5m
    assert_eq!(se_res, 0.0);
}

#[test]
fn test_trapezoidal_harmonic_voltage() {
    let f0 = 100e6;
    let vdd = 3.3;
    let duty = 0.5;
    let tr = 1.0e-9;

    // Harmonic 0 should be 0.0
    assert_eq!(trapezoidal_harmonic_voltage(0, vdd, duty, tr, f0), 0.0);

    // Fundamental (n=1)
    let v1 = trapezoidal_harmonic_voltage(1, vdd, duty, tr, f0);
    assert!(v1 > 1.5 && v1 < 3.3);

    // Even harmonics for 50% duty cycle should be nearly zero due to sinc(n * 0.5)
    let v2 = trapezoidal_harmonic_voltage(2, vdd, duty, tr, f0);
    assert!(v2 < 1e-6);

    // Odd harmonic (n=3)
    let v3 = trapezoidal_harmonic_voltage(3, vdd, duty, tr, f0);
    assert!(v3 > 0.1);
}

#[test]
fn test_ssc_attenuation() {
    let f0 = 100e6;
    let ssc_pct = 0.5; // 0.5%
    let rbw = 120_000.0;

    let delta1 = ssc_attenuation_db(1, f0, ssc_pct, rbw);
    // delta_ssc * 1 * 100e6 = 0.005 * 100e6 = 500,000 Hz
    // ratio = 1 + 500k / 120k = 1 + 4.166 = 5.166
    // 10 * log10(5.166) = ~7.13 dB
    assert!(delta1 > 6.5 && delta1 < 7.5);

    // Higher harmonic has wider spread, so greater attenuation
    let delta3 = ssc_attenuation_db(3, f0, ssc_pct, rbw);
    assert!(delta3 > delta1);
}

#[test]
fn test_fcc_part15b_limits() {
    // Class B at 3m
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassB, 3.0, 50.0),
        40.0
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassB, 3.0, 150.0),
        43.5
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassB, 3.0, 500.0),
        46.0
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassB, 3.0, 1000.0),
        54.0
    );

    // Class A at 10m
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassA, 10.0, 50.0),
        39.1
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassA, 10.0, 150.0),
        43.5
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassA, 10.0, 500.0),
        46.4
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::FccPart15BClassA, 10.0, 1000.0),
        49.5
    );
}

#[test]
fn test_cispr32_limits() {
    // CISPR 32 Class B at 10m
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::Cispr32ClassB, 10.0, 100.0),
        30.0
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::Cispr32ClassB, 10.0, 500.0),
        37.0
    );

    // CISPR 32 Class B at 3m
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::Cispr32ClassB, 3.0, 100.0),
        40.0
    );
    assert_eq!(
        get_regulatory_limit_dbuvm(EmcStandard::Cispr32ClassB, 3.0, 500.0),
        47.0
    );
}

#[test]
fn test_margin_evaluation() {
    let limit = 40.0;

    // Field 30.0 -> Margin 10.0 >= 6.0 => Pass
    let (m1, s1) = evaluate_emission_margin(30.0, limit, 6.0);
    assert_eq!(m1, 10.0);
    assert_eq!(s1, ComplianceStatus::Pass);

    // Field 36.0 -> Margin 4.0 in [0, 6) => MarginalWarning
    let (m2, s2) = evaluate_emission_margin(36.0, limit, 6.0);
    assert_eq!(m2, 4.0);
    assert_eq!(s2, ComplianceStatus::MarginalWarning);

    // Field 42.0 -> Margin -2.0 < 0 => FailOverLimit
    let (m3, s3) = evaluate_emission_margin(42.0, limit, 6.0);
    assert_eq!(m3, -2.0);
    assert_eq!(s3, ComplianceStatus::FailOverLimit);
}

#[test]
fn test_json_parser_and_serializer() {
    let raw = r#"{
        "name": "EUT-1",
        "freq": 100.5,
        "enabled": true,
        "tags": ["rf", "emc"],
        "nested": {
            "score": 100
        }
    }"#;

    let parsed = JsonParser::parse(raw).expect("valid json");
    assert_eq!(parsed.get("name").unwrap().as_str(), Some("EUT-1"));
    assert_eq!(parsed.get("freq").unwrap().as_f64(), Some(100.5));
    assert_eq!(parsed.get("enabled").unwrap().as_bool(), Some(true));
    assert_eq!(parsed.get("tags").unwrap().as_array().unwrap().len(), 2);
    assert_eq!(
        parsed.get("nested").unwrap().get("score").unwrap().as_f64(),
        Some(100.0)
    );

    let compact = parsed.to_compact_string();
    let re_parsed = JsonParser::parse(&compact).expect("valid compact json");
    assert_eq!(parsed, re_parsed);
}

#[test]
fn test_auditor_hardened_fixture() {
    let content = fs::read_to_string("fixtures/hardened_emc_policy.json").expect("read hardened");
    let json_val = JsonParser::parse(&content).expect("parse json");
    let policy = EmcPolicyInput::from_json(&json_val).expect("policy from json");

    let report = run_emc_posture_audit(&policy);
    assert_eq!(report.score, 100);
    assert_eq!(report.total_deductions, 0);
    assert_eq!(report.failed_rules_count, 0);
    assert_eq!(report.passed_rules_count, 10);
    assert!(report.is_compliant);
}

#[test]
fn test_auditor_vulnerable_fixture() {
    let content =
        fs::read_to_string("fixtures/vulnerable_emc_policy.json").expect("read vulnerable");
    let json_val = JsonParser::parse(&content).expect("parse json");
    let policy = EmcPolicyInput::from_json(&json_val).expect("policy from json");

    let report = run_emc_posture_audit(&policy);
    assert_eq!(report.score, 0);
    assert_eq!(report.total_deductions, 124);
    assert_eq!(report.failed_rules_count, 10);
    assert_eq!(report.passed_rules_count, 0);
    assert!(!report.is_compliant);
}

#[test]
fn test_all_10_audit_rules_isolated() {
    // Base default is fully hardened (score 100)
    let base = EmcPolicyInput::default();
    let base_rep = run_emc_posture_audit(&base);
    assert_eq!(base_rep.score, 100);

    // Rule 1: worst_case_margin_db < 6.0 (-25)
    let mut p1 = base.clone();
    p1.worst_case_margin_db = 5.5;
    let r1 = run_emc_posture_audit(&p1);
    assert_eq!(r1.total_deductions, 25);
    assert_eq!(r1.score, 75);

    // Rule 2: high_speed_traces_cross_split_plane (-25)
    let mut p2 = base.clone();
    p2.high_speed_traces_cross_split_plane = true;
    let r2 = run_emc_posture_audit(&p2);
    assert_eq!(r2.total_deductions, 25);
    assert_eq!(r2.score, 75);

    // Rule 3: tempest_crypto_shielding_enabled false (-25)
    let mut p3 = base.clone();
    p3.tempest_crypto_shielding_enabled = false;
    let r3 = run_emc_posture_audit(&p3);
    assert_eq!(r3.total_deductions, 25);
    assert_eq!(r3.score, 75);

    // Rule 4: external_io_cmc_filtered false (-10)
    let mut p4 = base.clone();
    p4.external_io_cmc_filtered = false;
    let r4 = run_emc_posture_audit(&p4);
    assert_eq!(r4.total_deductions, 10);
    assert_eq!(r4.score, 90);

    // Rule 5: max_enclosure_slot_length_mm > 25.0 (-10)
    let mut p5 = base.clone();
    p5.max_enclosure_slot_length_mm = 30.0;
    let r5 = run_emc_posture_audit(&p5);
    assert_eq!(r5.total_deductions, 10);
    assert_eq!(r5.score, 90);

    // Rule 6: ssc false and damping < 10 (-10)
    let mut p6 = base.clone();
    p6.ssc_enabled_on_system_clocks = false;
    p6.clock_series_damping_ohms = 5.0;
    let r6 = run_emc_posture_audit(&p6);
    assert_eq!(r6.total_deductions, 10);
    assert_eq!(r6.score, 90);

    // Rule 7: max_switching_loop_area_mm2 > 15.0 (-10)
    let mut p7 = base.clone();
    p7.max_switching_loop_area_mm2 = 20.0;
    let r7 = run_emc_posture_audit(&p7);
    assert_eq!(r7.total_deductions, 10);
    assert_eq!(r7.score, 90);

    // Rule 8: probe_antenna_factor_calibrated false (-4)
    let mut p8 = base.clone();
    p8.probe_antenna_factor_calibrated = false;
    let r8 = run_emc_posture_audit(&p8);
    assert_eq!(r8.total_deductions, 4);
    assert_eq!(r8.score, 96);

    // Rule 9: pdn_target_impedance_met false (-4)
    let mut p9 = base.clone();
    p9.pdn_target_impedance_met = false;
    let r9 = run_emc_posture_audit(&p9);
    assert_eq!(r9.total_deductions, 4);
    assert_eq!(r9.score, 96);

    // Rule 10: ambient_noise_baseline_subtracted false (-1)
    let mut p10 = base.clone();
    p10.ambient_noise_baseline_subtracted = false;
    let r10 = run_emc_posture_audit(&p10);
    assert_eq!(r10.total_deductions, 1);
    assert_eq!(r10.score, 99);
}

#[test]
fn test_sarif_and_json_exporters() {
    let p = EmcPolicyInput::default();
    let rep = run_emc_posture_audit(&p);

    let tbl = format_audit_table(&rep);
    assert!(tbl.contains("LOGIHERTZ EMC / EMI & TEMPEST POSTURE AUDIT REPORT"));
    assert!(tbl.contains("LH-EMC-001"));

    let json_str = format_audit_json(&rep);
    assert!(json_str.contains("\"score\": 100"));

    let sarif_str = format_audit_sarif(&rep);
    assert!(sarif_str.contains("\"version\": \"2.1.0\""));
    assert!(sarif_str.contains("lh-emc-fcc"));
}

#[test]
fn test_cli_subprocesses() {
    let bin_path = env!("CARGO_BIN_EXE_lh-emc-fcc");

    // 1. --help
    let help_out = Command::new(bin_path)
        .arg("--help")
        .output()
        .expect("exec --help");
    assert!(help_out.status.success());
    let help_str = String::from_utf8_lossy(&help_out.stdout);
    assert!(help_str.contains("Logihertz EMC & FCC Pre-Compliance Helper"));

    // 2. --version
    let ver_out = Command::new(bin_path)
        .arg("--version")
        .output()
        .expect("exec --version");
    assert!(ver_out.status.success());
    let ver_str = String::from_utf8_lossy(&ver_out.stdout);
    assert!(ver_str.contains("lh-emc-fcc 0.1.0"));

    // 3. scan hardened policy -> exit 0
    let scan_hard = Command::new(bin_path)
        .args(["scan", "fixtures/hardened_emc_policy.json"])
        .output()
        .expect("exec scan hardened");
    assert!(scan_hard.status.success());

    // 4. scan vulnerable policy -> exit 1
    let scan_vuln = Command::new(bin_path)
        .args(["scan", "fixtures/vulnerable_emc_policy.json"])
        .output()
        .expect("exec scan vulnerable");
    assert_eq!(scan_vuln.status.code(), Some(1));

    // 5. scan with --format json
    let scan_json = Command::new(bin_path)
        .args([
            "scan",
            "fixtures/hardened_emc_policy.json",
            "--format",
            "json",
        ])
        .output()
        .expect("exec scan json");
    assert!(scan_json.status.success());
    let sj_str = String::from_utf8_lossy(&scan_json.stdout);
    assert!(sj_str.contains("\"score\": 100"));

    // 6. scan with --format sarif
    let scan_sarif = Command::new(bin_path)
        .args([
            "scan",
            "fixtures/hardened_emc_policy.json",
            "--format",
            "sarif",
        ])
        .output()
        .expect("exec scan sarif");
    assert!(scan_sarif.status.success());
    let ss_str = String::from_utf8_lossy(&scan_sarif.stdout);
    assert!(ss_str.contains("2.1.0"));

    // 7. extrapolate-scan sample_nearfield_hscan.json -> exit 0
    let ext_scan = Command::new(bin_path)
        .args(["extrapolate-scan", "fixtures/sample_nearfield_hscan.json"])
        .output()
        .expect("exec extrapolate-scan");
    assert!(ext_scan.status.success());

    // 8. clock-harmonics
    let harm = Command::new(bin_path)
        .args([
            "clock-harmonics",
            "--freq-mhz",
            "50.0",
            "--rise-ns",
            "1.5",
            "--vdd",
            "3.3",
            "--duty",
            "0.5",
            "--ssc-pct",
            "0.5",
        ])
        .output()
        .expect("exec clock-harmonics");
    assert!(harm.status.success());

    // 8b. clock-harmonics with --strict and unshielded clock fails
    let harm_strict = Command::new(bin_path)
        .args(["clock-harmonics", "--freq-mhz", "50.0", "--strict"])
        .output()
        .expect("exec clock-harmonics strict");
    assert_eq!(harm_strict.status.code(), Some(1));

    // 8c. clock-harmonics with --strict and 40dB enclosure shielding passes
    let harm_shielded = Command::new(bin_path)
        .args([
            "clock-harmonics",
            "--freq-mhz",
            "50.0",
            "--enclosure-db",
            "40.0",
            "--strict",
        ])
        .output()
        .expect("exec clock-harmonics shielded");
    assert!(harm_shielded.status.success());

    // 9. check-limit Pass
    let cl_pass = Command::new(bin_path)
        .args(["check-limit", "--freq-mhz", "100.0", "--field-dbuv", "30.0"])
        .output()
        .expect("exec check-limit pass");
    assert!(cl_pass.status.success());

    // 10. check-limit Fail (field 55 dBuV/m > limit 43.5 dBuV/m at 100 MHz FCC-B)
    let cl_fail = Command::new(bin_path)
        .args(["check-limit", "--freq-mhz", "100.0", "--field-dbuv", "55.0"])
        .output()
        .expect("exec check-limit fail");
    assert_eq!(cl_fail.status.code(), Some(1));
}
