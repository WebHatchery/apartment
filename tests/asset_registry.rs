use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const RUNTIME_TEXTURES: [&str; 66] = [
    "assets/textures/tenant_student.jpg",
    "assets/textures/tenant_professional.jpg",
    "assets/textures/tenant_artist.jpg",
    "assets/textures/tenant_family.jpg",
    "assets/textures/tenant_elderly.jpg",
    "assets/textures/tenant_body_poses.png",
    "assets/textures/tenant_body_poses_alt.png",
    "assets/textures/tenant_face_emotions.png",
    "assets/textures/tenant_face_emotions_alt.png",
    "assets/textures/npc_uncle_artie.png",
    "assets/textures/staff_portraits.png",
    "assets/textures/achievement_emblems.png",
    "assets/textures/design_bare.png",
    "assets/textures/design_practical.png",
    "assets/textures/design_cozy.jpg",
    "assets/textures/building_exterior.jpg",
    "assets/textures/property_facades.png",
    "assets/textures/hallway.jpg",
    "assets/textures/apartment_door.png",
    "assets/textures/window_street.png",
    "assets/textures/window_quiet.png",
    "assets/textures/neighborhood_downtown.jpg",
    "assets/textures/neighborhood_suburbs.jpg",
    "assets/textures/neighborhood_industrial.jpg",
    "assets/textures/neighborhood_historic.jpg",
    "assets/textures/workspace_icons.png",
    "assets/textures/upgrade_icons.png",
    "assets/textures/status_icons.png",
    "assets/textures/mission_icons.png",
    "assets/textures/mail_icons.png",
    "assets/textures/icon_money.png",
    "assets/textures/icon_repair.png",
    "assets/textures/icon_upgrade.png",
    "assets/textures/icon_soundproofing.png",
    "assets/textures/icon_noise.png",
    "assets/textures/icon_rent.png",
    "assets/textures/icon_application.png",
    "assets/textures/icon_key.png",
    "assets/textures/icon_condition_good.png",
    "assets/textures/icon_condition_poor.png",
    "assets/textures/icon_calendar.png",
    "assets/textures/icon_mail.png",
    "assets/textures/icon_inspection.png",
    "assets/textures/icon_market.png",
    "assets/textures/happiness_ecstatic.png",
    "assets/textures/happiness_happy.png",
    "assets/textures/happiness_neutral.png",
    "assets/textures/happiness_unhappy.png",
    "assets/textures/happiness_miserable.png",
    "assets/textures/event_atlas.png",
    "assets/textures/event_rent_collected.png",
    "assets/textures/event_tenant_moved_in.png",
    "assets/textures/event_tenant_moved_out.png",
    "assets/textures/event_noise_complaint.png",
    "assets/textures/event_pipe_burst.png",
    "assets/textures/event_inspection.png",
    "assets/textures/event_heatwave.png",
    "assets/textures/event_new_business.png",
    "assets/textures/event_developer_offer.png",
    "assets/textures/title_background.jpg",
    "assets/textures/title_logo.png",
    "assets/textures/menu_button_bg.png",
    "assets/textures/decoration_plant.png",
    "assets/textures/decoration_lamp.png",
    "assets/textures/decoration_books.png",
    "assets/textures/decoration_coffee.png",
];

#[test]
fn asset_registry_matches_external_runtime_textures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(root.join("asset_registry.json"))
        .expect("asset_registry.json must be readable");
    let registry: Value = serde_json::from_str(&json).expect("asset registry must be valid JSON");
    assert_eq!(registry["version"], 1);

    let registered: BTreeSet<&str> = registry["assets"]
        .as_array()
        .expect("asset registry needs an assets array")
        .iter()
        .map(|entry| entry.as_str().expect("asset paths must be strings"))
        .collect();
    let expected: BTreeSet<&str> = RUNTIME_TEXTURES.into_iter().collect();
    assert_eq!(registered, expected);

    for relative in registered {
        assert!(
            root.join(relative).is_file(),
            "registered runtime asset is missing: {relative}"
        );
    }
}
