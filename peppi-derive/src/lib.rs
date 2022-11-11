use darling::{ast, FromDeriveInput, FromField, FromMeta};
use quote::{quote, ToTokens};

type Result<T> = std::result::Result<T, darling::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u8, u8);

impl FromMeta for Version {
	fn from_string(value: &str) -> Result<Self> {
		if let Ok(re) = regex::Regex::new(r"^(\d+)\.(\d+)$") {
			if let Some(caps) = re.captures(value) {
				return Ok(Version(
					caps.get(1).unwrap().as_str().parse::<u8>().unwrap(),
					caps.get(2).unwrap().as_str().parse::<u8>().unwrap(),
				));
			}
		}
		Err(darling::Error::unsupported_format("X.Y"))
	}
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(slippi), supports(struct_any))]
pub(crate) struct MyInputReceiver {
	ident: syn::Ident,
	generics: syn::Generics,
	data: ast::Data<(), MyFieldReceiver>,
}

fn if_ver(version: Option<Version>, inner: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
	match version {
		Some(version) => {
			let Version(major, minor) = version;
			quote!(match version.0 > #major || (version.0 == #major && version.1 >= #minor) {
				true => Some(#inner),
				_ => None,
			},)
		}
		_ => quote!(Some(#inner),),
	}
}

/// Takes an `Option<...>` type and returns the inner type.
fn wrapped_type(ty: &syn::Type) -> Option<&syn::Type> {
	match ty {
		syn::Type::Path(tpath) => {
			let segment = &tpath.path.segments[0];
			match segment.ident.to_string().as_str() {
				"Option" =>
				// FIXME: will miss `std::option::Option`, etc
				{
					match &segment.arguments {
						syn::PathArguments::AngleBracketed(args) => match &args.args[0] {
							syn::GenericArgument::Type(ty) => Some(ty),
							_ => None,
						},
						_ => None,
					}
				}
				_ => None,
			}
		}
		_ => None,
	}
}

impl ToTokens for MyInputReceiver {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let MyInputReceiver {
			ref ident,
			ref generics,
			ref data,
		} = *self;

		let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
		let mut fields = data
			.as_ref()
			.take_struct()
			.expect("Should never be enum")
			.fields;
		fields.sort_by_key(|f| f.version);

		let mut arrow_defaults = quote!();
		let mut arrow_fields = quote!();
		let mut arrow_arrays = quote!();
		let mut arrow_pushers = quote!();
		let mut arrow_null_pushers = quote!();
		let mut arrow_readers = quote!();

		for (i, f) in fields.into_iter().enumerate() {
			let ident = &f.ident;
			let name = ident
				.as_ref()
				.map(|n| n.to_string().trim_start_matches("r#").to_string())
				.unwrap_or(format!("{}", i));
			let ty = &f.ty;
			arrow_defaults.extend(quote!(
				#ident: <#ty as ::peppi_arrow::Arrow>::arrow_default(),
			));
			arrow_fields.extend(if_ver(
				f.version,
				quote!(::arrow2::datatypes::Field::new(
					#name,
					<#ty>::data_type(context),
					<#ty>::is_nullable(),
				)),
			));
			arrow_arrays.extend(if_ver(
				f.version,
				quote!(Box::new(<#ty>::arrow_array(context))
					as Box<dyn ::arrow2::array::MutableArray>),
			));
			arrow_pushers.extend(quote!(
				if num_fields > #i {
					self.#ident.arrow_push(array.mut_values()[#i].deref_mut());
				}
			));
			arrow_null_pushers.extend(quote!(
				if num_fields > #i {
					<#ty>::arrow_push_null(array.mut_values()[#i].deref_mut());
				}
			));
			arrow_readers.extend(if f.version.is_some() {
				let wrapped =
					wrapped_type(ty).expect(stringify!(Failed to unwrap type for: #ident));
				quote!(
					let values = struct_array.values();
					if values.len() > #i {
						let mut value = <#wrapped as ::peppi_arrow::Arrow>::arrow_default();
						value.arrow_read(values[#i].as_ref(), idx);
						self.#ident = Some(value);
					}
				)
			} else {
				quote!(
					self.#ident.arrow_read(struct_array.values()[#i].as_ref(), idx);
				)
			});
		}

		tokens.extend(quote! {
			impl #impl_generics ::peppi_arrow::Arrow for #ident #ty_generics #where_clause {
				type ArrowArray = ::arrow2::array::MutableStructArray;

				fn arrow_default() -> Self {
					Self {
						#arrow_defaults
					}
				}

				fn data_type<C: ::peppi_arrow::Context>(context: C) -> ::arrow2::datatypes::DataType {
					let version = context.slippi_version();
					let fields = vec![#arrow_fields].into_iter().filter_map(|f| f).collect();
					::arrow2::datatypes::DataType::Struct(fields)
				}

				fn arrow_array<C: ::peppi_arrow::Context>(context: C) -> Self::ArrowArray {
					let version = context.slippi_version();
					let data_type = Self::data_type(context);
					let values: Vec<_> = vec![#arrow_arrays].into_iter().filter_map(|f| f).collect();
					::arrow2::array::MutableStructArray::new(data_type, values)
				}

				fn arrow_push(&self, array: &mut dyn ::arrow2::array::MutableArray) {
					use std::ops::DerefMut;
					let array = array.as_mut_any().downcast_mut::<Self::ArrowArray>()
						.expect(stringify!(Failed to downcast array for: #ident));
					let num_fields = array.values().len();
					#arrow_pushers
					array.push(true);
				}

				fn arrow_push_null(array: &mut dyn ::arrow2::array::MutableArray) {
					use std::ops::DerefMut;
					let array = array.as_mut_any().downcast_mut::<Self::ArrowArray>()
						.expect(stringify!(Failed to downcast array for: #ident));
					let num_fields = array.values().len();
					#arrow_null_pushers
					array.push(false);
				}

				fn arrow_read(&mut self, array: &dyn ::arrow2::array::Array, idx: usize) {
					let struct_array = array.as_any().downcast_ref::<::arrow2::array::StructArray>()
						.expect(stringify!(Failed to downcast array for: #ident));
					#arrow_readers
				}
			}
		});
	}
}

#[derive(Debug, FromField)]
#[darling(attributes(slippi))]
pub(crate) struct MyFieldReceiver {
	ident: Option<syn::Ident>,
	ty: syn::Type,
	#[darling(default)]
	version: Option<Version>,
}

#[proc_macro_derive(Arrow, attributes(slippi))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = syn::parse(input).expect("Failed to parse item");
	build_converters(ast).expect("Failed to build converters")
}

fn build_converters(ast: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
	let receiver = MyInputReceiver::from_derive_input(&ast).map_err(|e| e.flatten())?;
	Ok(quote!(#receiver).into())
}
