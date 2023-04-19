use crate::game_state::{Globals, Screen, GAME_GLOBALS};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use vngineer_core::script::*;

#[intuicio_function(module_name = "vn_screen", use_context)]
fn show_screen(context: &mut Context, name: VnValue, module_name: VnValue) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!").to_owned();
    let module_name = module_name
        .as_text()
        .expect("`module_name` is not a text!")
        .to_owned();
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    if let Some(index) = globals
        .screens
        .iter()
        .position(|screen| screen.name == name && screen.module_name == module_name)
    {
        globals.screens.remove(index);
    }
    globals.screens.push(Screen { name, module_name });
    VnResult::Continue
}

#[intuicio_function(module_name = "vn_screen", use_context)]
fn hide_screen(context: &mut Context, name: VnValue, module_name: VnValue) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!");
    let module_name = module_name.as_text().expect("`module_name` is not a text!");
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    while let Some(index) = globals
        .screens
        .iter()
        .position(|screen| screen.name == name && screen.module_name == module_name)
    {
        globals.screens.remove(index);
    }
    VnResult::Continue
}

pub fn install(registry: &mut Registry) {
    registry.add_function(show_screen::define_function(registry));
    registry.add_function(hide_screen::define_function(registry));
}
