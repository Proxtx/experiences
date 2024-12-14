use crate::available_experiences_plugins::get_plugins;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn generate_render_plugins(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let struct_name = &input.ident;
    let generics = &input.generics;

    let plugins = get_plugins()
        .into_iter()
        .map(|v| (v.clone(), format!("{}", v)))
        .collect::<Vec<_>>();
    let av_idents = plugins.iter().map(|v| {
        let ident = Ident::new(&v.0, Span::call_site());
        quote! {
            #ident
        }
    });

    let im_idents = plugins.iter().map(|v| {
        let ident = Ident::new(&v.1, Span::call_site());
        quote! {
            #ident
        }
    });

    let expaned = quote! {
        #input

        impl #generics #struct_name #generics {
            pub async fn init() -> #struct_name #generics {
                #struct_name {
                    renderers: HashMap::from([#((AvailableExperiencesPlugins::#av_idents, Box::new(#im_idents::PluginRenderer::new().await) as Box<dyn PluginRenderer>),)*]),
                }
            }
        }
    };

    TokenStream::from(expaned)
}
