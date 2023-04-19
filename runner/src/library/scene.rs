use super::easing;
use crate::game_state::{Globals, Transition, GAME_GLOBALS};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use vngineer_core::script::*;

#[intuicio_function(module_name = "vn_scene", use_context)]
fn scene(
    context: &mut Context,
    name: VnValue,
    duration: VnValue,
    ease_in: VnValue,
    ease_out: VnValue,
    ease_in_out: VnValue,
) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!");
    let duration = duration.as_number().unwrap_or_default();
    let easing = easing(ease_in, ease_out, ease_in_out);
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    let from = globals.scene_transition.to.take();
    let to = if globals.scenes.contains_key(name) {
        Some(name.to_owned())
    } else {
        None
    };
    globals.scene_transition = Transition {
        from,
        to,
        time: 0.0,
        duration,
        easing,
    };
    VnResult::Continue
}

pub fn install(registry: &mut Registry) {
    registry.add_function(scene::define_function(registry));
}
