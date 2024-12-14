mod available_experiences_plugins;
mod render_plugins;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn generate_available_experiences_plugins(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    available_experiences_plugins::generate_available_experiences_plugins(item)
}

#[proc_macro_attribute]
pub fn generate_render_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    render_plugins::generate_render_plugins(item)
}
