// skill_system/src/relationship_integration.rs
// Optional integration glue for `extensions/relationship_dynamics`.

use crate::RelationshipContext;

/// Convert relationship dynamics state into a lightweight [`RelationshipContext`].
///
/// This allows skills to adapt to relationship templates and (safe/consensual) preferences without
/// binding core skill logic to the relationship engine.
pub fn map_partnership_to_context(p: &relationship_dynamics::Partnership) -> RelationshipContext {
    RelationshipContext {
        template: Some(p.template.template_name().to_string()),
        intimacy_level: p.template.intimacy_level().map(|x| x.to_string()),
        attachment_style: Some(format!("{:?}", p.attachment_profile.style)),
        fantasy_preferences: vec![],
    }
}

