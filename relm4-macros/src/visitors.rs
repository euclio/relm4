use syn::{Ident, ImplItemType, ImplItemMethod, Macro, visit::{self, Visit}};

#[derive(Default)]
pub struct ComponentTypes<'ast> {
    pub widget_type_item: Option<&'ast ImplItemType>,
    pub other_type_items: Vec<&'ast ImplItemType>,
}

impl<'ast> Visit<'ast> for ComponentTypes<'ast> {
    fn visit_impl_item_type(&mut self, ty: &'ast syn::ImplItemType) {
        if &ty.ident == "Widgets" {
            self.widget_type_item = Some(ty);
        } else {
            self.other_type_items.push(ty);
        }

        visit::visit_impl_item_type(self, ty);
    }
}

#[derive(Default)]
pub struct ComponentMacros<'ast> {
    pub view_widgets: Option<&'ast Macro>,
    pub additional_fields: Option<&'ast Macro>,
    pub menu: Option<&'ast Macro>,
    pub errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for ComponentMacros<'ast> {
    fn visit_impl_item_macro(&mut self, mac: &'ast syn::ImplItemMacro) {
        match mac.mac.path.get_ident().map(Ident::to_string).as_deref() {
            Some("view") => {
                if self.view_widgets.is_some() {
                    self.errors.push(syn::Error::new_spanned(mac, "duplicate view macro"));
                }
                self.view_widgets = Some(&mac.mac)
            }
            Some("additional_fields") => {
                if self.additional_fields.is_some() {
                    self.errors.push(syn::Error::new_spanned(mac, "duplicate additional_fields macro"));
                }
                self.additional_fields = Some(&mac.mac)
            }
            Some("menu") => {
                if self.menu.is_some() {
                    self.errors.push(syn::Error::new_spanned(mac, "duplicate menu macro"));
                }
                self.menu = Some(&mac.mac)
            }
        }

        visit::visit_impl_item_macro(self, mac);
    }
}

#[derive(Default)]
pub struct ComponentFunctions<'ast> {
    pub init: Option<&'ast ImplItemMethod>,
    pub pre_view: Option<&'ast ImplItemMethod>,
    pub post_view: Option<&'ast ImplItemMethod>,
    pub other_functions: Vec<&'ast ImplItemMethod>,
    pub errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for ComponentFunctions<'ast> {
    fn visit_impl_item_method(&mut self, func: &'ast ImplItemMethod) {
        match &*func.sig.ident.to_string() {
            "init" => {
                if self.init.is_some() {
                    self.errors.push(syn::Error::new_spanned(func, "duplicate init function"));
                }
                self.init = Some(func);
            }
            "pre_view" => {
                if self.pre_view.is_some() {
                    self.errors.push(syn::Error::new_spanned(func, "duplicate pre_view function"));
                }
                self.pre_view = Some(func);
            }
            "post_view" => {
                if self.post_view.is_some() {
                    self.errors.push(syn::Error::new_spanned(func, "duplicate post_view function"));
                }
                self.post_view = Some(func);
            }
            _ => self.other_functions.push(func),
        }

        visit::visit_impl_item_method(self, func);
    }
}
