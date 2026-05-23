use crate::encoding::{SubLevel, TopLevel};
use crate::model::force::{ForceLaw, ForceTerm, MAX_FORCE_TERMS};
use crate::simulation::SimulationConfig;

use super::applied::AppliedUrlState;

const URL_STATE_VERSION: u32 = 1;

pub fn encode_applied_state(state: &AppliedUrlState) -> Result<String, String> {
    let state = state.clone().clamped();
    validate_applied_state(&state)?;

    let mut pairs: Vec<(String, SubLevel)> = Vec::new();
    pairs.push(("v".into(), SubLevel::new(URL_STATE_VERSION.to_string())));
    pairs.push((
        "soft".into(),
        SubLevel::encode_from_f32_vec(&[state.physics.softening]),
    ));
    pairs.push((
        "merge".into(),
        SubLevel::encode_from_f32_vec(&[state.physics.merge_radius_factor]),
    ));
    pairs.push((
        "ts".into(),
        SubLevel::encode_from_f32_vec(&[state.time_scale]),
    ));
    pairs.push((
        "pause".into(),
        SubLevel::new(if state.paused { "1" } else { "0" }),
    ));
    pairs.push(("seed".into(), SubLevel::new(state.initial.seed.to_string())));
    pairs.push((
        "nstars".into(),
        SubLevel::new(state.initial.n_stars.to_string()),
    ));
    pairs.push((
        "stmass".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.star_mass]),
    ));
    pairs.push((
        "ror".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.star_orbit_radius]),
    ));
    pairs.push((
        "dmmin".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.disk_mass_min]),
    ));
    pairs.push((
        "dmmax".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.disk_mass_max]),
    ));
    pairs.push((
        "drmin".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.disk_r_min]),
    ));
    pairs.push((
        "drmax".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.disk_r_max]),
    ));
    pairs.push((
        "dh".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.disk_height]),
    ));
    pairs.push((
        "vpert".into(),
        SubLevel::encode_from_f32_vec(&[state.initial.initial_v_perturbation]),
    ));
    pairs.push((
        "active".into(),
        SubLevel::new(state.initial.active_count.to_string()),
    ));

    for term in &state.force.terms[..state.force.term_count as usize] {
        pairs.push((
            "term".into(),
            SubLevel::new(format!(
                "s:{},e:{},c:{}",
                term.sign,
                term.exponent,
                SubLevel::encode_from_f32_vec(&[term.coefficient]).as_str()
            )),
        ));
    }

    Ok(TopLevel(pairs).encode())
}

pub fn decode_applied_state(query: &str) -> Result<AppliedUrlState, String> {
    let top = TopLevel::decode(query)?;

    let mut version = None;
    let mut physics = AppliedUrlState::default().physics;
    let mut initial = AppliedUrlState::default().initial;
    let mut time_scale = None;
    let mut paused = None;
    let mut legacy_g = None;
    let mut terms = Vec::<ForceTerm>::new();

    for (key, val) in top.pairs() {
        match key.as_str() {
            "v" => version = Some(val.decode_to_u32()?),
            "g" => legacy_g = Some(decode_f32(val)?),
            "soft" => physics.softening = decode_f32(val)?,
            "merge" => physics.merge_radius_factor = decode_f32(val)?,
            "ts" => time_scale = Some(decode_f32(val)?),
            "pause" => paused = Some(decode_pause(val)?),
            "seed" => initial.seed = decode_u64(val)?,
            "nstars" => initial.n_stars = val.decode_to_u32()?,
            "stmass" => initial.star_mass = decode_f32(val)?,
            "ror" => initial.star_orbit_radius = decode_f32(val)?,
            "dmmin" => initial.disk_mass_min = decode_f32(val)?,
            "dmmax" => initial.disk_mass_max = decode_f32(val)?,
            "drmin" => initial.disk_r_min = decode_f32(val)?,
            "drmax" => initial.disk_r_max = decode_f32(val)?,
            "dh" => initial.disk_height = decode_f32(val)?,
            "vpert" => initial.initial_v_perturbation = decode_f32(val)?,
            "active" => initial.active_count = val.decode_to_u32()?,
            "term" => terms.push(parse_force_term(val)?),
            _ => {}
        }
    }

    let version = version.ok_or("missing v")?;
    if version != URL_STATE_VERSION {
        return Err(format!("unsupported url state version {version}"));
    }

    let mut force = force_from_terms(terms)?;
    if let Some(g) = legacy_g {
        force.set_gravity_coefficient(g);
    }

    let state = AppliedUrlState {
        physics,
        initial,
        force,
        time_scale: time_scale.unwrap_or(SimulationConfig::default().time_scale),
        paused: paused.unwrap_or(false),
    }
    .clamped();

    validate_applied_state(&state)?;
    Ok(state)
}

fn validate_applied_state(state: &AppliedUrlState) -> Result<(), String> {
    if !state.physics.softening.is_finite()
        || !state.physics.merge_radius_factor.is_finite()
    {
        return Err("non-finite physics parameter".into());
    }
    if !state.time_scale.is_finite() {
        return Err("non-finite time scale".into());
    }
    if !state.force.is_valid() {
        return Err("invalid force law".into());
    }
    Ok(())
}

fn decode_f32(val: &SubLevel) -> Result<f32, String> {
    let v = val.decode_to_f32_vec()?;
    if v.len() != 1 {
        return Err(format!("expected one f32, got {}", v.len()));
    }
    if !v[0].is_finite() {
        return Err("non-finite f32".into());
    }
    Ok(v[0])
}

fn decode_u64(val: &SubLevel) -> Result<u64, String> {
    val.as_str()
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("bad u64: {}", val.as_str()))
}

fn decode_pause(val: &SubLevel) -> Result<bool, String> {
    match val.as_str().trim() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => val.decode_to_bool_bin(),
    }
}

fn parse_force_term(val: &SubLevel) -> Result<ForceTerm, String> {
    let wire = val.decode_to_kv_pairs()?;
    Ok(ForceTerm {
        sign: wire.get_i8("s")?,
        exponent: wire.get_i32("e")?,
        coefficient: wire
            .get_f32("c")
            .map_err(|_| "missing or invalid term coefficient".to_string())?,
    })
}

fn force_from_terms(terms: Vec<ForceTerm>) -> Result<ForceLaw, String> {
    if terms.is_empty() {
        return Err("missing force terms".into());
    }
    if terms.len() > MAX_FORCE_TERMS {
        return Err("too many force terms".into());
    }
    let term_count = terms.len() as u8;
    let mut slots = [ForceTerm {
        sign: 0,
        exponent: 0,
        coefficient: 0.0,
    }; MAX_FORCE_TERMS];
    for (index, term) in terms.into_iter().enumerate() {
        slots[index] = term;
    }
    let force = ForceLaw {
        terms: slots,
        term_count,
    };
    if !force.is_valid() {
        return Err("invalid force law".into());
    }
    Ok(force)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::constants::G;
    use crate::model::{ForceLaw, InitialConditions};

    #[test]
    fn round_trip_defaultish() {
        let state = AppliedUrlState::default().clamped();
        let q = encode_applied_state(&state).unwrap();
        assert!(q.starts_with("v=1"), "body: {q}");
        let out = decode_applied_state(&q).unwrap();
        assert_eq!(encode_applied_state(&out).unwrap(), q);
    }

    #[test]
    fn round_trip_custom_force_and_initial() {
        let state = AppliedUrlState {
            force: ForceLaw::preset_gravity_plus_repulsion(G),
            initial: InitialConditions {
                seed: 42,
                n_stars: 3,
                active_count: 500,
                ..InitialConditions::default()
            },
            paused: true,
            time_scale: 2.0,
            ..AppliedUrlState::default()
        }
        .clamped();
        let q = encode_applied_state(&state).unwrap();
        let out = decode_applied_state(&q).unwrap();
        assert_eq!(encode_applied_state(&out).unwrap(), q);
        assert_eq!(out.initial.seed, 42);
        assert_eq!(out.force.term_count, 2);
        assert!(out.paused);
    }

    #[test]
    fn decode_ignores_unknown_keys() {
        let state = AppliedUrlState::default().clamped();
        let base = encode_applied_state(&state).unwrap();
        let q = format!("{base}&future_key=abc");
        let out = decode_applied_state(&q).unwrap();
        assert_eq!(encode_applied_state(&out).unwrap(), base);
    }

    #[test]
    fn rejects_unsupported_version() {
        let state = AppliedUrlState::default();
        let mut q = encode_applied_state(&state).unwrap();
        q = q.replacen("v=1", "v=2", 1);
        assert!(decode_applied_state(&q).is_err());
    }
}
