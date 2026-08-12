//! # Narrative Module
//!
//! Handles story-driven elements and dynamic events:
//! - `Stories`: Procedural tenant storylines.
//! - `Events`: One-off or chained narrative events.
//! - `Mail`: In-game messaging system.
//! - `Tutorial`: Guided introduction flow.
//! - `Missions`: Quests and objectives.
//! - `Notifications`: Game hints and relationship change pop-ups.

pub mod dialogue; // Make public so DialogueEffect is accessible
pub mod events;
mod mail;
pub(crate) mod missions;
pub mod notifications;
mod stories;
mod tutorial;

pub use dialogue::{ActiveDialogue, DialogueSystem};
pub use events::{NarrativeEvent, NarrativeEventSystem};
pub use mail::{MailItem, MailType, Mailbox};
pub use missions::{ActiveTaxBreak, MissionGoal, MissionManager, MissionReward, MissionStatus};
pub use notifications::{NotificationCategory, NotificationManager, RelationshipChange};
pub use stories::{LifeChangeType, StoryImpact, TenantRequest, TenantStory};
pub use tutorial::{TutorialManager, TutorialMilestone};
pub mod achievements;
pub use achievements::AchievementSystem;
pub mod events_config;
pub mod relationship_config;
pub use events_config::{load_events_config, TenantEventsConfig};
pub use relationship_config::{load_relationship_config, RelationshipEventsConfig};
