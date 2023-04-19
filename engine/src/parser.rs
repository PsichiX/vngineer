use std::collections::HashMap;

use crate::script::*;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct VnScriptParser;

pub fn parse(content: &str) -> Result<VnFile, String> {
    match VnScriptParser::parse(Rule::file, content) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            match pair.as_rule() {
                Rule::file => Ok(parse_file(pair)),
                rule => unreachable!("{:?}", rule),
            }
        }
        Err(error) => Err(format!("{}", error)),
    }
}

fn parse_file(pair: Pair<Rule>) -> VnFile {
    let pairs = pair.into_inner();
    let mut result = VnFile::default();
    for pair in pairs {
        match pair.as_rule() {
            Rule::import => {
                result.dependencies.insert(parse_import(pair));
            }
            Rule::story_item => {
                let pairs = pair.into_inner();
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::config => {
                            let (name, config) = parse_config(pair);
                            result.story.configs.insert(name, config);
                        }
                        Rule::character => {
                            let (name, character) = parse_character(pair);
                            result.story.characters.insert(name, character);
                        }
                        Rule::scene => {
                            let (name, scene) = parse_scene(pair);
                            result.story.scenes.insert(name, scene);
                        }
                        Rule::chapter => {
                            let (name, chapter) = parse_chapter(pair);
                            result.story.chapters.insert(name, chapter);
                        }
                        rule => unreachable!("Unsupported: {:?}", rule),
                    }
                }
            }
            Rule::EOI => {}
            rule => unreachable!("Unsupported: {:?}", rule),
        }
    }
    result
}

fn parse_import(pair: Pair<Rule>) -> String {
    parse_text(pair.into_inner().next().unwrap())
}

fn parse_chapter(pair: Pair<Rule>) -> (String, VnChapter) {
    let mut pairs = pair.into_inner();
    let name = parse_identifier(pairs.next().unwrap());
    let mut result = VnChapter::default();
    for pair in pairs {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::label => {
                result.items.push(VnChapterItem::Label(parse_label(pair)));
            }
            Rule::chapter_action => {
                result
                    .items
                    .push(VnChapterItem::Action(parse_chapter_action(pair)));
            }
            rule => unreachable!("Unsupported: {:?}", rule),
        }
    }
    (name, result)
}

fn parse_label(pair: Pair<Rule>) -> String {
    parse_identifier(pair.into_inner().next().unwrap())
}

fn parse_chapter_action(pair: Pair<Rule>) -> VnAction {
    let mut pairs = pair.into_inner();
    let (name, module_name) = parse_chapter_action_path(pairs.next().unwrap());
    let params = pairs.map(parse_property).collect();
    VnAction {
        name,
        module_name,
        params,
    }
}

fn parse_chapter_action_path(pair: Pair<Rule>) -> (String, Option<String>) {
    let result = pair.into_inner().map(parse_identifier).collect::<Vec<_>>();
    if result.len() == 1 {
        (result[0].to_owned(), None)
    } else {
        (result[1].to_owned(), Some(result[0].to_owned()))
    }
}

fn parse_scene(pair: Pair<Rule>) -> (String, VnScene) {
    let mut pairs = pair.into_inner();
    let name = parse_identifier(pairs.next().unwrap());
    let properties = pairs.map(parse_property).collect();
    (name, VnScene { properties })
}

fn parse_character(pair: Pair<Rule>) -> (String, VnCharacter) {
    let mut pairs = pair.into_inner();
    let name = parse_identifier(pairs.next().unwrap());
    let properties = pairs.map(parse_property).collect();
    (name, VnCharacter { properties })
}

fn parse_config(pair: Pair<Rule>) -> (String, VnConfig) {
    let mut pairs = pair.into_inner();
    let name = parse_identifier(pairs.next().unwrap());
    let properties = pairs.map(parse_property).collect();
    (name, VnConfig { properties })
}

fn parse_property(pair: Pair<Rule>) -> (String, VnValue) {
    let mut pairs = pair.into_inner();
    let name = parse_identifier(pairs.next().unwrap());
    let value = parse_value(pairs.next().unwrap());
    (name, value)
}

fn parse_value(pair: Pair<Rule>) -> VnValue {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::none => VnValue::None,
        Rule::text => VnValue::Text(parse_text(pair)),
        Rule::number => VnValue::Number(parse_number(pair)),
        Rule::color => VnValue::Color(parse_color(pair)),
        Rule::bool_true => VnValue::Boolean(true),
        Rule::bool_false => VnValue::Boolean(false),
        Rule::map => VnValue::Map(parse_map(pair)),
        Rule::array => VnValue::Array(parse_array(pair)),
        Rule::identifier => VnValue::Text(parse_identifier(pair)),
        rule => unreachable!("Unsupported: {:?}", rule),
    }
}

fn parse_array(pair: Pair<Rule>) -> Vec<VnValue> {
    pair.into_inner().map(parse_value).collect()
}

fn parse_map(pair: Pair<Rule>) -> HashMap<String, VnValue> {
    pair.into_inner().map(parse_property).collect()
}

fn parse_color(pair: Pair<Rule>) -> u32 {
    u32::from_str_radix(pair.as_str(), 16).unwrap()
}

fn parse_number(pair: Pair<Rule>) -> f64 {
    pair.as_str().parse::<f64>().unwrap()
}

fn parse_text(pair: Pair<Rule>) -> String {
    snailquote::unescape(pair.as_str()).unwrap()
}

fn parse_identifier(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let content = std::fs::read_to_string("../resources/main.vns").unwrap();
        parse(&content).unwrap();
    }
}
