//! Tenant background generation from the `tenant_backgrounds.json` pools.

use std::collections::HashMap;
use std::sync::OnceLock;

use macroquad_toolkit::data_loader::{load_json_file_with_fallback_sync, JsonFallbackPolicy};
use macroquad_toolkit::rng;
use serde::Deserialize;

use super::TenantStory;
use crate::tenant::TenantArchetype;

/// Flavour pools; per-archetype maps are keyed by the lowercase archetype id.
#[derive(Debug, Default, Deserialize)]
struct BackgroundPools {
    job_titles: HashMap<String, Vec<String>>,
    hometowns: Vec<String>,
    move_reasons: HashMap<String, Vec<String>>,
    hobbies: HashMap<String, Vec<String>>,
    personality_traits: Vec<String>,
}

const ARCHETYPES: [TenantArchetype; 5] = [
    TenantArchetype::Student,
    TenantArchetype::Professional,
    TenantArchetype::Artist,
    TenantArchetype::Family,
    TenantArchetype::Elderly,
];

impl BackgroundPools {
    /// Every archetype needs a job, a reason to move and hobbies, or new
    /// residents arrive with placeholder backgrounds.
    fn validate(&self) -> Result<(), String> {
        if self.hometowns.is_empty() || self.personality_traits.is_empty() {
            return Err("hometowns and personality_traits must not be empty".to_string());
        }
        for archetype in &ARCHETYPES {
            let id = archetype_key(archetype);
            for (pool, name) in [
                (&self.job_titles, "job_titles"),
                (&self.move_reasons, "move_reasons"),
                (&self.hobbies, "hobbies"),
            ] {
                if pool.get(&id).is_none_or(Vec::is_empty) {
                    return Err(format!("{name} has no entries for '{id}'"));
                }
            }
        }
        Ok(())
    }
}

fn archetype_key(archetype: &TenantArchetype) -> String {
    archetype.name().to_lowercase()
}

fn pools() -> &'static BackgroundPools {
    static POOLS: OnceLock<BackgroundPools> = OnceLock::new();
    POOLS.get_or_init(|| {
        let loaded = load_json_file_with_fallback_sync::<BackgroundPools>(
            "assets/tenant_backgrounds.json",
            macroquad_toolkit::include_json_str!("../../../assets/tenant_backgrounds.json"),
            JsonFallbackPolicy::ReadError,
        )
        .map_err(|error| error.to_string())
        .and_then(|pools| pools.validate().map(|()| pools));
        loaded.unwrap_or_else(|error| {
            eprintln!("Invalid tenant_backgrounds.json: {error}");
            BackgroundPools::default()
        })
    })
}

fn pick(pool: Option<&Vec<String>>, fallback: &str) -> String {
    pool.and_then(|values| rng::choose(values).cloned())
        .unwrap_or_else(|| fallback.to_string())
}

pub(super) fn generate(tenant_id: u32, archetype: &TenantArchetype) -> TenantStory {
    let pools = pools();
    let key = archetype_key(archetype);

    let job_title = pick(pools.job_titles.get(&key), "Worker");
    let hometown = pick(Some(&pools.hometowns), "Unknown");
    let move_reason = pick(pools.move_reasons.get(&key), "Needed a change");

    let hobby_pool = pools.hobbies.get(&key).cloned().unwrap_or_default();
    let num_hobbies = rng::gen_range(1, 3);
    let hobbies = rng::choose_multiple(&hobby_pool, num_hobbies)
        .into_iter()
        .cloned()
        .collect();

    let num_traits = rng::gen_range(1, 3);
    let personality_traits = rng::choose_multiple(&pools.personality_traits, num_traits)
        .into_iter()
        .cloned()
        .collect();

    let (has_partner, has_children, num_children) = match archetype {
        TenantArchetype::Student => (false, false, 0),
        TenantArchetype::Professional => (rng::gen_range(0, 100) < 30, false, 0),
        TenantArchetype::Artist => (rng::gen_range(0, 100) < 20, false, 0),
        TenantArchetype::Family => (true, true, rng::gen_range(1, 4)),
        TenantArchetype::Elderly => (rng::gen_range(0, 100) < 50, rng::gen_range(0, 100) < 70, 0),
    };

    TenantStory {
        tenant_id,
        job_title,
        hometown,
        move_reason,
        hobbies,
        personality_traits,
        has_partner,
        has_children,
        num_children,
        story_events: Vec::new(),
        pending_request: None,
    }
}
