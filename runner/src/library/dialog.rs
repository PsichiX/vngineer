use super::{easing, GameTransition};
use crate::game_state::{DialogTransition, Globals, Transition, GAME_GLOBALS};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use intuicio_frontend_simpleton::prelude::*;
use vngineer_core::{
    script::*,
    vm::{Globals as VnGlobals, VN_GLOBALS},
};

const CHOICE_PROPERTY: &str = "CHOICE";

#[allow(clippy::too_many_arguments)]
#[intuicio_function(module_name = "vn_dialog", use_context)]
fn say(
    context: &mut Context,
    who: VnValue,
    what: VnValue,
    choices: VnValue,
    duration: VnValue,
    ease_in: VnValue,
    ease_out: VnValue,
    ease_in_out: VnValue,
    non_blocking: VnValue,
) -> VnResult {
    let who = who.as_text();
    let what = what.as_text().expect("`what` is not a text!");
    let choices = choices.as_array();
    let duration = duration.as_number().unwrap_or_default();
    let easing = easing(ease_in, ease_out, ease_in_out);
    let non_blocking = non_blocking.as_boolean().unwrap_or_default();
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    let from = globals.dialog_transition.to.take();
    globals.dialog_transition = Transition {
        from,
        to: Some(DialogTransition {
            character: who.map(|name| name.to_owned()),
            text: what.to_owned(),
            choices: choices
                .map(|choices| {
                    choices
                        .iter()
                        .enumerate()
                        .map(|(index, choice)| {
                            choice
                                .as_text()
                                .unwrap_or_else(|| panic!("`choices[{}]` is not a text!", index))
                                .to_owned()
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }),
        time: 0.0,
        duration,
        easing,
    };
    if !non_blocking {
        globals.is_dialog_blocked = true;
    }
    VnResult::Continue
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "GameDialogTransition", module_name = "vn")]
pub struct GameDialogTransition {
    pub character: Reference,
    pub text: Reference,
    pub choices: Reference,
}

#[intuicio_function(module_name = "dialog", use_context, use_registry)]
fn transition(context: &Context, registry: &Registry) -> Reference {
    let globals = context.custom::<Globals>(GAME_GLOBALS).unwrap();
    Reference::new(
        GameTransition {
            from: globals
                .dialog_transition
                .from
                .as_ref()
                .map(|from| {
                    Reference::new(
                        GameDialogTransition {
                            character: from
                                .character
                                .as_ref()
                                .map(|name| Reference::new_text(name.to_owned(), registry))
                                .unwrap_or_default(),
                            text: Reference::new_text(from.text.to_owned(), registry),
                            choices: if from.choices.is_empty() {
                                Reference::null()
                            } else {
                                Reference::new_array(
                                    from.choices
                                        .iter()
                                        .map(|choice| {
                                            Reference::new_text(choice.to_owned(), registry)
                                        })
                                        .collect(),
                                    registry,
                                )
                            },
                        },
                        registry,
                    )
                })
                .unwrap_or_default(),
            to: globals
                .dialog_transition
                .to
                .as_ref()
                .map(|to| {
                    Reference::new(
                        GameDialogTransition {
                            character: to
                                .character
                                .as_ref()
                                .map(|name| Reference::new_text(name.to_owned(), registry))
                                .unwrap_or_default(),
                            text: Reference::new_text(to.text.to_owned(), registry),
                            choices: if to.choices.is_empty() {
                                Reference::null()
                            } else {
                                Reference::new_array(
                                    to.choices
                                        .iter()
                                        .map(|choice| {
                                            Reference::new_text(choice.to_owned(), registry)
                                        })
                                        .collect(),
                                    registry,
                                )
                            },
                        },
                        registry,
                    )
                })
                .unwrap_or_default(),
            factor: Reference::new_real(globals.dialog_transition.sample(), registry),
        },
        registry,
    )
}

#[intuicio_function(module_name = "dialog", use_context)]
fn complete(context: &mut Context, choice: Reference) -> Reference {
    {
        let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
        globals.unblock_dialog();
    }
    if let Some(choice) = choice.read::<Integer>() {
        let globals = context.custom_mut::<VnGlobals>(VN_GLOBALS).unwrap();
        globals
            .properties
            .insert(CHOICE_PROPERTY.to_owned(), VnValue::Number(*choice as _));
    }
    Reference::null()
}

#[intuicio_function(module_name = "dialog", use_registry)]
fn text_fragment(
    registry: &Registry,
    text: Reference,
    percentage: Reference,
    forward: Reference,
) -> Reference {
    let text = text.read::<Text>().expect("`text` is not a text!");
    let percentage = *percentage
        .read::<Real>()
        .expect("`percentage` is not a number!");
    let forward = *forward
        .read::<Boolean>()
        .expect("`forward` is not a boolean!");
    let length = (text.len() as Real * percentage) as usize;
    if forward {
        Reference::new_text(text[0..length].to_owned(), registry)
    } else {
        Reference::new_text(text[length..].to_owned(), registry)
    }
}

pub fn install(registry: &mut Registry) {
    registry.add_function(say::define_function(registry));

    registry.add_struct(GameDialogTransition::define_struct(registry));
    registry.add_function(transition::define_function(registry));
    registry.add_function(complete::define_function(registry));
    registry.add_function(text_fragment::define_function(registry));
}
