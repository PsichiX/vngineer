use intuicio_essentials::prelude::*;
use intuicio_frontend_simpleton::prelude::*;
use std::{cell::RefCell, collections::HashMap};
use tetra::{
    graphics::{
        self,
        text::{Font, Text},
        Camera, Color, DrawParams, FilterMode, NineSlice, Rectangle, Texture,
    },
    input::{get_mouse_position, is_mouse_button_pressed, MouseButton},
    math::Vec2,
    time::get_delta_time,
    window::{self, quit},
    Context as TetraContext, State, TetraError,
};
use vngineer_core::prelude::*;
use vngineer_simpleton::*;

pub const GAME_GLOBALS: &str = "game-globals";

#[derive(Debug, Clone)]
pub struct Screen {
    pub name: String,
    pub module_name: String,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub variants: HashMap<String, String>,
    pub position: Vec2<f32>,
    pub rotation: f32,
    pub scale: Vec2<f32>,
    pub alignment: Vec2<f32>,
    pub effect: Option<String>,
}

impl Character {
    pub fn new(id: &str, properties: &HashMap<String, VnValue>) -> Self {
        Self {
            name: properties
                .get("name")
                .map(|name| {
                    name.as_text()
                        .unwrap_or_else(|| panic!("Character '{}' `name` is not a text!", id))
                        .to_owned()
                })
                .unwrap_or_else(|| panic!("Character '{}' does not have `name` property!", id)),
            variants: properties
                .get("variants")
                .map(|variants| {
                    variants
                        .as_map()
                        .unwrap_or_else(|| panic!("Character '{}' `variants` is not a map!", id))
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.to_owned(),
                                value
                                    .as_text()
                                    .unwrap_or_else(|| {
                                        panic!(
                                            "Character '{}' variant '{}' is not a text!",
                                            id, key
                                        )
                                    })
                                    .to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_else(|| panic!("Character '{}' does not have `name` property!", id)),
            position: properties
                .get("position")
                .map(|position| {
                    let position = position
                        .as_map()
                        .unwrap_or_else(|| panic!("Character '{}' `position` is not a map!", id));
                    let x = position
                        .get("x")
                        .map(|x| {
                            x.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `position.x` is not a number!", id)
                            })
                        })
                        .unwrap_or_default() as f32;
                    let y = position
                        .get("y")
                        .map(|y| {
                            y.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `position.y` is not a number!", id)
                            })
                        })
                        .unwrap_or_default() as f32;
                    (x, y)
                })
                .unwrap_or_default()
                .into(),
            rotation: properties
                .get("rotation")
                .map(|rotation| {
                    rotation
                        .as_number()
                        .unwrap_or_else(|| panic!("Character '{}' `rotation is not a number!", id))
                        as f32
                })
                .unwrap_or_default(),
            scale: properties
                .get("scale")
                .map(|scale| {
                    let scale = scale
                        .as_map()
                        .unwrap_or_else(|| panic!("Character '{}' `scale` is not a map!", id));
                    let x = scale
                        .get("x")
                        .map(|x| {
                            x.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `scale.x` is not a number!", id)
                            })
                        })
                        .unwrap_or(1.0) as f32;
                    let y = scale
                        .get("y")
                        .map(|y| {
                            y.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `scale.y` is not a number!", id)
                            })
                        })
                        .unwrap_or(1.0) as f32;
                    (x, y)
                })
                .unwrap_or((1.0, 1.0))
                .into(),
            alignment: properties
                .get("alignment")
                .map(|alignment| {
                    let alignment = alignment
                        .as_map()
                        .unwrap_or_else(|| panic!("Character '{}' `alignment` is not a map!", id));
                    let x = alignment
                        .get("x")
                        .map(|x| {
                            x.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `alignment.x` is not a number!", id)
                            })
                        })
                        .unwrap_or(0.5) as f32;
                    let y = alignment
                        .get("y")
                        .map(|y| {
                            y.as_number().unwrap_or_else(|| {
                                panic!("Character '{}' `alignment.y` is not a number!", id)
                            })
                        })
                        .unwrap_or(0.5) as f32;
                    (x, y)
                })
                .unwrap_or((0.5, 0.5))
                .into(),
            effect: properties.get("effect").map(|effect| {
                effect
                    .as_text()
                    .unwrap_or_else(|| panic!("Character '{}' `effect` is not a text!", id))
                    .to_owned()
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub background: String,
    pub effect: Option<String>,
}

impl Scene {
    pub fn new(id: &str, properties: &HashMap<String, VnValue>) -> Self {
        Self {
            background: properties
                .get("background")
                .map(|background| {
                    background
                        .as_text()
                        .unwrap_or_else(|| panic!("Scene '{}' `background` is not a text!", id))
                })
                .unwrap_or_else(|| panic!("Scene '{}' does not have `background` property!", id))
                .to_owned(),
            effect: properties.get("effect").map(|effect| {
                effect
                    .as_text()
                    .unwrap_or_else(|| panic!("Scene '{}' `effect` is not a text!", id))
                    .to_owned()
            }),
        }
    }
}

pub struct Resource<T, const ALIVE_TIME: u32> {
    data: T,
    pub time_left: f64,
}

impl<T, const ALIVE_TIME: u32> Resource<T, ALIVE_TIME> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            time_left: ALIVE_TIME as _,
        }
    }

    pub fn heartbeat(&mut self) {
        self.time_left = ALIVE_TIME as _;
    }
}

#[derive(Debug)]
pub struct Transition<T> {
    pub from: Option<T>,
    pub to: Option<T>,
    pub time: f64,
    pub duration: f64,
    #[allow(clippy::type_complexity)]
    pub easing: Option<fn(f64, f64, f64, f64) -> f64>,
}

impl<T> Default for Transition<T> {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            time: 0.0,
            duration: 0.0,
            easing: None,
        }
    }
}

impl<T> Transition<T> {
    pub fn update(&mut self, delta_time: f64) {
        self.time = (self.time + delta_time).clamp(0.0, self.duration);
    }

    pub fn sample(&self) -> f64 {
        self.easing
            .map(|easing| easing(self.time, 0.0, 1.0, self.duration).clamp(0.0, 1.0))
            .unwrap_or(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.time >= self.duration - 1.0e-6
    }
}

#[derive(Debug)]
pub struct CharacterTransition {
    pub character: String,
    pub variant: String,
}

#[derive(Debug)]
pub struct DialogTransition {
    pub character: Option<String>,
    pub text: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Border {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug)]
pub enum RenderCommand {
    Image {
        texture_asset: String,
        region: Rectangle,
        border: Option<Border>,
        visibility: f32,
    },
    Text {
        font_asset: String,
        size: f32,
        text: String,
        region: Rectangle,
        alignment: Vec2<f32>,
        visibility: f32,
    },
}

impl RenderCommand {
    fn draw(self, ctx: &mut TetraContext, globals: &mut Globals) {
        match self {
            RenderCommand::Image {
                texture_asset,
                region,
                border,
                visibility,
            } => {
                let texture = if let Some(texture) = globals.textures.get(&texture_asset) {
                    texture.borrow().data.clone()
                } else {
                    let mut texture = Texture::new(ctx, &texture_asset)
                        .unwrap_or_else(|_| panic!("Could not load `{}` texture!", &texture_asset));
                    texture.set_filter_mode(ctx, FilterMode::Linear);
                    globals.textures.insert(
                        texture_asset.to_owned(),
                        Resource::new(texture.clone()).into(),
                    );
                    texture
                };
                let position = region.top_left();
                let size = texture.size();
                if let Some(border) = border {
                    texture.draw_nine_slice(
                        ctx,
                        &NineSlice {
                            region: Rectangle::new(0.0, 0.0, size.0 as f32, size.1 as f32),
                            left: border.left,
                            right: border.right,
                            top: border.top,
                            bottom: border.bottom,
                        },
                        region.width,
                        region.height,
                        DrawParams {
                            position,
                            color: Color::WHITE.with_alpha(visibility),
                            ..Default::default()
                        },
                    )
                } else {
                    texture.draw(
                        ctx,
                        DrawParams {
                            position,
                            scale: Vec2::new(region.width, region.height)
                                / Vec2::new(size.0 as f32, size.1 as f32),
                            color: Color::WHITE.with_alpha(visibility),
                            ..Default::default()
                        },
                    );
                }
            }
            RenderCommand::Text {
                font_asset,
                size,
                text,
                region,
                alignment,
                visibility,
            } => {
                let font = if let Some(font) = globals.fonts.get(&font_asset) {
                    font.borrow().data.clone()
                } else {
                    let mut font = Font::vector(ctx, &font_asset, size)
                        .unwrap_or_else(|_| panic!("Could not load `{}` font!", &font_asset));
                    font.set_filter_mode(ctx, FilterMode::Linear);
                    globals
                        .fonts
                        .insert(font_asset.to_owned(), Resource::new(font.clone()).into());
                    font
                };
                let position = region.top_left();
                let container_size = Vec2::new(region.width, region.height);
                let mut renderable = Text::new(text.as_str(), font);
                renderable.set_max_width(Some(region.width));
                let size = renderable
                    .get_bounds(ctx)
                    .map(|rect| Vec2::new(rect.width, rect.height))
                    .unwrap_or_default();
                let position = Vec2::lerp(position, position + container_size - size, alignment);
                renderable.draw(
                    ctx,
                    DrawParams {
                        position,
                        color: Color::WHITE.with_alpha(visibility),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

pub struct Globals {
    pub configs: HashMap<String, Reference>,
    pub screens: Vec<Screen>,
    pub characters: HashMap<String, Character>,
    pub scenes: HashMap<String, Scene>,
    pub textures: HashMap<String, RefCell<Resource<Texture, 10>>>,
    pub fonts: HashMap<String, RefCell<Resource<Font, 10>>>,
    pub character_transitions: Vec<Transition<CharacterTransition>>,
    pub scene_transition: Transition<String>,
    pub dialog_transition: Transition<DialogTransition>,
    pub mouse_position: Vec2<f32>,
    pub clicked: bool,
    pub(crate) is_dialog_blocked: bool,
    render_commands: Vec<RenderCommand>,
    camera: Camera,
}

impl Globals {
    pub fn unblock_dialog(&mut self) {
        if self.dialog_transition.is_complete() {
            self.is_dialog_blocked = false;
        }
    }

    pub fn draw(&mut self, command: RenderCommand) {
        self.render_commands.push(command);
    }

    fn in_progress(&self) -> bool {
        self.is_dialog_blocked
            || self
                .character_transitions
                .iter()
                .any(|transition| !transition.is_complete())
            || !self.scene_transition.is_complete()
            || !self.dialog_transition.is_complete()
    }

    fn manage_assets_loading(&mut self, ctx: &mut TetraContext) {
        if let Some(asset) = self
            .scene_transition
            .from
            .as_ref()
            .and_then(|name| self.scenes.get(name))
            .map(|scene| &scene.background)
        {
            if !self.textures.contains_key(asset) {
                let mut texture = Texture::new(ctx, asset)
                    .unwrap_or_else(|_| panic!("Could not load `{}` texture!", asset));
                texture.set_filter_mode(ctx, FilterMode::Linear);
                self.textures
                    .insert(asset.to_owned(), Resource::new(texture).into());
            }
        }
        if let Some(asset) = self
            .scene_transition
            .to
            .as_ref()
            .and_then(|name| self.scenes.get(name))
            .map(|scene| &scene.background)
        {
            if !self.textures.contains_key(asset) {
                let mut texture = Texture::new(ctx, asset)
                    .unwrap_or_else(|_| panic!("Could not load `{}` texture!", asset));
                texture.set_filter_mode(ctx, FilterMode::Linear);
                self.textures
                    .insert(asset.to_owned(), Resource::new(texture).into());
            }
        }
        for transition in &self.character_transitions {
            if let Some(asset) = transition.from.as_ref().and_then(|transition| {
                self.characters
                    .get(&transition.character)
                    .and_then(|character| character.variants.get(&transition.variant))
            }) {
                if !self.textures.contains_key(asset) {
                    let mut texture = Texture::new(ctx, asset)
                        .unwrap_or_else(|_| panic!("Could not load `{}` texture!", asset));
                    texture.set_filter_mode(ctx, FilterMode::Linear);
                    self.textures
                        .insert(asset.to_owned(), Resource::new(texture).into());
                }
            }
            if let Some(asset) = transition.to.as_ref().and_then(|transition| {
                self.characters
                    .get(&transition.character)
                    .and_then(|character| character.variants.get(&transition.variant))
            }) {
                if !self.textures.contains_key(asset) {
                    let mut texture = Texture::new(ctx, asset)
                        .unwrap_or_else(|_| panic!("Could not load `{}` texture!", asset));
                    texture.set_filter_mode(ctx, FilterMode::Linear);
                    self.textures
                        .insert(asset.to_owned(), Resource::new(texture).into());
                }
            }
        }
    }

    fn manage_assets_lifetime(&mut self, delta_time: f64) {
        let to_remove = self
            .textures
            .iter_mut()
            .filter_map(|(key, value)| {
                let mut value = value.borrow_mut();
                value.time_left -= delta_time;
                if value.time_left <= 0.0 {
                    Some(key.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for key in to_remove {
            self.textures.remove(&key);
        }

        let to_remove = self
            .fonts
            .iter_mut()
            .filter_map(|(key, value)| {
                let mut value = value.borrow_mut();
                value.time_left -= delta_time;
                if value.time_left <= 0.0 {
                    Some(key.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for key in to_remove {
            self.fonts.remove(&key);
        }
    }

    fn update_transitions(&mut self, delta_time: f64) {
        self.dialog_transition.update(delta_time);
        self.scene_transition.update(delta_time);
        let to_remove = self
            .character_transitions
            .iter_mut()
            .enumerate()
            .filter_map(|(index, transition)| {
                transition.update(delta_time);
                if transition.to.is_none() && transition.is_complete() {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for index in to_remove.into_iter().rev() {
            self.character_transitions.remove(index);
        }
    }

    fn update_inputs(&mut self, ctx: &mut TetraContext) {
        self.clicked = is_mouse_button_pressed(ctx, MouseButton::Left);
        self.mouse_position = get_mouse_position(ctx);
    }

    fn update_camera(&mut self, ctx: &mut TetraContext, desired_width: f32, desired_height: f32) {
        self.camera = Camera::with_window_size(ctx);
        let viewport_aspect = self.camera.viewport_width / self.camera.viewport_height;
        let desired_aspect = desired_width / desired_height;
        self.camera.scale = if viewport_aspect > desired_aspect {
            self.camera.viewport_width / desired_width
        } else {
            self.camera.viewport_height / desired_height
        }
        .into();
        self.camera.update();
    }

    fn draw_scene(&mut self, ctx: &mut TetraContext) {
        let from = self
            .scene_transition
            .from
            .as_ref()
            .filter(|name| {
                self.scene_transition
                    .to
                    .as_ref()
                    .map(|n| n != *name)
                    .unwrap_or(true)
            })
            .and_then(|name| self.scenes.get(name))
            .and_then(|scene| Some(self.textures.get(&scene.background)?.borrow_mut()));
        let to = self
            .scene_transition
            .to
            .as_ref()
            .and_then(|name| self.scenes.get(name))
            .and_then(|scene| Some(self.textures.get(&scene.background)?.borrow_mut()));
        let factor = self.scene_transition.sample() as f32;
        if let Some(mut texture) = from {
            let origin = Vec2 {
                x: texture.data.width() as f32 * 0.5,
                y: texture.data.height() as f32 * 0.5,
            };
            texture.heartbeat();
            texture.data.draw(
                ctx,
                DrawParams {
                    origin,
                    color: Color::WHITE.with_alpha(1.0 - factor),
                    ..Default::default()
                },
            );
        }
        if let Some(mut texture) = to {
            let origin = Vec2 {
                x: texture.data.width() as f32 * 0.5,
                y: texture.data.height() as f32 * 0.5,
            };
            texture.heartbeat();
            texture.data.draw(
                ctx,
                DrawParams {
                    origin,
                    color: Color::WHITE.with_alpha(factor),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_characters(&mut self, ctx: &mut TetraContext) {
        for character_transition in &self.character_transitions {
            let from = character_transition.from.as_ref().and_then(|transition| {
                if let Some(t) = character_transition.to.as_ref() {
                    if t.character == transition.character && t.variant == transition.variant {
                        return None;
                    }
                }
                let character = self.characters.get(&transition.character)?;
                let variant = character.variants.get(&transition.variant)?;
                Some((
                    character.position,
                    character.scale,
                    character.rotation,
                    character.alignment,
                    self.textures.get(variant)?.borrow_mut(),
                ))
            });
            let to = character_transition.to.as_ref().and_then(|transition| {
                let character = self.characters.get(&transition.character)?;
                let variant = character.variants.get(&transition.variant)?;
                Some((
                    character.position,
                    character.scale,
                    character.rotation,
                    character.alignment,
                    self.textures.get(variant)?.borrow_mut(),
                ))
            });
            let factor = character_transition.sample() as f32;
            let camera_region = self.camera.visible_rect();
            if let Some((position, scale, rotation, alignment, mut texture)) = from {
                let position = Vec2 {
                    x: camera_region.width * position.x * 0.5,
                    y: camera_region.height * position.y * 0.5,
                };
                let origin = Vec2 {
                    x: texture.data.width() as f32,
                    y: texture.data.height() as f32,
                } * alignment;
                texture.heartbeat();
                texture.data.draw(
                    ctx,
                    DrawParams {
                        position,
                        rotation,
                        scale,
                        origin,
                        color: Color::WHITE.with_alpha(1.0 - factor),
                    },
                );
            }
            if let Some((position, scale, rotation, alignment, mut texture)) = to {
                let position = Vec2 {
                    x: camera_region.width * position.x * 0.5,
                    y: camera_region.height * position.y * 0.5,
                };
                let origin = Vec2 {
                    x: texture.data.width() as f32,
                    y: texture.data.height() as f32,
                } * alignment;
                texture.heartbeat();
                texture.data.draw(
                    ctx,
                    DrawParams {
                        position,
                        rotation,
                        scale,
                        origin,
                        color: Color::WHITE.with_alpha(factor),
                    },
                );
            }
        }
    }

    fn draw_commands(&mut self, ctx: &mut TetraContext) {
        for command in std::mem::take(&mut self.render_commands) {
            command.draw(ctx, self);
        }
    }
}

pub struct GameState {
    pub vm: Vm,
    pub desired_width: f32,
    pub desired_height: f32,
}

impl GameState {
    pub fn new(
        mut vm: Vm,
        story: VnStory,
        entry: &str,
        desired_width: f32,
        desired_height: f32,
    ) -> Self {
        vm.enter(entry, None);
        let (context, registry) = vm.host_mut().context_and_registry();
        let configs = story
            .configs
            .iter()
            .map(|(name, config)| {
                let properties = config
                    .properties
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value_to_reference(value, registry)))
                    .collect();
                (name.to_owned(), Reference::new_map(properties, registry))
            })
            .collect();
        let characters = story
            .characters
            .iter()
            .map(|(id, character)| (id.to_owned(), Character::new(id, &character.properties)))
            .collect();
        let scenes = story
            .scenes
            .iter()
            .map(|(id, scene)| (id.to_owned(), Scene::new(id, &scene.properties)))
            .collect();
        context.set_custom(
            GAME_GLOBALS,
            Globals {
                configs,
                screens: Default::default(),
                characters,
                scenes,
                textures: Default::default(),
                fonts: Default::default(),
                character_transitions: Default::default(),
                scene_transition: Default::default(),
                dialog_transition: Default::default(),
                mouse_position: Default::default(),
                clicked: false,
                is_dialog_blocked: false,
                render_commands: Default::default(),
                camera: Camera::new(0.0, 0.0),
            },
        );
        Self {
            vm,
            desired_width,
            desired_height,
        }
    }

    fn draw_screens(&mut self, width: Real, height: Real) {
        let host = self.vm.host_mut();
        let (context, registry) = host.context_and_registry();
        let screens = context
            .custom::<Globals>(GAME_GLOBALS)
            .unwrap()
            .screens
            .to_owned();
        let width = Reference::new_real(width, registry);
        let height = Reference::new_real(height, registry);
        for screen in screens {
            let function = registry
                .find_function(FunctionQuery {
                    name: Some(screen.name.as_str().into()),
                    module_name: Some(screen.module_name.as_str().into()),
                    ..Default::default()
                })
                .unwrap_or_else(|| {
                    panic!(
                        "Function `{}::{}` not found!",
                        screen.module_name, screen.name,
                    )
                });
            function.call::<(Reference,), _>(
                context,
                registry,
                (width.clone(), height.clone()),
                true,
            );
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut TetraContext) -> Result<(), TetraError> {
        if !self.vm.is_running() {
            quit(ctx);
            return Ok(());
        }
        let delta_time = get_delta_time(ctx).as_secs_f64();
        let globals = self
            .vm
            .host_mut()
            .context()
            .custom_mut::<Globals>(GAME_GLOBALS)
            .unwrap();
        globals.manage_assets_loading(ctx);
        globals.manage_assets_lifetime(delta_time);
        globals.update_transitions(delta_time);
        globals.update_inputs(ctx);
        if !globals.in_progress() {
            self.vm.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut TetraContext) -> Result<(), TetraError> {
        graphics::clear(ctx, Color::BLACK);
        {
            let globals = self
                .vm
                .host_mut()
                .context()
                .custom_mut::<Globals>(GAME_GLOBALS)
                .unwrap();
            globals.update_camera(ctx, self.desired_width, self.desired_height);
            graphics::set_transform_matrix(ctx, globals.camera.as_matrix());
            globals.draw_scene(ctx);
            globals.draw_characters(ctx);
        }
        graphics::reset_transform_matrix(ctx);
        let (width, height) = window::get_size(ctx);
        self.draw_screens(width as Real, height as Real);
        {
            let globals = self
                .vm
                .host_mut()
                .context()
                .custom_mut::<Globals>(GAME_GLOBALS)
                .unwrap();
            globals.draw_commands(ctx);
        }
        Ok(())
    }
}
