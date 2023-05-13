use crate::parser;
use intuicio_essentials::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum VnValue {
    #[default]
    None,
    Boolean(bool),
    Number(f64),
    Text(String),
    Color(u32),
    Array(Vec<VnValue>),
    Map(HashMap<String, VnValue>),
}

impl VnValue {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Self::Boolean(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_color(&self) -> Option<u32> {
        if let Self::Color(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&[Self]> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Self>> {
        if let Self::Map(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn is_same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::None, Self::None)
                | (Self::Boolean(_), Self::Boolean(_))
                | (Self::Number(_), Self::Number(_))
                | (Self::Text(_), Self::Text(_))
                | (Self::Color(_), Self::Color(_))
                | (Self::Array(_), Self::Array(_))
                | (Self::Map(_), Self::Map(_))
        )
    }
}

#[derive(Default, Clone)]
pub enum VnResult {
    #[default]
    Continue,
    JumpTo {
        chapter: Option<String>,
        label: Option<String>,
    },
    Enter {
        chapter: Option<String>,
        label: Option<String>,
    },
    Exit,
}

#[derive(Debug, Default)]
pub struct VnFile {
    pub dependencies: HashSet<String>,
    pub story: VnStory,
}

impl VnFile {
    pub fn parse(content: &str) -> Result<Self, String> {
        parser::parse(content)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnStory {
    pub configs: HashMap<String, VnConfig>,
    pub characters: HashMap<String, VnCharacter>,
    pub scenes: HashMap<String, VnScene>,
    pub chapters: HashMap<String, VnChapter>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnConfig {
    pub properties: HashMap<String, VnValue>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnCharacter {
    pub properties: HashMap<String, VnValue>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnScene {
    pub properties: HashMap<String, VnValue>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VnChapter {
    pub items: Vec<VnChapterItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VnChapterItem {
    Label(String),
    Action(VnAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VnAction {
    pub name: String,
    pub module_name: Option<String>,
    pub params: HashMap<String, VnValue>,
}

impl VnAction {
    pub fn evaluate(&self, context: &mut Context, registry: &Registry) -> VnResult {
        let function = registry
            .find_function(FunctionQuery {
                name: Some(self.name.as_str().into()),
                module_name: self.module_name.as_ref().map(|name| name.into()),
                ..Default::default()
            })
            .unwrap_or_else(|| {
                panic!(
                    "Function {}::{} not found in registry!",
                    self.module_name.as_deref().unwrap_or(""),
                    self.name
                )
            });
        let value_type = TypeHash::of::<VnValue>();
        if function.signature().outputs.len() == 1 {
            let param = &function.signature().outputs[0];
            if param.struct_handle.type_hash() != TypeHash::of::<VnResult>() {
                panic!(
                    "Function {}::{} output `{}` is not `VnResult`!",
                    self.module_name.as_deref().unwrap_or(""),
                    self.name,
                    param.name
                );
            }
        } else {
            panic!(
                "Function {}::{} should have one `VnResult` output!",
                self.module_name.as_deref().unwrap_or(""),
                self.name
            );
        }
        for param in function.signature().inputs.iter().rev() {
            if param.struct_handle.type_hash() != value_type {
                panic!(
                    "Function {}::{} input `{}` is not `VnValue`!",
                    self.module_name.as_deref().unwrap_or(""),
                    self.name,
                    param.name
                );
            }
            if let Some(value) = self.params.get(&param.name) {
                context.stack().push(value.clone());
            } else {
                context.stack().push(VnValue::None);
            }
        }
        function.invoke(context, registry);
        context.stack().pop::<VnResult>().unwrap()
    }
}

#[derive(Debug, Default)]
pub struct VnPackage {
    pub files: HashMap<String, VnFile>,
}

impl VnPackage {
    pub fn new<CP>(path: &str, content_provider: &mut CP) -> Result<Self, Box<dyn Error>>
    where
        CP: ScriptContentProvider<VnFile>,
    {
        let mut result = Self::default();
        result.load(path, content_provider)?;
        Ok(result)
    }

    pub fn load<CP>(&mut self, path: &str, content_provider: &mut CP) -> Result<(), Box<dyn Error>>
    where
        CP: ScriptContentProvider<VnFile>,
    {
        let path = content_provider.sanitize_path(path)?;
        if self.files.contains_key(&path) {
            return Ok(());
        }
        for content in content_provider.unpack_load(&path)? {
            if let Some(module) = content.data? {
                let dependencies = module.dependencies.to_owned();
                self.files.insert(content.name, module);
                for relative in dependencies {
                    let path = content_provider.join_paths(&content.path, &relative)?;
                    self.load(&path, content_provider)?;
                }
            }
        }
        Ok(())
    }

    pub fn compile(self) -> VnStory {
        let mut result = VnStory::default();
        for file in self.files.into_values() {
            result.configs.extend(file.story.configs);
            result.characters.extend(file.story.characters);
            result.scenes.extend(file.story.scenes);
            result.chapters.extend(file.story.chapters);
        }
        result
    }

    #[cfg(feature = "plugins")]
    pub fn install_plugins(&self, registry: &mut Registry, search_paths: &[&str]) {
        use intuicio_essentials::{core::core_version, prelude::*};
        use std::{env::consts::DLL_EXTENSION, path::PathBuf};

        for file in self.files.values() {
            'plugin: for path in &file.dependencies {
                let mut path = PathBuf::from(path);
                if path
                    .extension()
                    .map(|extension| extension == "plugin")
                    .unwrap_or_default()
                {
                    path.set_extension(DLL_EXTENSION);
                    for search_path in search_paths {
                        let path = PathBuf::from(search_path).join(&path);
                        if install_plugin(
                            path.to_string_lossy().as_ref(),
                            registry,
                            Some(core_version()),
                        )
                        .is_ok()
                        {
                            continue 'plugin;
                        }
                    }
                    panic!("Could not load plugin: {:?}", path);
                }
            }
        }
    }
}

pub struct VnContentParser;

impl BytesContentParser<VnFile> for VnContentParser {
    fn parse(&self, bytes: Vec<u8>) -> Result<VnFile, Box<dyn Error>> {
        let content = String::from_utf8(bytes)?;
        Ok(VnFile::parse(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script() {
        let mut content_provider = ExtensionContentProvider::<VnFile>::default()
            .extension("vns", FileContentProvider::new("vns", VnContentParser))
            .extension("plugin", IgnoreContentProvider)
            .extension("simp", IgnoreContentProvider)
            .default_extension("vns");
        VnPackage::new("../resources/main.vns", &mut content_provider)
            .unwrap()
            .compile();
    }
}
