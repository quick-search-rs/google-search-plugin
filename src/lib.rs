use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr, RString, RVec},
};
use quick_search_lib::{ColoredChar, Log, PluginId, SearchLib, SearchLib_Ref, SearchResult, Searchable, Searchable_TO};

static NAME: &str = "Google-Search";

#[export_root_module]
pub fn get_library() -> SearchLib_Ref {
    SearchLib { get_searchable }.leak_into_prefix()
}

#[sabi_extern_fn]
fn get_searchable(id: PluginId, logger: quick_search_lib::ScopedLogger) -> Searchable_TO<'static, RBox<()>> {
    let this = Google::new(id, logger);
    Searchable_TO::from_value(this, TD_Opaque)
}

struct Google {
    id: PluginId,
    client: reqwest::blocking::Client,
    config: quick_search_lib::Config,
    logger: quick_search_lib::ScopedLogger,
}

impl Google {
    fn new(id: PluginId, logger: quick_search_lib::ScopedLogger) -> Self {
        Self {
            id,
            client: reqwest::blocking::Client::new(),
            config: default_config(),
            logger,
        }
    }
}

impl Searchable for Google {
    fn search(&self, query: RString) -> RVec<SearchResult> {
        let mut res: Vec<SearchResult> = vec![];

        let url = format!("https://google.com/complete/search?q={}&output=toolbar&hl=en", urlencoding::encode(query.as_str()));

        if let Ok(response) = self.client.get(url).send() {
            if let Ok(text) = response.text() {
                if let Ok(xml) = roxmltree::Document::parse(&text) {
                    for node in xml.descendants() {
                        if node.tag_name().name() == "suggestion" {
                            if let Some(data) = node.attribute("data") {
                                let data = data.to_string();
                                res.push(SearchResult::new(&data));
                            }
                        }
                    }
                }
            }
        }

        res.sort_by(|a, b| a.title().cmp(b.title()));
        res.dedup_by(|a, b| a.title() == b.title());
        if self.config.get("Always return query even if no results found").and_then(|e| e.as_bool()).unwrap_or(true) {
            res.retain(|r| r.title() != query);
            res.insert(0, SearchResult::new(&query));
        }

        res.into()
    }
    fn name(&self) -> RStr<'static> {
        NAME.into()
    }
    fn colored_name(&self) -> RVec<quick_search_lib::ColoredChar> {
        // can be dynamic although it's iffy how it might be used
        vec![
            ColoredChar::new_rgba('G', 66, 133, 244, 255),
            ColoredChar::new_rgba('o', 219, 68, 55, 255),
            ColoredChar::new_rgba('o', 244, 180, 0, 255),
            ColoredChar::new_rgba('g', 66, 133, 244, 255),
            ColoredChar::new_rgba('l', 15, 157, 88, 255),
            ColoredChar::new_rgba('e', 219, 68, 55, 255),
        ]
        .into()
    }
    fn execute(&self, result: &SearchResult) {
        // let s = result.extra_info();
        // if let Ok::<clipboard::ClipboardContext, Box<dyn std::error::Error>>(mut clipboard) = clipboard::ClipboardProvider::new() {
        //     if let Ok(()) = clipboard::ClipboardProvider::set_contents(&mut clipboard, s.to_owned()) {
        //         println!("copied to clipboard: {}", s);
        //     } else {
        //         println!("failed to copy to clipboard: {}", s);
        //     }
        // } else {
        //     log::error!("failed to copy to clipboard: {}", s);
        // }

        // finish up, above is a clipboard example

        if let Err(e) = webbrowser::open(&format!("https://google.com/search?q={}", urlencoding::encode(result.title()))) {
            self.logger.error(&format!("failed to open browser: {}", e));
        }
    }
    fn plugin_id(&self) -> PluginId {
        self.id.clone()
    }
    fn get_config_entries(&self) -> quick_search_lib::Config {
        default_config()
    }
    fn lazy_load_config(&mut self, config: quick_search_lib::Config) {
        self.config = config;
    }
}

fn default_config() -> quick_search_lib::Config {
    let mut config = quick_search_lib::Config::new();
    config.insert("Always return query even if no results found".into(), quick_search_lib::EntryType::Bool { value: true });
    config
}
