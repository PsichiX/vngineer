use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use intuicio_frontend_simpleton::prelude::*;
use vngineer_core::prelude::*;

#[derive(IntuicioStruct, Default)]
struct VnResultJumpTo {
    pub chapter: Reference,
    pub label: Reference,
}

#[derive(IntuicioStruct, Default)]
struct VnResultEnter {
    pub chapter: Reference,
    pub label: Reference,
}

#[derive(IntuicioStruct, Default)]
struct VnResultExit;

pub fn value_to_reference(value: &VnValue, registry: &Registry) -> Reference {
    match value {
        VnValue::None => Reference::null(),
        VnValue::Boolean(value) => Reference::new_boolean(*value, registry),
        VnValue::Number(value) => Reference::new_real(*value, registry),
        VnValue::Text(value) => Reference::new_text(value.to_owned(), registry),
        VnValue::Color(value) => Reference::new_integer(*value as Integer, registry),
        VnValue::Array(value) => Reference::new_array(
            value
                .iter()
                .map(|value| value_to_reference(value, registry))
                .collect(),
            registry,
        ),
        VnValue::Map(value) => Reference::new_map(
            value
                .iter()
                .map(|(key, value)| (key.to_owned(), value_to_reference(value, registry)))
                .collect(),
            registry,
        ),
    }
}

#[intuicio_function(module_name = "vn", use_context, use_registry)]
pub fn simpleton(
    context: &mut Context,
    registry: &Registry,
    name: VnValue,
    module_name: VnValue,
    arguments: VnValue,
) -> VnResult {
    let name = name.as_text().expect("`name` is not a text!");
    let module_name = module_name.as_text();
    let arguments = arguments.as_map().expect("`arguments` is not a map!");
    let function = registry
        .find_function(FunctionQuery {
            name: Some(name.into()),
            module_name: module_name.map(|name| name.into()),
            ..Default::default()
        })
        .unwrap_or_else(|| {
            panic!(
                "Function `{}::{}` not found!",
                name,
                module_name.unwrap_or_default()
            );
        });
    for param in function.signature().inputs.iter().rev() {
        if let Some(value) = arguments.get(&param.name) {
            context.stack().push(value_to_reference(value, registry));
        }
    }
    function.invoke(context, registry);
    if function.signature().outputs.len() != 1 {
        panic!(
            "Function `{}::{}` must have single output!",
            name,
            module_name.unwrap_or_default()
        );
    }
    let result = context.stack().pop::<Reference>().unwrap_or_else(|| {
        panic!(
            "Function `{}::{}` must return `Reference`!",
            name,
            module_name.unwrap_or_default()
        );
    });
    let result = if let Some(result) = result.read::<VnResultJumpTo>() {
        let chapter = result.chapter.read::<Text>().map(|value| value.to_owned());
        let label = result.label.read::<Text>().map(|value| value.to_owned());
        VnResult::JumpTo { chapter, label }
    } else if let Some(result) = result.read::<VnResultEnter>() {
        let chapter = result.chapter.read::<Text>().map(|value| value.to_owned());
        let label = result.label.read::<Text>().map(|value| value.to_owned());
        VnResult::Enter { chapter, label }
    } else if result.read::<VnResultExit>().is_some() {
        VnResult::Exit
    } else {
        VnResult::Continue
    };
    result
}

pub fn install(registry: &mut Registry) {
    registry.add_struct(VnResultJumpTo::define_struct(registry));
    registry.add_struct(VnResultEnter::define_struct(registry));
    registry.add_struct(VnResultExit::define_struct(registry));
    registry.add_function(simpleton::define_function(registry));
}
