use super::*;
use crate::data::contracts::ContractModifier;
use crate::economy::upgrades::GameUpgrades;

/// A deterministic GameState for payout maths: no meta upgrades, no contract.
fn clean_state() -> GameState {
    let mut s = GameState::new();
    s.upgrades = GameUpgrades::new();
    s.active_contract = None;
    s.threat_signature = 0;
    s
}

#[test]
fn payout_increases_with_scrap() {
    let mut s = clean_state();
    s.resources.scrap = 0;
    let low = s.calculate_payout().total;
    s.resources.scrap = 200;
    let high = s.calculate_payout().total;
    assert!(high > low, "more hoarded scrap should pay more");
}

#[test]
fn risk_bonus_is_uncapped() {
    let mut s = clean_state();
    s.threat_signature = 100; // 100 * 12 = 1200, far above the old 240 cap
    let payout = s.calculate_payout();
    assert!(
        payout.risk_bonus >= 1200,
        "risk bonus must scale with danger, uncapped (got {})",
        payout.risk_bonus
    );
}

#[test]
fn higher_signature_pays_more() {
    let mut s = clean_state();
    s.threat_signature = 5;
    let quiet = s.calculate_payout().total;
    s.threat_signature = 40;
    let loud = s.calculate_payout().total;
    assert!(loud > quiet, "a louder run should pay more");
}

#[test]
fn signal_tier_thresholds() {
    let mut s = clean_state();
    s.threat_signature = WAVE_GRACE_POWER - 1;
    assert_eq!(s.signal_tier(), 0);
    s.threat_signature = WAVE_GRACE_POWER;
    assert_eq!(s.signal_tier(), 1);
    s.threat_signature = WAVE_T1_POWER;
    assert_eq!(s.signal_tier(), 2);
    s.threat_signature = WAVE_T2_POWER;
    assert_eq!(s.signal_tier(), 3);
    s.threat_signature = WAVE_T3_POWER;
    assert_eq!(s.signal_tier(), 4);
}

#[test]
fn contract_multiplier_boosts_payout() {
    let mut s = clean_state();
    let base = s.calculate_payout().total;
    s.active_contract = Some(ContractModifier {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: String::new(),
        signature_delta: 0,
        scrap_multiplier: 1.0,
        payout_multiplier: 2.0,
        start_scrap: 0,
    });
    let boosted = s.calculate_payout().total;
    assert!(
        boosted > base,
        "a payout-multiplier contract should pay more"
    );
}

#[test]
fn failure_payout_banks_less_than_escape() {
    let mut s = clean_state();
    s.resources.scrap = 100;
    let escape = s.calculate_payout().total;
    let failure = s.calculate_failure_payout().total;
    assert!(
        failure < escape,
        "a failed run's recovery value must be far below a clean escape"
    );
}
