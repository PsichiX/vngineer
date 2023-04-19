// NOTE: refactor and cleanup this entire shitty application!
// It's ok for a prototype, definitely not for production.

mod game_state;
mod library;

use crate::game_state::GameState;
use clap::Parser;
use intuicio_essentials::prelude::*;
use intuicio_frontend_simpleton::prelude::*;
use std::path::PathBuf;
use tetra::{time::Timestep, ContextBuilder};
use vngineer_core::prelude::*;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input cartridge file path.
    #[arg(value_name = "PATH")]
    entry: String,
}

fn main() -> tetra::Result {
    let cli = Cli::parse();
    let mut root = PathBuf::from(&cli.entry);
    root.pop();
    let root = root.to_string_lossy().to_string();

    let mut registry = Registry::default();
    vngineer_core::library::install(&mut registry);
    intuicio_frontend_simpleton::library::install(&mut registry);
    vngineer_simpleton::install(&mut registry);
    crate::library::install(&mut registry);

    let mut vn_content_provider = ExtensionContentProvider::<VnFile>::default()
        .extension("vns", FileContentProvider::new("vns", VnContentParser))
        .extension("plugin", IgnoreContentProvider)
        .extension("simp", IgnoreContentProvider)
        .default_extension("vns");
    let vn_package = VnPackage::new(&cli.entry, &mut vn_content_provider).unwrap();
    vn_package.install_plugins(&mut registry, &[root.as_str()]);

    let mut simpleton_content_provider = ExtensionContentProvider::<SimpletonModule>::default()
        .extension(
            "simp",
            FileContentProvider::new("simp", SimpletonContentParser),
        )
        .extension("plugin", IgnoreContentProvider)
        .default_extension("simp");
    for (path, file) in &vn_package.files {
        let mut directory = PathBuf::from(path);
        directory.pop();
        for path in &file.dependencies {
            let entry = directory.join(path).to_string_lossy().to_string();
            let simpleton_package =
                SimpletonPackage::new(&entry, &mut simpleton_content_provider).unwrap();
            simpleton_package.install_plugins(&mut registry, &[root.as_str()]);
            simpleton_package
                .compile()
                .install::<VmScope<SimpletonScriptExpression>>(&mut registry, None);
        }
    }

    let story = vn_package.compile();
    let host = Host::new(Context::new(10240, 10240, 0), registry.into());
    let mut vm = Vm::new(host);
    vm.add_story(&story);

    let (title, width, height, desired_width, desired_height, fullscreen, fps, entry) = story
        .configs
        .get("application")
        .map(|config| {
            let title = config
                .properties
                .get("title")
                .map(|title| title.as_text().expect("`title` is not a text!"))
                .unwrap_or("vngineer");
            let width = config
                .properties
                .get("width")
                .map(|width| width.as_number().expect("`width` is not a number!") as _)
                .unwrap_or(1024);
            let height = config
                .properties
                .get("height")
                .map(|height| height.as_number().expect("`height` is not a number!") as _)
                .unwrap_or(768);
            let desired_width = config
                .properties
                .get("desired_width")
                .map(|desired_width| {
                    desired_width
                        .as_number()
                        .expect("`desired_width` is not a number!") as f32
                })
                .unwrap_or(1024.0);
            let desired_height = config
                .properties
                .get("desired_height")
                .map(|desired_height| {
                    desired_height
                        .as_number()
                        .expect("`desired_height` is not a number!") as f32
                })
                .unwrap_or(768.0);
            let fullscreen = config
                .properties
                .get("fullscreen")
                .map(|fullscreen| {
                    fullscreen
                        .as_boolean()
                        .expect("`fullscreen` is not a number!")
                })
                .unwrap_or(false);
            let fps = config
                .properties
                .get("fps")
                .map(|fps| fps.as_number().expect("`fps` is not a number!"))
                .unwrap_or(30.0);
            let entry = config
                .properties
                .get("entry")
                .map(|entry| entry.as_text().expect("`entry` is not a text!"))
                .unwrap_or("start")
                .to_owned();
            (
                title,
                width,
                height,
                desired_width,
                desired_height,
                fullscreen,
                fps,
                entry,
            )
        })
        .unwrap_or_else(|| {
            (
                "vngineer",
                1024,
                768,
                1024.0,
                768.0,
                false,
                30.0,
                "start".to_owned(),
            )
        });

    let mut root = PathBuf::from(&cli.entry);
    root.pop();
    let _ = std::env::set_current_dir(root);

    ContextBuilder::new(title, width, height)
        .fullscreen(fullscreen)
        .show_mouse(true)
        .quit_on_escape(false)
        .resizable(true)
        .timestep(Timestep::Fixed(fps))
        .build()?
        .run(|_| {
            Ok(GameState::new(
                vm,
                story,
                &entry,
                desired_width,
                desired_height,
            ))
        })
}
