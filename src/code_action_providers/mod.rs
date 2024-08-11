use std::{collections::HashMap, path::PathBuf, sync::Arc};

use lua::provider::LuaProvider;
use tower_lsp::lsp_types::CodeAction;
use traits::ActionProvider;
use yaml::{config, provider::YamlProvider};

use crate::{
    llm_handlers::traits::Llm,
    nonsense::{self, IndexedText, TextAdapter},
    read_language_config_files,
};

pub mod helper;
pub mod lua;
pub mod parsed_document;
pub mod traits;
pub mod yaml;

const SUPPORTED_LANGUAGES: [&str; 7] = [
    "gitcommit",
    "go",
    "markdown",
    "python",
    "rust",
    "text",
    "__all__",
];

pub fn load_providers(
    code_actions_config_dir: PathBuf,
    prompt_handler: Arc<Llm>,
) -> HashMap<String, Vec<Box<dyn ActionProvider>>> {
    let mut providers: HashMap<String, Vec<Box<dyn ActionProvider>>> = Default::default();

    //log::info!("Processing  config-dir: {:?}", config_dir);
    for language in SUPPORTED_LANGUAGES {
        let config_dir = code_actions_config_dir.join(language);
        for config_path in read_language_config_files(&config_dir, "yaml") {
            //log::info!("Processing language config: {:?}", config_path);
            match config::CodeActionConfig::from_yaml(&config_path) {
                Ok(language_config) => {
                    for (k, config) in language_config.code_actions.into_iter().enumerate() {
                        //log::info!("Register action {} for {:?}", config.name, config_path);
                        providers
                            .entry(language.to_owned())
                            .or_default()
                            .push(Box::new(YamlProvider::from_config(
                                config,
                                &format!("{}.{k}", config_path.to_string_lossy()),
                                prompt_handler.clone(),
                            )));
                    }
                }
                Err(_e) => {
                    //log::warn!("Cannot read {:?} because of {}", &config_path, e);
                }
            };
        }
        for config_path in read_language_config_files(&config_dir, "lua") {
            //log::info!("Processing language config: {:?}", config_path);
            providers
                .entry(language.to_owned())
                .or_default()
                .push(Box::new(
                    LuaProvider::try_new(&config_path.to_string_lossy(), prompt_handler.clone())
                        .unwrap(),
                ));
        }
    }
    providers
}

pub fn find_resolver<'a>(
    providers: &'a HashMap<String, Vec<Box<dyn ActionProvider>>>,
    code_action_id: &str,
    lang: &str,
) -> Option<&'a Box<dyn ActionProvider>> {
    for target_lang in [lang, "__all__"] {
        if let Some(language_specific_providers) = providers.get(target_lang) {
            for provider in language_specific_providers.iter() {
                if provider.can_handle(code_action_id) {
                    return Some(provider);
                }
            }
        }
    }
    None
}

pub fn map_to_lsp(r: &mut CodeAction, index: &IndexedText<String>) {
    // if let Ok(r) = r.as_mut() {
    if let Some(e) = r.edit.as_mut() {
        if let Some(c) = e.changes.as_mut() {
            for value in c.values_mut() {
                for text_edit in value.iter_mut() {
                    let fake = std::ops::Range::<nonsense::Pos> {
                        start: nonsense::Pos {
                            line: text_edit.range.start.line,
                            col: text_edit.range.start.character,
                        },
                        end: nonsense::Pos {
                            line: text_edit.range.end.line,
                            col: text_edit.range.end.character,
                        },
                    };
                    let rs = index.range_to_lsp_range(&fake).unwrap();
                    text_edit.range = rs;
                }
            }
        }
    }
    // }
}
