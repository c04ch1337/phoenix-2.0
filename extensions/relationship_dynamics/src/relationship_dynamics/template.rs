use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Persisted state keys (Soul Vault / encrypted).
pub const SOUL_KEY_RELATIONSHIP_TEMPLATE: &str = "relationship_dynamics:template";
pub const SOUL_KEY_RELATIONSHIP_INTIMACY_LEVEL: &str = "relationship_dynamics:intimacy_level";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntimacyLevel {
    Light,
    Deep,
    Eternal,
}

impl Default for IntimacyLevel {
    fn default() -> Self {
        Self::Light
    }
}

impl fmt::Display for IntimacyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Light => "Light",
            Self::Deep => "Deep",
            Self::Eternal => "Eternal",
        };
        write!(f, "{s}")
    }
}

impl FromStr for IntimacyLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Light" | "light" => Ok(Self::Light),
            "Deep" | "deep" => Ok(Self::Deep),
            "Eternal" | "eternal" => Ok(Self::Eternal),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InteractionWeights {
    pub affirmation: f32,
    pub support: f32,
    pub deep_talk: f32,
    pub play: f32,
    pub planning: f32,
    pub conflict_repair: f32,
}

impl InteractionWeights {
    pub fn normalized(self) -> Self {
        let sum = self.affirmation
            + self.support
            + self.deep_talk
            + self.play
            + self.planning
            + self.conflict_repair;
        if sum <= 0.0 {
            return Self {
                affirmation: 1.0,
                support: 1.0,
                deep_talk: 1.0,
                play: 1.0,
                planning: 1.0,
                conflict_repair: 1.0,
            };
        }
        Self {
            affirmation: self.affirmation / sum,
            support: self.support / sum,
            deep_talk: self.deep_talk / sum,
            play: self.play / sum,
            planning: self.planning / sum,
            conflict_repair: self.conflict_repair / sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipTemplate {
    CasualFriendship,
    SupportivePartnership,
    GrowthOrientedPartnership,
    IntimatePartnership { intimacy_level: IntimacyLevel },
}

impl Default for RelationshipTemplate {
    fn default() -> Self {
        Self::IntimatePartnership {
            intimacy_level: IntimacyLevel::Light,
        }
    }
}

impl fmt::Display for RelationshipTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CasualFriendship => write!(f, "CasualFriendship"),
            Self::SupportivePartnership => write!(f, "SupportivePartnership"),
            Self::GrowthOrientedPartnership => write!(f, "GrowthOrientedPartnership"),
            Self::IntimatePartnership { intimacy_level } => {
                write!(f, "IntimatePartnership({intimacy_level})")
            }
        }
    }
}

impl FromStr for RelationshipTemplate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "CasualFriendship" => Ok(Self::CasualFriendship),
            "SupportivePartnership" => Ok(Self::SupportivePartnership),
            "GrowthOrientedPartnership" => Ok(Self::GrowthOrientedPartnership),
            "IntimatePartnership" => Ok(Self::IntimatePartnership {
                intimacy_level: IntimacyLevel::default(),
            }),
            _ => Err(()),
        }
    }
}

impl RelationshipTemplate {
    pub fn template_name(&self) -> &'static str {
        match self {
            Self::CasualFriendship => "CasualFriendship",
            Self::SupportivePartnership => "SupportivePartnership",
            Self::GrowthOrientedPartnership => "GrowthOrientedPartnership",
            Self::IntimatePartnership { .. } => "IntimatePartnership",
        }
    }

    pub fn intimacy_level(&self) -> Option<IntimacyLevel> {
        match self {
            Self::IntimatePartnership { intimacy_level } => Some(*intimacy_level),
            _ => None,
        }
    }

    pub fn set_intimacy_level(&mut self, level: IntimacyLevel) {
        if let Self::IntimatePartnership { intimacy_level } = self {
            *intimacy_level = level;
        }
    }

    /// #1 RelationshipTemplate -> interaction weights.
    pub fn get_interaction_weights(&self) -> InteractionWeights {
        // These are intentionally simple defaults; callers can evolve them later.
        match self {
            Self::CasualFriendship => InteractionWeights {
                affirmation: 0.15,
                support: 0.20,
                deep_talk: 0.15,
                play: 0.25,
                planning: 0.15,
                conflict_repair: 0.10,
            }
            .normalized(),
            Self::SupportivePartnership => InteractionWeights {
                affirmation: 0.20,
                support: 0.30,
                deep_talk: 0.15,
                play: 0.10,
                planning: 0.15,
                conflict_repair: 0.10,
            }
            .normalized(),
            Self::GrowthOrientedPartnership => InteractionWeights {
                affirmation: 0.15,
                support: 0.20,
                deep_talk: 0.30,
                play: 0.10,
                planning: 0.20,
                conflict_repair: 0.05,
            }
            .normalized(),
            Self::IntimatePartnership { intimacy_level } => {
                let deep_bonus = match intimacy_level {
                    IntimacyLevel::Light => 0.00,
                    IntimacyLevel::Deep => 0.05,
                    IntimacyLevel::Eternal => 0.10,
                };
                InteractionWeights {
                    affirmation: 0.25,
                    support: 0.25,
                    deep_talk: 0.25 + deep_bonus,
                    play: 0.10,
                    planning: 0.10,
                    conflict_repair: 0.05,
                }
                .normalized()
            }
        }
    }

    /// #1 Load template from `.env` (RELATIONSHIP_TEMPLATE), otherwise return provided default.
    pub fn from_env_or_default(default: Self) -> Self {
        dotenvy::dotenv().ok();
        let raw = std::env::var("RELATIONSHIP_TEMPLATE").unwrap_or_default();
        let intimacy = std::env::var("RELATIONSHIP_INTIMACY_LEVEL")
            .ok()
            .and_then(|s| IntimacyLevel::from_str(&s).ok())
            .unwrap_or_default();
        match raw.trim() {
            "CasualFriendship" => Self::CasualFriendship,
            "SupportivePartnership" => Self::SupportivePartnership,
            "GrowthOrientedPartnership" => Self::GrowthOrientedPartnership,
            "IntimatePartnership" => Self::IntimatePartnership {
                intimacy_level: intimacy,
            },
            _ => default,
        }
    }
}

