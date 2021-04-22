use plugin_api::{ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId};

#[wasm_plugin_guest::export_function]
fn plugin_description() -> PluginDescription {
    PluginDescription {
        plugin_id: PluginId(11229058760733382699),
        display_name: "basic weapons".to_string(),
        items: vec![],
    }
}
