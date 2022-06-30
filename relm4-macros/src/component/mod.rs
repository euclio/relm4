use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Error, PathArguments, Visibility};
use syn::visit::Visit;

use crate::menu::Menus;
use crate::visitors::{ComponentMacros, ComponentTypes};

mod funcs;
pub(super) mod inject_view_code;
pub(crate) mod token_streams;

use inject_view_code::inject_view_code;

pub(crate) fn generate_tokens(vis: Option<Visibility>, data: syn::ItemImpl) -> TokenStream2 {
    let mut component_types = ComponentTypes::default();
    component_types.visit_item_impl(&data);
    let ComponentTypes { widget_type_item, other_type_items } = component_types;
    let widgets_type = widget_type_item.map(|def| &def.ty);

    let trait_ = &data.trait_.as_ref().unwrap().1;
    let ty = &data.self_ty;
    let outer_attrs = &data.attrs;

    let mut component_macros = ComponentMacros::default();
    component_macros.visit_item_impl(&data);

    if let Some((first, rest)) = component_macros.errors.split_first_mut() {
        for err in rest {
            first.combine(*err);
        }

        return first.into_compile_error();
    }

    let ComponentMacros {
        view_widgets,
        additional_fields,
        menu,
        ..
    } = component_macros;

    // Generate menu tokens
    let menus_stream = menu.map(|menu| {
        menu.parse_body::<Menus>()
            .map(|menus| menus.menus_stream())
            .unwrap_or_else(|e| e.to_compile_error())
    });

    let funcs = data.items.iter().filter_map(|item| {
        match item {
            syn::ImplItem::Method(func) => Some(func.clone()),
            _ => None,
        }
    }).collect::<Vec<_>>();
    let funcs::Funcs {
        init,
        pre_view,
        post_view,
        unhandled_fns,
        root_name,
        model_name,
    } = match funcs::Funcs::new(funcs) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    let token_streams::TokenStreams {
        error,
        init_root,
        rename_root,
        struct_fields,
        init: init_widgets,
        assign,
        connect,
        return_fields,
        destructure_fields,
        update_view,
    } = view_widgets.generate_streams(&vis, &model_name, Some(&root_name), false);

    let root_widget_type = view_widgets.root_type();

    let impl_generics = &data.generics;
    let where_clause = &data.generics.where_clause;

    // Extract identifiers from additional fields for struct initialization: "test: u8" => "test"
    let additional_fields_return_stream = if let Some(fields) = &additional_fields {
        let mut tokens = TokenStream2::new();
        for field in fields.inner.pairs() {
            tokens.extend(field.value().ident.to_token_stream());
            tokens.extend(quote! {,});
        }
        tokens
    } else {
        TokenStream2::new()
    };

    let view_code = quote! {
        #rename_root
        #menus_stream
        #init_widgets
        #connect
        {
            #error
        }
        #assign
    };

    let widgets_return_code = quote! {
        Self::Widgets {
            #return_fields
            #additional_fields_return_stream
        }
    };

    let init_injected = match inject_view_code(init, view_code, widgets_return_code) {
        Ok(method) => method,
        Err(err) => return err.to_compile_error(),
    };

    quote! {
        #[allow(dead_code)]
        #(#outer_attrs)*
        #[derive(Debug)]
        #vis struct #widgets_type {
            #struct_fields
            #additional_fields
        }

        impl #impl_generics #trait_ for #ty #where_clause {
            type Root = #root_widget_type;
            #widget_type_item

            #(#other_type_items)*

            fn init_root() -> Self::Root {
                #init_root
            }

            #init_injected

            /// Update the view to represent the updated model.
            fn update_view(
                &self,
                widgets: &mut Self::Widgets,
                sender: &ComponentSender<Self>,
            ) {
                #[allow(unused_variables)]
                let Self::Widgets {
                    #destructure_fields
                    #additional_fields_return_stream
                } = widgets;

                #[allow(unused_variables)]
                let #model_name = self;

                #pre_view
                #update_view
                (|| { #post_view })();
            }

            #(#unhandled_fns)*
        }
    }
}
