use std::{
    path::Path,
    collections::HashMap,
};

use macroquad::{
    texture::Image,
    experimental::animation::AnimatedSprite,
};

use wasm_plugin_host::WasmPlugin;

use crate::item::{ItemType, ItemImplementationRegistry};


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(u64);

#[derive(Clone)]
pub struct PluginDescription {
    plugin_id: PluginId,
    display_name: String,
    items: Vec<ItemDescription>,
}

#[derive(Clone)]
pub struct ItemDescription {
    pub item_type: ItemType,
    pub display_name: String,
    pub image: Image,
    pub sprite: AnimatedSprite,
    pub fx_sprite: AnimatedSprite,
}

struct GameApiContext {
}

pub struct PluginRegistry(HashMap<PluginId, (WasmPlugin, GameApiContext)>);

impl PluginRegistry {
    pub fn load(path: impl AsRef<Path>, item_registry: &mut ItemImplementationRegistry) -> Self {
        let mut plugins = HashMap::new();
        for entry in path.as_ref().read_dir().expect("Unable to read plugins directory") {
            if let Ok(entry) = entry {
                if entry.path().to_str().unwrap().contains(".wasm") {
                    let mut plugin = wasm_plugin_host::WasmPlugin::load(entry.path()).expect(&format!("Failed to load plugin {:?}", entry.path()));
                    let description: PluginDescription = plugin.call_function("plugin_description").expect(&format!("Failed to call 'plugin_description' on plugin {:?}", entry.path()));

                    for item in &description.items {
                        item_registry.add(item, description.plugin_id);
                    }

                    plugins.insert(description.plugin_id, (plugin, GameApiContext {}));
                }
            }
        }

        PluginRegistry(plugins)
    }
}
