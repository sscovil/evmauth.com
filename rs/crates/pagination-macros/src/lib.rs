use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Attribute macro that adds pagination fields following the Relay spec.
///
/// Adds four optional fields following Relay GraphQL Cursor Connections Specification:
/// - `first: Option<i64>` - Number of items for forward pagination
/// - `after: Option<String>` - Cursor for forward pagination
/// - `last: Option<i64>` - Number of items for backward pagination
/// - `before: Option<String>` - Cursor for backward pagination
///
/// Also adds a `to_page()` method that converts these fields to a `pagination::Page`.
#[proc_macro_attribute]
pub fn with_pagination(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    // Only process structs
    let Data::Struct(ref mut data_struct) = input.data else {
        return syn::Error::new_spanned(input, "#[with_pagination] can only be used on structs")
            .to_compile_error()
            .into();
    };

    // Only process structs with named fields
    let Fields::Named(ref mut fields) = data_struct.fields else {
        return syn::Error::new_spanned(input, "#[with_pagination] requires named fields")
            .to_compile_error()
            .into();
    };

    // Add Relay pagination fields
    let new_fields: syn::FieldsNamed = syn::parse_quote! {
        {
            /// Number of items for forward pagination (cannot be combined with `last` or `before`)
            #[param(nullable = true, minimum = 1, maximum = 100, default = 20)]
            pub first: Option<i64>,
            /// Cursor to paginate after (i.e. forward pagination)
            #[param(nullable = true)]
            pub after: Option<String>,
            /// Number of items for backward pagination (cannot be combined with `first` or `after`)
            #[param(nullable = true, minimum = 1, maximum = 100)]
            pub last: Option<i64>,
            /// Cursor to paginate before (i.e. backward pagination)
            #[param(nullable = true)]
            pub before: Option<String>,
        }
    };

    // Append the new fields to existing fields
    for field in new_fields.named {
        fields.named.push(field);
    }

    let struct_name = &input.ident;

    TokenStream::from(quote! {
        #input

        impl #struct_name {
            /// Convert Relay pagination query parameters to a Page
            pub fn to_page(&self) -> Result<pagination::Page, pagination::PaginationError> {
                let page = pagination::Page {
                    first: self.first,
                    after: self.after.clone(),
                    last: self.last,
                    before: self.before.clone(),
                };

                // Validate pagination parameters
                page.validate()?;

                Ok(page)
            }
        }
    })
}
