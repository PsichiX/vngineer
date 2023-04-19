use crate::game_state::{Border, Globals, RenderCommand, GAME_GLOBALS};
use intuicio_essentials::{core as intuicio_core, data as intuicio_data, prelude::*};
use intuicio_frontend_simpleton::prelude::*;
use tetra::{graphics::Rectangle, math::Vec2};

#[intuicio_function(module_name = "render", use_context)]
fn draw_image(
    context: &mut Context,
    texture_asset: Reference,
    region: Reference,
    border: Reference,
    visibility: Reference,
) -> Reference {
    let texture_asset = texture_asset
        .read::<Text>()
        .expect("`texture_asset` is not a text!")
        .to_owned();
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
    let border = border.read::<Array>().map(|array| Border {
        left: *array[0]
            .read::<Real>()
            .expect("`region[0]` is not a number!") as f32,
        right: *array[1]
            .read::<Real>()
            .expect("`region[1]` is not a number!") as f32,
        top: *array[1]
            .read::<Real>()
            .expect("`region[2]` is not a number!") as f32,
        bottom: *array[1]
            .read::<Real>()
            .expect("`region[3]` is not a number!") as f32,
    });
    let visibility = *visibility
        .read::<Real>()
        .expect("`visibility` is nto a number!") as f32;
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    globals.draw(RenderCommand::Image {
        texture_asset,
        region,
        border,
        visibility,
    });
    Reference::null()
}

#[intuicio_function(module_name = "render", use_context)]
fn draw_text(
    context: &mut Context,
    font_asset: Reference,
    size: Reference,
    text: Reference,
    region: Reference,
    alignment: Reference,
    visibility: Reference,
) -> Reference {
    let font_asset = font_asset
        .read::<Text>()
        .expect("`font_asset` is not a text!")
        .to_owned();
    let size = *size.read::<Real>().expect("`size` is not a number!") as f32;
    let text = text
        .read::<Text>()
        .expect("`text` is not a text!")
        .to_owned();
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
    let alignment = alignment
        .read::<Array>()
        .expect("`alignment` is not an array!");
    let alignment = Vec2::new(
        *alignment[0]
            .read::<Real>()
            .expect("`alignment[0]` is not a number!") as f32,
        *alignment[1]
            .read::<Real>()
            .expect("`alignment[1]` is not a number!") as f32,
    );
    let visibility = *visibility
        .read::<Real>()
        .expect("`visibility` is nto a number!") as f32;
    let globals = context.custom_mut::<Globals>(GAME_GLOBALS).unwrap();
    globals.draw(RenderCommand::Text {
        font_asset,
        size,
        text,
        region,
        alignment,
        visibility,
    });
    Reference::null()
}

pub fn install(registry: &mut Registry) {
    registry.add_function(draw_image::define_function(registry));
    registry.add_function(draw_text::define_function(registry));
}
