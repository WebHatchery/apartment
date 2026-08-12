use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use std::collections::HashMap;

const ASSET_PACK_PATH: &str = "assets.zip";

const JPEG_TEXTURE_IDS: &[&str] = &[
    "building_exterior",
    "design_cozy",
    "hallway",
    "neighborhood_downtown",
    "neighborhood_historic",
    "neighborhood_industrial",
    "neighborhood_suburbs",
    "tenant_artist",
    "tenant_elderly",
    "tenant_family",
    "tenant_professional",
    "tenant_student",
    "title_background",
];

pub struct AssetManager {
    pub textures: HashMap<String, Texture2D>,
    pub loaded: bool,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            loaded: false,
        }
    }

    pub async fn load_assets(&mut self) {
        let asset_pack = AssetPack::load(ASSET_PACK_PATH).await.ok();
        let asset_ids = vec![
            // Tenant Portraits
            "tenant_student",
            "tenant_professional",
            "tenant_artist",
            "tenant_family",
            "tenant_elderly",
            // Modular in-room tenant layers
            "tenant_body_poses",
            "tenant_face_emotions",
            // Designs
            "design_bare",
            "design_practical",
            "design_cozy",
            // Building Elements
            "building_exterior",
            "hallway",
            "apartment_door",
            "window_street",
            "window_quiet",
            // Neighborhoods
            "neighborhood_downtown",
            "neighborhood_suburbs",
            "neighborhood_industrial",
            "neighborhood_historic",
            // UI Icons
            "icon_money",
            "icon_repair",
            "icon_upgrade",
            "icon_soundproofing",
            "icon_noise",
            "icon_rent",
            "icon_application",
            "icon_key",
            "icon_condition_good",
            "icon_condition_poor",
            "icon_calendar",
            "icon_mail",
            "icon_inspection",
            "icon_market",
            // Happiness
            "happiness_ecstatic",
            "happiness_happy",
            "happiness_neutral",
            "happiness_unhappy",
            "happiness_miserable",
            // Events
            "event_rent_collected",
            "event_tenant_moved_in",
            "event_tenant_moved_out",
            "event_noise_complaint",
            "event_pipe_burst",
            "event_inspection",
            "event_heatwave",
            "event_new_business",
            "event_developer_offer",
            // Title & Menu
            "title_background",
            "title_logo",
            "menu_button_bg",
            // Decor
            "decoration_plant",
            "decoration_lamp",
            "decoration_books",
            "decoration_coffee",
        ];

        for id in asset_ids {
            // Large painterly art ships as JPEG to keep the web package small; the small
            // pixel-art icons stay lossless PNG, where JPEG ringing would be visible.
            let mut loaded = false;
            let extensions = if JPEG_TEXTURE_IDS.contains(&id) {
                ["jpg", "png"]
            } else {
                ["png", "jpg"]
            };
            for extension in extensions {
                let path = format!("assets/textures/{}.{}", id, extension);
                let filter = if JPEG_TEXTURE_IDS.contains(&id)
                    || matches!(id, "tenant_body_poses" | "tenant_face_emotions")
                {
                    FilterMode::Linear
                } else {
                    FilterMode::Nearest
                };
                if let Ok(texture) =
                    load_texture_from_pack_or_file(asset_pack.as_ref(), &path, filter).await
                {
                    self.textures.insert(id.to_string(), texture);
                    loaded = true;
                    break;
                }
            }

            if !loaded {
                // Silently skip missing textures - game uses fallback rendering
                #[cfg(not(target_arch = "wasm32"))]
                println!("Texture not found: assets/textures/{}", id);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        println!("Assets loaded: {} textures", self.textures.len());
        self.loaded = true;
    }

    /// Get a texture by ID. Returns None if not found.
    pub fn get_texture(&self, id: &str) -> Option<&Texture2D> {
        self.textures.get(id)
    }
}
