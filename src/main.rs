//! Command-line interface for `lh-emc-fcc`.
//!
//! Provides commands for:
//! - `scan`: Auditing a PCB/enclosure policy against the 10 EMC/TEMPEST rules.
//! - `extrapolate-scan`: Converting near-field spectrum analyzer scan telemetry into far-field radiated emissions.
//! - `clock-harmonics`: Calculating Fourier clock harmonic emissions and SSC peak reductions.
//! - `check-limit`: Querying regulatory limit lines (FCC Part 15B / CISPR 32) at specific frequencies.

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;

use lh_emc_fcc::auditor::{run_emc_posture_audit, EmcPolicyInput};
use lh_emc_fcc::emi_physics::{
    calculate_radiated_emission, equivalent_rf_current, magnetic_field_dbuam,
    magnetic_field_linear_am, receiver_voltage_dbuv, ssc_attenuation_db,
    trapezoidal_harmonic_voltage,
};
use lh_emc_fcc::exporters::{
    format_audit_json, format_audit_sarif, format_audit_table, format_harmonics_json,
    format_harmonics_table, format_scan_json, format_scan_table, ClockHarmonicPoint,
    ExtrapolatedPoint,
};
use lh_emc_fcc::fcc_cispr_limits::{
    evaluate_emission_margin, get_regulatory_limit_dbuvm, ComplianceStatus, EmcStandard,
};
use lh_emc_fcc::json::{JsonParser, JsonValue};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!(
        "Logihertz EMC & FCC Pre-Compliance Helper (lh-emc-fcc) v{}",
        VERSION
    );
    println!("Hardware EMC Pre-Compliance, Maxwell Extrapolator & 10-Rule TEMPEST Auditor\n");
    println!("USAGE:");
    println!("    lh-emc-fcc <SUBCOMMAND> [OPTIONS]\n");
    println!("SUBCOMMANDS:");
    println!("    scan <policy.json> [--format table|json|sarif]");
    println!("        Audit PCB layout and hardware EMC posture against 10 rules.\n");
    println!("    extrapolate-scan <scan.json> [--standard fcc-b|fcc-a|cispr-b|cispr-a] [--distance 3|10] [--format table|json]");
    println!("        Extrapolate near-field H-probe spectrum analyzer data to far-field radiated emissions.\n");
    println!("    clock-harmonics --freq-mhz <f0> [OPTIONS]");
    println!(
        "        Model trapezoidal clock Fourier harmonics, SSC reduction, and regulatory margins."
    );
    println!("        Options:");
    println!("            --freq-mhz <f0>        Clock fundamental frequency (MHz, required)");
    println!("            --rise-ns <tr_ns>      Signal 10%-90% rise time in ns (default: 1.5)");
    println!("            --vdd <v>              Supply voltage in Volts (default: 3.3)");
    println!("            --duty <d>             Duty cycle 0.01 - 0.99 (default: 0.50)");
    println!(
        "            --ssc-pct <pct>        Spread-Spectrum Clocking down-spread % (default: 0.5)"
    );
    println!("            --damping-ohms <r>     Series source damping resistance in Ohms (default: 0.0)");
    println!(
        "            --enclosure-db <db>    Enclosure shielding attenuation in dB (default: 0.0)"
    );
    println!(
        "            --standard <std>       fcc-b | fcc-a | cispr-b | cispr-a (default: fcc-b)"
    );
    println!("            --distance <3|10>      Measurement distance in meters (default: 3)");
    println!("            --format <table|json>  Output display format (default: table)");
    println!("            --strict               Exit with code 1 if any harmonic exceeds limit\n");
    println!("    check-limit --freq-mhz <f> --field-dbuv <val> [OPTIONS]");
    println!("        Quick check of an emission field level against regulatory limit line.");
    println!("        Options:");
    println!("            --freq-mhz <f>         Frequency in MHz (required)");
    println!("            --field-dbuv <val>     Electric field in dBµV/m (required)");
    println!(
        "            --standard <std>       fcc-b | fcc-a | cispr-b | cispr-a (default: fcc-b)"
    );
    println!("            --distance <3|10>      Measurement distance in meters (default: 3)");
    println!("            --format <table|json>  Output display format (default: table)\n");
    println!("FLAGS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_usage();
            process::exit(0);
        }
        "-V" | "--version" => {
            println!("lh-emc-fcc {}", VERSION);
            process::exit(0);
        }
        "scan" => {
            handle_scan(&args[2..]);
        }
        "extrapolate-scan" => {
            handle_extrapolate_scan(&args[2..]);
        }
        "clock-harmonics" => {
            handle_clock_harmonics(&args[2..]);
        }
        "check-limit" => {
            handle_check_limit(&args[2..]);
        }
        other => {
            eprintln!("Error: Unknown subcommand '{}'\n", other);
            print_usage();
            process::exit(1);
        }
    }
}

fn handle_scan(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: 'scan' requires path to policy JSON file.");
        eprintln!("Usage: lh-emc-fcc scan <policy.json> [--format table|json|sarif]");
        process::exit(1);
    }

    let policy_path = &args[0];
    let mut format = "table";

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            format = args[i + 1].as_str();
            i += 2;
        } else {
            i += 1;
        }
    }

    let content = match fs::read_to_string(policy_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading policy file '{}': {}", policy_path, e);
            process::exit(1);
        }
    };

    let json_val = match JsonParser::parse(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON from '{}': {}", policy_path, e);
            process::exit(1);
        }
    };

    let policy = match EmcPolicyInput::from_json(&json_val) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading policy properties: {}", e);
            process::exit(1);
        }
    };

    let report = run_emc_posture_audit(&policy);

    match format {
        "json" => println!("{}", format_audit_json(&report)),
        "sarif" => println!("{}", format_audit_sarif(&report)),
        _ => println!("{}", format_audit_table(&report)),
    }

    if !report.is_compliant {
        process::exit(1);
    } else {
        process::exit(0);
    }
}

fn handle_extrapolate_scan(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: 'extrapolate-scan' requires path to scan JSON file.");
        eprintln!("Usage: lh-emc-fcc extrapolate-scan <scan.json> [--standard fcc-b|fcc-a|cispr-b|cispr-a] [--distance 3|10] [--format table|json]");
        process::exit(1);
    }

    let scan_path = &args[0];
    let mut standard = EmcStandard::FccPart15BClassB;
    let mut distance_m = 3.0;
    let mut format = "table";

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--standard" if i + 1 < args.len() => {
                if let Some(s) = EmcStandard::parse(&args[i + 1]) {
                    standard = s;
                } else {
                    eprintln!(
                        "Warning: Unknown standard '{}', defaulting to fcc-b",
                        args[i + 1]
                    );
                }
                i += 2;
            }
            "--distance" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<f64>() {
                    distance_m = d;
                }
                i += 2;
            }
            "--format" if i + 1 < args.len() => {
                format = args[i + 1].as_str();
                i += 2;
            }
            _ => i += 1,
        }
    }

    let content = match fs::read_to_string(scan_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading scan file '{}': {}", scan_path, e);
            process::exit(1);
        }
    };

    let json_val = match JsonParser::parse(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing scan JSON: {}", e);
            process::exit(1);
        }
    };

    let points_arr = if let Some(arr) = json_val.as_array() {
        arr
    } else if let Some(arr) = json_val.get("points").and_then(|v| v.as_array()) {
        arr
    } else if let Some(arr) = json_val.get("measurements").and_then(|v| v.as_array()) {
        arr
    } else {
        eprintln!("Error: Scan JSON must contain an array of measurements or 'points' field.");
        process::exit(1);
    };

    let top_probe_af = json_val
        .get("probe_af_db")
        .and_then(|v| v.as_f64())
        .unwrap_or(20.0);
    let top_probe_radius_m = json_val
        .get("probe_radius_mm")
        .and_then(|v| v.as_f64())
        .map(|mm| mm / 1000.0)
        .unwrap_or(0.005);
    let top_cable_loss_db = json_val
        .get("cable_loss_db")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5);
    let top_preamp_gain_db = json_val
        .get("preamp_gain_db")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let top_enclosure_shielding_db = json_val
        .get("enclosure_shielding_db")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let top_slot_length_m = json_val
        .get("slot_length_mm")
        .and_then(|v| v.as_f64())
        .map(|mm| mm / 1000.0)
        .unwrap_or(0.0);

    let mut extrapolated_points = Vec::new();
    let mut has_failure = false;

    for item in points_arr {
        let freq_mhz = item
            .get("freq_mhz")
            .and_then(|v| v.as_f64())
            .or_else(|| {
                item.get("freq_hz")
                    .and_then(|v| v.as_f64())
                    .map(|h| h / 1e6)
            })
            .unwrap_or(100.0);
        let psa_dbm = item
            .get("psa_dbm")
            .and_then(|v| v.as_f64())
            .or_else(|| item.get("power_dbm").and_then(|v| v.as_f64()))
            .unwrap_or(-50.0);
        let cable_loss_db = item
            .get("cable_loss_db")
            .and_then(|v| v.as_f64())
            .unwrap_or(top_cable_loss_db);
        let preamp_gain_db = item
            .get("preamp_gain_db")
            .and_then(|v| v.as_f64())
            .unwrap_or(top_preamp_gain_db);
        let probe_af_db = item
            .get("probe_af_db")
            .and_then(|v| v.as_f64())
            .unwrap_or(top_probe_af);
        let probe_radius_m = item
            .get("probe_radius_mm")
            .and_then(|v| v.as_f64())
            .map(|mm| mm / 1000.0)
            .unwrap_or(top_probe_radius_m);
        let loop_area_m2 = item
            .get("loop_area_mm2")
            .and_then(|v| v.as_f64())
            .map(|mm2| mm2 * 1e-6)
            .unwrap_or(10.0 * 1e-6);
        let cable_length_m = item
            .get("cable_length_m")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);
        let enclosure_shielding_db = item
            .get("enclosure_shielding_db")
            .and_then(|v| v.as_f64())
            .unwrap_or(top_enclosure_shielding_db);
        let slot_length_m = item
            .get("slot_length_mm")
            .and_then(|v| v.as_f64())
            .map(|mm| mm / 1000.0)
            .unwrap_or(top_slot_length_m);

        let v_dbuv = receiver_voltage_dbuv(psa_dbm, cable_loss_db, preamp_gain_db);
        let h_dbuam = magnetic_field_dbuam(v_dbuv, probe_af_db);
        let h_linear_am = magnetic_field_linear_am(h_dbuam);
        let i_eq_a = equivalent_rf_current(h_linear_am, probe_radius_m);
        let i_dm_a = i_eq_a;
        let i_cm_a = 0.05 * i_eq_a; // standard 5% common-mode cable coupling ratio

        let freq_hz = freq_mhz * 1e6;
        let (unshielded_dbuvm, se_db, shielded_dbuvm) = calculate_radiated_emission(
            freq_hz,
            distance_m,
            loop_area_m2,
            i_dm_a,
            cable_length_m,
            i_cm_a,
            enclosure_shielding_db,
            slot_length_m,
        );

        let limit_dbuvm = get_regulatory_limit_dbuvm(standard, distance_m, freq_mhz);
        let (margin_db, status) = evaluate_emission_margin(shielded_dbuvm, limit_dbuvm, 6.0);
        if status == ComplianceStatus::FailOverLimit {
            has_failure = true;
        }

        extrapolated_points.push(ExtrapolatedPoint {
            freq_mhz,
            measured_psa_dbm: psa_dbm,
            receiver_voltage_dbuv: v_dbuv,
            magnetic_field_dbuam: h_dbuam,
            equivalent_current_ma: i_eq_a * 1000.0,
            unshielded_field_dbuvm: unshielded_dbuvm,
            shielding_eff_db: se_db,
            shielded_field_dbuvm: shielded_dbuvm,
            regulatory_limit_dbuvm: limit_dbuvm,
            margin_db,
            status,
        });
    }

    match format {
        "json" => println!(
            "{}",
            format_scan_json(&extrapolated_points, standard.code(), distance_m)
        ),
        _ => println!(
            "{}",
            format_scan_table(&extrapolated_points, standard.code(), distance_m)
        ),
    }

    if has_failure {
        process::exit(1);
    } else {
        process::exit(0);
    }
}

fn handle_clock_harmonics(args: &[String]) {
    let mut f0_mhz: Option<f64> = None;
    let mut rise_ns = 1.5;
    let mut vdd = 3.3;
    let mut duty = 0.5;
    let mut ssc_pct = 0.5;
    let mut damping_ohms = 0.0;
    let mut enclosure_db = 0.0;
    let mut standard = EmcStandard::FccPart15BClassB;
    let mut distance_m = 3.0;
    let mut format = "table";
    let mut strict = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--freq-mhz" if i + 1 < args.len() => {
                f0_mhz = args[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "--rise-ns" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    rise_ns = v;
                }
                i += 2;
            }
            "--vdd" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    vdd = v;
                }
                i += 2;
            }
            "--duty" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    duty = v;
                }
                i += 2;
            }
            "--ssc-pct" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    ssc_pct = v;
                }
                i += 2;
            }
            "--damping-ohms" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    damping_ohms = v;
                }
                i += 2;
            }
            "--enclosure-db" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<f64>() {
                    enclosure_db = v;
                }
                i += 2;
            }
            "--standard" if i + 1 < args.len() => {
                if let Some(s) = EmcStandard::parse(&args[i + 1]) {
                    standard = s;
                }
                i += 2;
            }
            "--distance" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<f64>() {
                    distance_m = d;
                }
                i += 2;
            }
            "--format" if i + 1 < args.len() => {
                format = args[i + 1].as_str();
                i += 2;
            }
            "--strict" => {
                strict = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let f0 = match f0_mhz {
        Some(f) if f > 0.0 => f,
        _ => {
            eprintln!("Error: --freq-mhz <f0> is required and must be > 0.");
            process::exit(1);
        }
    };

    let f0_hz = f0 * 1e6;
    let tr_s = rise_ns * 1e-9;
    let loop_area_m2 = 12.0 * 1e-6; // 12 mm² clock routing loop area
    let mut points = Vec::new();
    let mut has_failure = false;

    // Evaluate up to 15 harmonics or 1 GHz
    for n in 1..=15 {
        let f_harm_hz = (n as f64) * f0_hz;
        let f_harm_mhz = f_harm_hz / 1e6;
        if f_harm_mhz > 1000.0 {
            break;
        }

        let v_env = trapezoidal_harmonic_voltage(n, vdd, duty, tr_s, f0_hz);
        // Current into 50 Ohm trace impedance plus optional series damping resistor
        let z_total = 50.0 + damping_ohms.max(0.0);
        let i_harm_dm = v_env / z_total;
        let i_harm_cm = 0.03 * i_harm_dm; // 3% common-mode leakage

        let (unshielded_dbuvm, _, shielded_dbuvm) = calculate_radiated_emission(
            f_harm_hz,
            distance_m,
            loop_area_m2,
            i_harm_dm,
            0.3,
            i_harm_cm,
            enclosure_db,
            0.0,
        );

        let delta_ssc = ssc_attenuation_db(n, f0_hz, ssc_pct, 120_000.0);
        let net_field = (shielded_dbuvm - delta_ssc).max(0.0);

        let limit = get_regulatory_limit_dbuvm(standard, distance_m, f_harm_mhz);
        let (margin, status) = evaluate_emission_margin(net_field, limit, 6.0);
        if status == ComplianceStatus::FailOverLimit {
            has_failure = true;
        }

        points.push(ClockHarmonicPoint {
            harmonic_n: n,
            freq_mhz: f_harm_mhz,
            envelope_volts: v_env,
            base_field_dbuvm: unshielded_dbuvm,
            ssc_attenuation_db: delta_ssc,
            net_field_dbuvm: net_field,
            limit_dbuvm: limit,
            margin_db: margin,
            status,
        });
    }

    match format {
        "json" => println!(
            "{}",
            format_harmonics_json(&points, f0, vdd, rise_ns, ssc_pct)
        ),
        _ => println!(
            "{}",
            format_harmonics_table(&points, f0, vdd, rise_ns, ssc_pct)
        ),
    }

    if strict && has_failure {
        process::exit(1);
    } else {
        process::exit(0);
    }
}

fn handle_check_limit(args: &[String]) {
    let mut freq_mhz: Option<f64> = None;
    let mut field_dbuv: Option<f64> = None;
    let mut standard = EmcStandard::FccPart15BClassB;
    let mut distance_m = 3.0;
    let mut format = "table";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--freq-mhz" if i + 1 < args.len() => {
                freq_mhz = args[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "--field-dbuv" if i + 1 < args.len() => {
                field_dbuv = args[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "--standard" if i + 1 < args.len() => {
                if let Some(s) = EmcStandard::parse(&args[i + 1]) {
                    standard = s;
                }
                i += 2;
            }
            "--distance" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<f64>() {
                    distance_m = d;
                }
                i += 2;
            }
            "--format" if i + 1 < args.len() => {
                format = args[i + 1].as_str();
                i += 2;
            }
            _ => i += 1,
        }
    }

    let f = match freq_mhz {
        Some(val) if val > 0.0 => val,
        _ => {
            eprintln!("Error: --freq-mhz <f> is required.");
            process::exit(1);
        }
    };
    let field = match field_dbuv {
        Some(val) => val,
        _ => {
            eprintln!("Error: --field-dbuv <val> is required.");
            process::exit(1);
        }
    };

    let limit = get_regulatory_limit_dbuvm(standard, distance_m, f);
    let (margin, status) = evaluate_emission_margin(field, limit, 6.0);

    if format == "json" {
        let obj = JsonValue::Object(vec![
            ("freq_mhz".to_string(), JsonValue::Number(f)),
            ("field_dbuvm".to_string(), JsonValue::Number(field)),
            (
                "standard".to_string(),
                JsonValue::String(standard.code().to_string()),
            ),
            ("distance_m".to_string(), JsonValue::Number(distance_m)),
            ("limit_dbuvm".to_string(), JsonValue::Number(limit)),
            ("margin_db".to_string(), JsonValue::Number(margin)),
            (
                "status".to_string(),
                JsonValue::String(status.as_str().to_string()),
            ),
        ]);
        println!("{}", obj.to_pretty_string());
    } else {
        println!("============================================================");
        println!(
            " REGULATORY LIMIT VERIFICATION: {} ({:.1}m) ",
            standard.code(),
            distance_m
        );
        println!("============================================================");
        println!(" Frequency:           {:.2} MHz", f);
        println!(" Measured Field:      {:.2} dBµV/m", field);
        println!(" Regulatory Limit:    {:.2} dBµV/m", limit);
        println!(" Margin to Limit:     {:.2} dB", margin);
        let status_color = match status {
            ComplianceStatus::Pass => "\x1b[32mPASS (Margin >= 6.0 dB)\x1b[0m",
            ComplianceStatus::MarginalWarning => {
                "\x1b[33mMARGINAL_WARNING (0.0 <= Margin < 6.0 dB)\x1b[0m"
            }
            ComplianceStatus::FailOverLimit => "\x1b[31mFAIL_OVER_LIMIT (Over Limit)\x1b[0m",
        };
        println!(" Compliance Status:   {}", status_color);
        println!("============================================================");
    }

    if status == ComplianceStatus::FailOverLimit {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
