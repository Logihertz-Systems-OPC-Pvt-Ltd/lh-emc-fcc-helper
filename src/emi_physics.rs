//! Electromagnetic Compatibility (EMC) & Radiated Emission Physics Engine.
//!
//! Implements analytical Maxwell and dipole radiation formulas, near-field magnetic
//! probe conversions, small-loop Differential-Mode (DM) and short-wire Common-Mode (CM)
//! far-field extrapolations, aperture slot shielding attenuation, and Fourier trapezoidal
//! clock harmonic envelopes with Spread-Spectrum Clocking (SSC) peak reduction.

#![forbid(unsafe_code)]

use std::f64::consts::PI;

/// Speed of light in free space (m/s).
pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

/// Intrinsic characteristic impedance of free space (Ohms, approximately 120 * pi).
pub const ETA_0_OHMS: f64 = 376.730_313_668;

/// Ground plane reflection factor (+6 dB voltage doubling over semi-anechoic ground plane).
pub const GROUND_REFLECTION_FACTOR: f64 = 2.0;

/// Standard Quasi-Peak receiver resolution bandwidth for CISPR Band B (30 MHz - 1 GHz) in Hz.
pub const DEFAULT_RBW_HZ: f64 = 120_000.0;

/// Converts a 50-Ohm spectrum analyzer power reading (dBm) to receiver voltage (dBµV).
///
/// Under 50-Ohm matched impedance:
///   0 dBm corresponds to 107.0 dBµV.
///   V_dBµV = P_SA_dBm + 107.0 + CableLoss_dB - PreampGain_dB.
#[inline]
pub fn receiver_voltage_dbuv(psa_dbm: f64, cable_loss_db: f64, preamp_gain_db: f64) -> f64 {
    psa_dbm + 107.0 + cable_loss_db - preamp_gain_db
}

/// Converts receiver voltage (dBµV) to magnetic field strength (dBµA/m) using the
/// calibrated magnetic antenna factor (AF_H in dB(Ω⁻¹/m) or dB(S/m)).
///
///   H_dBµA/m = V_dBµV + AF_H.
#[inline]
pub fn magnetic_field_dbuam(v_dbuv: f64, antenna_factor_h_db: f64) -> f64 {
    v_dbuv + antenna_factor_h_db
}

/// Converts magnetic field strength from logarithmic (dBµA/m) to linear units (A/m).
///
///   H_A/m = 10 ^ ((H_dBµA/m - 120.0) / 20.0)
#[inline]
pub fn magnetic_field_linear_am(h_dbuam: f64) -> f64 {
    10.0_f64.powf((h_dbuam - 120.0) / 20.0)
}

/// Calculates equivalent RF current (Amperes) flowing through a conductor or trace
/// enclosed by a calibrated magnetic near-field loop probe of radius `probe_radius_m`.
///
/// From Ampere's circuital law:
///   ∮ H · dl = I_eq  =>  I_eq = 2 * π * r_probe * H_near.
pub fn equivalent_rf_current(h_am: f64, probe_radius_m: f64) -> f64 {
    let r = if probe_radius_m <= 0.0 {
        0.005 // default 5 mm probe radius
    } else {
        probe_radius_m
    };
    2.0 * PI * r * h_am
}

/// Calculates the far-field Differential-Mode (DM) radiated electric field (V/m)
/// from an electrically small printed circuit board loop carrying RF current.
///
/// Mathematical Model:
///   E_DM(f, d) = G_gnd * (eta_0 * π * A_loop * I_DM * f²) / (c² * d)
///
/// Parameters:
/// - `freq_hz`: Frequency in Hertz.
/// - `distance_m`: Measurement distance in meters (e.g. 3.0 m or 10.0 m).
/// - `loop_area_m2`: Loop area in square meters.
/// - `current_dm_a`: Differential-mode RF loop current in Amperes.
pub fn diff_mode_radiated_field_vm(
    freq_hz: f64,
    distance_m: f64,
    loop_area_m2: f64,
    current_dm_a: f64,
) -> f64 {
    if freq_hz <= 0.0 || distance_m <= 0.0 || loop_area_m2 <= 0.0 || current_dm_a <= 0.0 {
        return 0.0;
    }
    let numerator = GROUND_REFLECTION_FACTOR
        * ETA_0_OHMS
        * PI
        * loop_area_m2
        * current_dm_a
        * freq_hz
        * freq_hz;
    let denominator = SPEED_OF_LIGHT_M_S * SPEED_OF_LIGHT_M_S * distance_m;
    numerator / denominator
}

/// Calculates the far-field Common-Mode (CM) radiated electric field (V/m)
/// from an attached peripheral cable or chassis structure acting as a monopole/dipole.
///
/// Mathematical Model (short-wire / quarter-wave monopole approximation):
///   E_CM(f, d) = G_gnd * (eta_0 * L_cm * I_CM * f) / (2 * c * d)
///
/// Parameters:
/// - `freq_hz`: Frequency in Hertz.
/// - `distance_m`: Measurement distance in meters.
/// - `cable_length_m`: Common-mode radiating cable length in meters.
/// - `current_cm_a`: Common-mode RF current in Amperes.
pub fn common_mode_radiated_field_vm(
    freq_hz: f64,
    distance_m: f64,
    cable_length_m: f64,
    current_cm_a: f64,
) -> f64 {
    if freq_hz <= 0.0 || distance_m <= 0.0 || cable_length_m <= 0.0 || current_cm_a <= 0.0 {
        return 0.0;
    }
    let numerator = GROUND_REFLECTION_FACTOR * ETA_0_OHMS * cable_length_m * current_cm_a * freq_hz;
    let denominator = 2.0 * SPEED_OF_LIGHT_M_S * distance_m;
    numerator / denominator
}

/// Combines Differential-Mode and Common-Mode radiated fields in quadrature (V/m).
///
///   E_total = sqrt(E_DM² + E_CM²)
#[inline]
pub fn total_electric_field_vm(e_dm_vm: f64, e_cm_vm: f64) -> f64 {
    (e_dm_vm * e_dm_vm + e_cm_vm * e_cm_vm).sqrt()
}

/// Converts electric field from linear V/m to logarithmic dBµV/m.
///
///   E_dBµV/m = 20 * log10(E_V/m * 10⁶)
pub fn electric_field_dbuvm(e_vm: f64) -> f64 {
    if e_vm <= 1e-15 {
        return 0.0;
    }
    let u_v_m = e_vm * 1_000_000.0;
    20.0 * u_v_m.log10()
}

/// Calculates the Shielding Effectiveness (SE in dB) of an enclosure aperture or seam slot.
///
/// For a slot of maximum dimension `slot_length_m` in a conductive enclosure:
///   λ = c / f
///   SE_slot(f) = 20 * log10(λ / (2 * l_slot))
///
/// When slot length <= 0.0, the enclosure is considered seam-welded / aperture-free (returns 100.0 dB).
/// At frequencies where λ < 2 * l_slot (slot enters resonant passband), SE degrades to 0.0 dB.
pub fn slot_shielding_effectiveness_db(freq_hz: f64, slot_length_m: f64) -> f64 {
    if slot_length_m <= 0.0 || freq_hz <= 0.0 {
        return 100.0;
    }
    let wavelength = SPEED_OF_LIGHT_M_S / freq_hz;
    let cutoff_dimension = 2.0 * slot_length_m;
    if cutoff_dimension <= 0.0 {
        return 100.0;
    }
    let ratio = wavelength / cutoff_dimension;
    if ratio <= 1.0 {
        0.0
    } else {
        20.0 * ratio.log10()
    }
}

/// Computes the unshielded and shielded net radiated electric field (dBµV/m) at a given distance.
#[allow(clippy::too_many_arguments)]
pub fn calculate_radiated_emission(
    freq_hz: f64,
    distance_m: f64,
    loop_area_m2: f64,
    current_dm_a: f64,
    cable_length_m: f64,
    current_cm_a: f64,
    enclosure_shielding_db: f64,
    max_slot_length_m: f64,
) -> (f64, f64, f64) {
    let e_dm_vm = diff_mode_radiated_field_vm(freq_hz, distance_m, loop_area_m2, current_dm_a);
    let e_cm_vm = common_mode_radiated_field_vm(freq_hz, distance_m, cable_length_m, current_cm_a);
    let e_tot_vm = total_electric_field_vm(e_dm_vm, e_cm_vm);
    let e_unshielded_dbuvm = electric_field_dbuvm(e_tot_vm);

    let se_slot_db = slot_shielding_effectiveness_db(freq_hz, max_slot_length_m);
    let se_effective_db = enclosure_shielding_db.min(se_slot_db).max(0.0);

    let e_shielded_dbuvm = (e_unshielded_dbuvm - se_effective_db).max(0.0);
    (e_unshielded_dbuvm, se_effective_db, e_shielded_dbuvm)
}

/// Normalized sinc function: sinc(x) = sin(π * x) / (π * x), with sinc(0) = 1.0.
#[inline]
fn sinc_pi(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// Computes the Fourier series envelope amplitude (Volts peak) of harmonic `n` for a
/// trapezoidal periodic clock waveform with supply voltage `vdd`, duty cycle `duty`,
/// rise/fall time `tr_s`, and fundamental frequency `f0_hz`.
///
/// Formula:
///   V(n) = 2 * V_dd * D * |sinc(n * D)| * |sinc(n * t_r / T)|
///   where T = 1 / f_0.
pub fn trapezoidal_harmonic_voltage(
    harmonic_n: u32,
    vdd: f64,
    duty_cycle: f64,
    tr_s: f64,
    f0_hz: f64,
) -> f64 {
    if harmonic_n == 0 || vdd <= 0.0 || f0_hz <= 0.0 {
        return 0.0;
    }
    let d = duty_cycle.clamp(0.01, 0.99);
    let t_period = 1.0 / f0_hz;
    let n_f = harmonic_n as f64;

    let duty_term = sinc_pi(n_f * d).abs();
    let rise_term = sinc_pi(n_f * tr_s / t_period).abs();

    2.0 * vdd * d * duty_term * rise_term
}

/// Calculates the peak emission reduction (attenuation in dB) achieved by Spread-Spectrum Clocking (SSC)
/// for harmonic `n` with nominal down-spread percentage `ssc_downspread_pct` (e.g. 0.5% or 1.0%)
/// measured under receiver resolution bandwidth `rbw_hz` (standard 120 kHz).
///
/// Mathematical Model:
///   ΔA_SSC(n) = 10 * log10(1 + (δ_ssc * n * f0) / RBW)
pub fn ssc_attenuation_db(
    harmonic_n: u32,
    f0_hz: f64,
    ssc_downspread_pct: f64,
    rbw_hz: f64,
) -> f64 {
    if harmonic_n == 0 || f0_hz <= 0.0 || ssc_downspread_pct <= 0.0 {
        return 0.0;
    }
    let rbw = if rbw_hz <= 0.0 {
        DEFAULT_RBW_HZ
    } else {
        rbw_hz
    };
    let delta_ssc = ssc_downspread_pct / 100.0;
    let spread_bandwidth = delta_ssc * (harmonic_n as f64) * f0_hz;
    if spread_bandwidth <= 0.0 {
        return 0.0;
    }
    let ratio = 1.0 + (spread_bandwidth / rbw);
    10.0 * ratio.log10()
}
