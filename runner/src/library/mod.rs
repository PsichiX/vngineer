pub mod character;
pub mod dialog;
pub mod render;
pub mod scene;
pub mod screen;

use easer::functions::*;
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use intuicio_frontend_simpleton::prelude::*;
use tetra::graphics::Rectangle;
use vngineer_core::script::*;

use crate::game_state::{Globals, GAME_GLOBALS};

#[allow(clippy::type_complexity)]
pub fn easing(
    ease_in: VnValue,
    ease_out: VnValue,
    ease_in_out: VnValue,
) -> Option<fn(f64, f64, f64, f64) -> f64> {
    if let Some(easing) = ease_in.as_text() {
        match easing {
            "back" => Some(Back::ease_in),
            "bounce" => Some(Bounce::ease_in),
            "circ" => Some(Circ::ease_in),
            "elastic" => Some(Elastic::ease_in),
            "expo" => Some(Expo::ease_in),
            "linear" => Some(Linear::ease_in),
            "quad" => Some(Quad::ease_in),
            "quart" => Some(Quart::ease_in),
            "quint" => Some(Quint::ease_in),
            "sine" => Some(Sine::ease_in),
            _ => None,
        }
    } else if let Some(easing) = ease_out.as_text() {
        match easing {
            "back" => Some(Back::ease_out),
            "bounce" => Some(Bounce::ease_out),
            "circ" => Some(Circ::ease_out),
            "elastic" => Some(Elastic::ease_out),
            "expo" => Some(Expo::ease_out),
            "linear" => Some(Linear::ease_out),
            "quad" => Some(Quad::ease_out),
            "quart" => Some(Quart::ease_out),
            "quint" => Some(Quint::ease_out),
            "sine" => Some(Sine::ease_out),
            _ => None,
        }
    } else if let Some(easing) = ease_in_out.as_text() {
        match easing {
            "back" => Some(Back::ease_in_out),
            "bounce" => Some(Bounce::ease_in_out),
            "circ" => Some(Circ::ease_in_out),
            "elastic" => Some(Elastic::ease_in_out),
            "expo" => Some(Expo::ease_in_out),
            "linear" => Some(Linear::ease_in_out),
            "quad" => Some(Quad::ease_in_out),
            "quart" => Some(Quart::ease_in_out),
            "quint" => Some(Quint::ease_in_out),
            "sine" => Some(Sine::ease_in_out),
            _ => None,
        }
    } else {
        None
    }
}

#[intuicio_function(module_name = "vn_debug")]
fn print(message: VnValue) -> VnResult {
    println!("{:#?}", message);
    VnResult::Continue
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "GameTransition", module_name = "vn")]
pub struct GameTransition {
    pub from: Reference,
    pub to: Reference,
    pub factor: Reference,
}

#[intuicio_function(module_name = "vn", use_context)]
fn config(context: &Context, name: Reference) -> Reference {
    let name = name.read::<Text>().expect("`name` is not a text!");
    let globals = context.custom::<Globals>(GAME_GLOBALS).unwrap();
    globals
        .configs
        .get(name.as_str())
        .cloned()
        .unwrap_or_default()
}

#[intuicio_function(module_name = "vn", use_context, use_registry)]
fn hover(context: &Context, registry: &Registry, region: Reference) -> Reference {
    let region = region.read::<Array>().expect("`region` is not an array!");
    let region = Rectangle::new(
        *region[0]
            .read::<Real>()
            .expect("`region[0]` is not a number!") as f32,
        *region[1]
            .read::<Real>()
            .expect("`region[1]` is not a number!") as f32,
        *region[2]
            .read::<Real>()
            .expect("`region[2]` is not a number!") as f32,
        *region[3]
            .read::<Real>()
            .expect("`region[3]` is not a number!") as f32,
    );
    let globals = context.custom::<Globals>(GAME_GLOBALS).unwrap();
    Reference::new_boolean(region.contains_point(globals.mouse_position), registry)
}

#[intuicio_function(module_name = "vn", use_context, use_registry)]
fn clicked(context: &Context, registry: &Registry) -> Reference {
    let globals = context.custom::<Globals>(GAME_GLOBALS).unwrap();
    Reference::new_boolean(globals.clicked, registry)
}

pub fn install(registry: &mut Registry) {
    scene::install(registry);
    character::install(registry);
    dialog::install(registry);
    render::install(registry);
    screen::install(registry);

    registry.add_function(print::define_function(registry));
    registry.add_struct(GameTransition::define_struct(registry));
    registry.add_function(config::define_function(registry));
    registry.add_function(hover::define_function(registry));
    registry.add_function(clicked::define_function(registry));
}
