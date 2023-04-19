use crate::{script::*, vm::*};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};

#[intuicio_function(module_name = "vn", use_context)]
pub fn set_global(context: &mut Context, name: VnValue, value: VnValue) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!");
    let globals = context
        .custom_mut::<Globals>(VN_GLOBALS)
        .expect("Cannot access VN globals!");
    globals.properties.insert(name.to_owned(), value);
    VnResult::Continue
}

#[intuicio_function(module_name = "vn", use_context)]
pub fn delete_global(context: &mut Context, name: VnValue) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!");
    let globals = context
        .custom_mut::<Globals>(VN_GLOBALS)
        .expect("Cannot access VN globals!");
    globals.properties.remove(name);
    VnResult::Continue
}

fn validate_query(
    global: &VnValue,
    is_type: VnValue,
    equals: VnValue,
    not_equals: VnValue,
    less_than: VnValue,
    greater_than: VnValue,
    has_items: VnValue,
) -> bool {
    if !is_type.is_none() && !global.is_same_type(&is_type) {
        return false;
    }
    if !equals.is_none() && global != &equals {
        return false;
    }
    if !not_equals.is_none() && global == &not_equals {
        return false;
    }
    if !less_than.is_none() {
        let result = match (global, less_than) {
            (VnValue::Number(a), VnValue::Number(b)) => *a < b,
            _ => false,
        };
        if !result {
            return false;
        }
    }
    if !greater_than.is_none() {
        let result = match (global, greater_than) {
            (VnValue::Number(a), VnValue::Number(b)) => *a > b,
            _ => false,
        };
        if !result {
            return false;
        }
    }
    if !has_items.is_none() {
        let result = match (global, has_items) {
            (VnValue::Array(a), VnValue::Array(b)) => b.iter().all(|item| a.contains(item)),
            (VnValue::Map(a), VnValue::Map(b)) => b
                .iter()
                .all(|(key, value)| a.get(key).map(|v| v == value).unwrap_or_default()),
            _ => false,
        };
        if !result {
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
#[intuicio_function(module_name = "vn", use_context)]
pub fn jump(
    context: &Context,
    chapter: VnValue,
    label: VnValue,
    global: VnValue,
    is_type: VnValue,
    equals: VnValue,
    not_equals: VnValue,
    less_than: VnValue,
    greater_than: VnValue,
    has_items: VnValue,
) -> VnResult {
    if let Some(global) = global.as_text() {
        let globals = context
            .custom::<Globals>(VN_GLOBALS)
            .expect("Cannot access VN globals!");
        if let Some(global) = globals.properties.get(global) {
            if !validate_query(
                global,
                is_type,
                equals,
                not_equals,
                less_than,
                greater_than,
                has_items,
            ) {
                return VnResult::Continue;
            }
        } else {
            return VnResult::Continue;
        }
    }
    let chapter = chapter.as_text().map(|name| name.to_owned());
    let label = label.as_text().map(|label| label.to_owned());
    VnResult::JumpTo { chapter, label }
}

#[allow(clippy::too_many_arguments)]
#[intuicio_function(module_name = "vn", use_context)]
pub fn enter(
    context: &Context,
    chapter: VnValue,
    label: VnValue,
    global: VnValue,
    is_type: VnValue,
    equals: VnValue,
    not_equals: VnValue,
    less_than: VnValue,
    greater_than: VnValue,
    has_items: VnValue,
) -> VnResult {
    if let Some(global) = global.as_text() {
        let globals = context
            .custom::<Globals>(VN_GLOBALS)
            .expect("Cannot access VN globals!");
        if let Some(global) = globals.properties.get(global) {
            if !validate_query(
                global,
                is_type,
                equals,
                not_equals,
                less_than,
                greater_than,
                has_items,
            ) {
                return VnResult::Continue;
            }
        } else {
            return VnResult::Continue;
        }
    }
    let chapter = chapter.as_text().map(|name| name.to_owned());
    let label = label.as_text().map(|label| label.to_owned());
    VnResult::Enter { chapter, label }
}

#[allow(clippy::too_many_arguments)]
#[intuicio_function(module_name = "vn", use_context)]
pub fn exit(
    context: &Context,
    global: VnValue,
    is_type: VnValue,
    equals: VnValue,
    not_equals: VnValue,
    less_than: VnValue,
    greater_than: VnValue,
    has_items: VnValue,
) -> VnResult {
    if let Some(global) = global.as_text() {
        let globals = context
            .custom::<Globals>(VN_GLOBALS)
            .expect("Cannot access VN globals!");
        if let Some(global) = globals.properties.get(global) {
            if !validate_query(
                global,
                is_type,
                equals,
                not_equals,
                less_than,
                greater_than,
                has_items,
            ) {
                return VnResult::Continue;
            }
        } else {
            return VnResult::Continue;
        }
    }
    VnResult::Exit
}

pub fn install(registry: &mut Registry) {
    registry.add_struct(define_native_struct! {
        registry => mod vn struct VnValue (VnValue) {}
    });
    registry.add_struct(define_native_struct! {
        registry => mod vn struct VnResult (VnResult) {}
    });
    registry.add_function(set_global::define_function(registry));
    registry.add_function(delete_global::define_function(registry));
    registry.add_function(jump::define_function(registry));
    registry.add_function(enter::define_function(registry));
    registry.add_function(exit::define_function(registry));
}
