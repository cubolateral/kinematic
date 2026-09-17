use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_track_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new_spanned(name, "TrackEnum can only be derived for enums.")
                .into_compile_error()
                .into();
        }
    };

    if let Some(variant) = variants
        .iter()
        .find(|variant| !matches!(variant.fields, Fields::Unit))
    {
        return syn::Error::new_spanned(
            variant,
            "TrackEnum only supports enums with fieldless variants.",
        )
        .into_compile_error()
        .into();
    }

    let variant_names: Vec<_> = variants.iter().map(|variant| &variant.ident).collect();
    let mut generics = input.generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(Self: Clone));
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics kinematic::core::TrackValueType for #name #type_generics #where_clause {
            type Input = Self;

            const CHOICES: kinematic::core::TrackChoices =
                kinematic::core::TrackChoices::Enum(
                    <Self as kinematic::core::TrackEnum>::VARIANTS,
                );

            fn into_track_value(self) -> kinematic::core::TrackValue {
                kinematic::core::TrackValue::Enum(match self {
                    #(Self::#variant_names => stringify!(#variant_names)),*
                })
            }

            fn from_track_value(value: kinematic::core::TrackValue) -> Option<Self> {
                match value {
                    kinematic::core::TrackValue::Enum(value) => match value {
                        #(stringify!(#variant_names) => Some(Self::#variant_names),)*
                        _ => None,
                    },
                    _ => None,
                }
            }
        }

        impl #impl_generics kinematic::core::TrackEnum for #name #type_generics #where_clause {
            const VARIANTS: &'static [&'static str] = &[#(stringify!(#variant_names)),*];
        }
    }
    .into()
}
