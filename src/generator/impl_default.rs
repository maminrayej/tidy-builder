use quote::quote;

use super::Generator;

impl<'a> Generator<'a> {
    /// Generate Default trait impl if there are no required fields
    pub fn default_trait(&self) -> Vec<proc_macro2::TokenStream> {
        if self.req_fields.len() > 0 {
            return vec![];
        }

        let impl_generics = &self.impl_generics;
        let s_ident = &self.s_ident;
        let ty_generics = &self.ty_generics;

        vec![quote! {
            impl #impl_generics Default for #s_ident #ty_generics {
                fn default() -> Self {
                    Self::builder().build()
                }
            }
        }]
    }
}
