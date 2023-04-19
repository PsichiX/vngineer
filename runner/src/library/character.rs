use super::easing;
use crate::game_state::{CharacterTransition, Globals, Transition, GAME_GLOBALS};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use vngineer_core::script::*;

#[intuicio_function(module_name = "vn_character", use_context)]
fn show(
    context: &mut Context,
    character: VnValue,
    variant: VnValue,
    duration: VnValue,
    ease_in: VnValue,
    ease_out: VnValue,
    ease_in_out: VnValue,
) -> VnResult {
    let character = character.as_text().expect("`character` is not a text!");
    let variant = variant.as_text().unwrap_or("default");
    let duration = duration.as_number().unwrap_or_default();
    let easing = easing(ease_in, ease_out, ease_in_out);
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    let found = globals.character_transitions.iter().position(|transition| {
        transition
            .to
            .as_ref()
            .map(|to| to.character == character)
            .unwrap_or_default()
    });
    let transition = if let Some(index) = found {
        let from = globals.character_transitions.remove(index).to.take();
        Transition {
            from,
            to: Some(CharacterTransition {
                character: character.to_owned(),
                variant: variant.to_owned(),
            }),
            time: 0.0,
            duration,
            easing,
        }
    } else {
        Transition {
            from: None,
            to: Some(CharacterTransition {
                character: character.to_owned(),
                variant: variant.to_owned(),
            }),
            time: 0.0,
            duration,
            easing,
        }
    };
    globals.character_transitions.push(transition);
    VnResult::Continue
}

#[intuicio_function(module_name = "vn_character", use_context)]
fn hide(
    context: &mut Context,
    character: VnValue,
    duration: VnValue,
    ease_in: VnValue,
    ease_out: VnValue,
    ease_in_out: VnValue,
) -> VnResult {
    let character = character.as_text().expect("`character` is not a text!");
    let duration = duration.as_number().unwrap_or_default();
    let easing = easing(ease_in, ease_out, ease_in_out);
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    let found = globals.character_transitions.iter().position(|transition| {
        transition
            .to
            .as_ref()
            .map(|to| to.character == character)
            .unwrap_or_default()
    });
    if let Some(index) = found {
        let from = globals.character_transitions.remove(index).to.take();
        globals.character_transitions.push(Transition {
            from,
            to: None,
            time: 0.0,
            duration,
            easing,
        });
    }
    VnResult::Continue
}

pub fn install(registry: &mut Registry) {
    registry.add_function(show::define_function(registry));
    registry.add_function(hide::define_function(registry));
}
