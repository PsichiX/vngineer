use crate::script::{VnChapter, VnChapterItem, VnResult, VnStory, VnValue};
use intuicio_essentials::prelude::*;
use std::collections::HashMap;

pub const VN_GLOBALS: &str = "vn-globals";

#[derive(Debug, Default)]
pub struct Globals {
    pub properties: HashMap<String, VnValue>,
}

struct State {
    chapter: String,
    position: usize,
}

pub struct Vm {
    host: Host,
    chapters: HashMap<String, VnChapter>,
    state: Vec<State>,
}

impl Vm {
    pub fn new(mut host: Host) -> Self {
        host.context().set_custom(VN_GLOBALS, Globals::default());
        Self {
            host,
            chapters: Default::default(),
            state: vec![],
        }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    pub fn enter(&mut self, chapter_name: &str, label: Option<&str>) -> bool {
        if let Some(chapter) = self.chapters.get(chapter_name) {
            let mut position = 0;
            if let Some(label) = label {
                let index = chapter.items.iter().position(|item| {
                    if let VnChapterItem::Label(name) = item {
                        name == label
                    } else {
                        false
                    }
                });
                if let Some(index) = index {
                    position = index;
                }
            }
            self.state.push(State {
                chapter: chapter_name.to_owned(),
                position,
            });
            true
        } else {
            false
        }
    }

    pub fn exit(&mut self) {
        self.state.pop();
        if let Some(state) = self.state.last_mut() {
            state.position += 1;
        }
    }

    pub fn is_running(&self) -> bool {
        !self.state.is_empty()
    }

    pub fn chapters(&self) -> impl Iterator<Item = (&str, &VnChapter)> {
        self.chapters
            .iter()
            .map(|(name, chapter)| (name.as_str(), chapter))
    }

    pub fn add_story(&mut self, story: &VnStory) {
        for (name, chapter) in &story.chapters {
            self.add_chapter(name, chapter.clone());
        }
    }

    pub fn add_chapter(&mut self, name: impl ToString, chapter: VnChapter) {
        self.chapters.insert(name.to_string(), chapter);
    }

    pub fn remove_chapter(&mut self, name: &str) -> Option<VnChapter> {
        self.chapters.remove(name)
    }

    pub fn remove_chapters(&mut self, mut f: impl FnMut(&str, &VnChapter) -> bool) {
        let to_remove = self
            .chapters
            .iter()
            .filter(|(name, chapter)| f(name.as_str(), chapter))
            .map(|(name, _)| name.to_owned())
            .collect::<Vec<_>>();
        for name in to_remove {
            self.chapters.remove(&name);
        }
    }

    pub fn step(&mut self) {
        let state = match self.state.last_mut() {
            Some(state) => state,
            None => return,
        };
        let chapter = match self.chapters.get(&state.chapter) {
            Some(chapter) => chapter,
            None => {
                self.state.pop();
                return;
            }
        };
        let item = match chapter.items.get(state.position) {
            Some(item) => item,
            None => {
                self.state.pop();
                return;
            }
        };
        match item {
            VnChapterItem::Label(_) => {
                state.position += 1;
            }
            VnChapterItem::Action(action) => {
                let (context, registry) = self.host.context_and_registry();
                match action.evaluate(context, registry) {
                    VnResult::Continue => {
                        state.position += 1;
                    }
                    VnResult::JumpTo {
                        chapter: chapter_name,
                        label,
                    } => {
                        let chapter_name = chapter_name.as_deref().unwrap_or(&state.chapter);
                        if let Some(chapter) = self.chapters.get(chapter_name) {
                            state.chapter = chapter_name.to_owned();
                            state.position = 0;
                            if let Some(label) = label {
                                let index = chapter.items.iter().position(|item| {
                                    if let VnChapterItem::Label(name) = item {
                                        name == &label
                                    } else {
                                        false
                                    }
                                });
                                if let Some(index) = index {
                                    state.position = index;
                                }
                            }
                        } else {
                            state.position += 1;
                        }
                    }
                    VnResult::Enter {
                        chapter: chapter_name,
                        label,
                    } => {
                        let chapter_name =
                            chapter_name.as_deref().unwrap_or(&state.chapter).to_owned();
                        if let Some(chapter) = self.chapters.get(&chapter_name) {
                            let mut position = 0;
                            if let Some(label) = label {
                                let index = chapter.items.iter().position(|item| {
                                    if let VnChapterItem::Label(name) = item {
                                        name == &label
                                    } else {
                                        false
                                    }
                                });
                                if let Some(index) = index {
                                    position = index;
                                }
                            }
                            self.state.push(State {
                                chapter: chapter_name,
                                position,
                            });
                        } else {
                            state.position += 1;
                        }
                    }
                    VnResult::Exit => {
                        self.state.pop();
                        if let Some(state) = self.state.last_mut() {
                            state.position += 1;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::*;
    use intuicio_essentials::core as intuicio_core;
    use intuicio_essentials::data as intuicio_data;

    #[intuicio_function()]
    fn show_screen(name: VnValue, module_name: VnValue) -> VnResult {
        println!(
            "show_screen | name: {:?} | module_name: {:?}",
            name, module_name
        );
        VnResult::Continue
    }

    #[intuicio_function()]
    fn hide_screen(name: VnValue, module_name: VnValue) -> VnResult {
        println!(
            "hide_screen | name: {:?} | module_name: {:?}",
            name, module_name
        );
        VnResult::Continue
    }

    #[intuicio_function()]
    fn scene(name: VnValue) -> VnResult {
        println!("scene | name: {:?}", name);
        VnResult::Continue
    }

    #[intuicio_function()]
    fn show(character: VnValue, variant: VnValue) -> VnResult {
        println!("show | character: {:?} | variant: {:?}", character, variant);
        VnResult::Continue
    }

    #[intuicio_function()]
    fn hide(character: VnValue) -> VnResult {
        println!("show | character: {:?}", character);
        VnResult::Continue
    }

    #[intuicio_function()]
    fn say(what: VnValue, who: VnValue, choices: VnValue) -> VnResult {
        println!(
            "say | what: {:?} | who: {:?} | choices: {:?}",
            what, who, choices
        );
        VnResult::Continue
    }

    fn install(registry: &mut Registry) {
        registry.add_function(show_screen::define_function(registry));
        registry.add_function(hide_screen::define_function(registry));
        registry.add_function(scene::define_function(registry));
        registry.add_function(show::define_function(registry));
        registry.add_function(hide::define_function(registry));
        registry.add_function(say::define_function(registry));
    }

    #[test]
    fn test_vm() {
        let mut content_provider = ExtensionContentProvider::<VnFile>::default()
            .extension("vns", FileContentProvider::new("vns", VnContentParser))
            .extension("plugin", IgnoreContentProvider)
            .extension("simp", IgnoreContentProvider)
            .default_extension("vns");
        let story = VnPackage::new("../resources/main.vns", &mut content_provider)
            .unwrap()
            .compile();
        let mut registry = Registry::default().with_basic_types();
        crate::library::install(&mut registry);
        install(&mut registry);
        let host = Host::new(Context::new(1024, 1024, 1024), registry.into());
        let mut vm = Vm::new(host);

        vm.add_story(&story);
        vm.enter("welcome", None);
        while vm.is_running() {
            vm.step();
        }
    }
}
