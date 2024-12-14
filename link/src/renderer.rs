use std::collections::HashMap;

use server_api::plugin::PluginRenderer;

#[experiences_link_proc_macro::generate_available_experiences_plugins]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AvailableExperiencesPlugins {}

#[experiences_link_proc_macro::generate_render_plugins]
pub struct PluginRenderers {
    pub renderers: HashMap<AvailableExperiencesPlugins, Box<dyn PluginRenderer>>,
}
