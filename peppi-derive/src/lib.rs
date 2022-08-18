use darling::{ast, FromDeriveInput, FromField, FromMeta};
use quote::{quote, ToTokens};

type Result<T> = std::result::Result<T, darling::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version (u8, u8);

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
		},
		_ => quote!(Some(#inner),),
	}
}

/// Takes an `Option<...>` type and returns the inner type.
fn wrapped_type(ty: &syn::Type) -> Option<&syn::Type> {
	match ty {
		syn::Type::Path(tpath) => {
			let segment = &tpath.path.segments[0];
			match segment.ident.to_string().as_str() {
				"Option" => // FIXME: will miss `std::option::Option`, etc
					match &segment.arguments {
						syn::PathArguments::AngleBracketed(args) =>
							match &args.args[0] {
								syn::GenericArgument::Type(ty) =>
									Some(ty),
								_ => None,
							},
						_ => None,
					}
				_ => None,
			}
		},
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
		let mut arrow_builders = quote!();
		let mut arrow_writers = quote!();
		let mut arrow_null_writers = quote!();
		let mut arrow_readers = quote!();

		for (i, f) in fields.into_iter().enumerate() {
			let ident = &f.ident;
			let name = ident.as_ref()
				.map(|n| n.to_string().trim_start_matches("r#").to_string())
				.unwrap_or(format!("{}", i));
			let ty = &f.ty;
			arrow_defaults.extend(
				quote!(
					#ident: <#ty as ::peppi_arrow::Arrow>::default(),
				)
			);
			arrow_fields.extend(if_ver(f.version,
				quote!(::arrow::datatypes::Field::new(
					#name,
					<#ty>::data_type(context),
					<#ty>::is_nullable(),
				))
			));
			arrow_builders.extend(if_ver(f.version,
				quote!(Box::new(<#ty>::builder(len, context))
					as Box<dyn ::arrow::array::ArrayBuilder>)
			));
			arrow_writers.extend(
				quote!(
					let x: Option<usize> = None;
					if num_fields > #i {
						self.#ident.write(
							builder.field_builder::<<#ty as ::peppi_arrow::Arrow>::Builder>(#i).expect(stringify!(Failed to create builder for: #ident)),
							context,
						);
					}
				)
			);
			arrow_null_writers.extend(
				quote!(
					if num_fields > #i {
						<#ty>::write_null(
							builder.field_builder::<<#ty as ::peppi_arrow::Arrow>::Builder>(#i).expect(stringify!(Failed to create null builder for: #ident)),
							context,
						);
					}
				)
			);
			arrow_readers.extend(
				if f.version.is_some() {
					let wrapped = wrapped_type(ty).expect(stringify!(Failed to unwrap type for: #ident));
					quote!(
						if struct_array.num_columns() > #i {
							let mut value = <#wrapped as ::peppi_arrow::Arrow>::default();
							value.read(struct_array.column(#i).clone(), idx);
							self.#ident = Some(value);
						}
					)
				} else {
					quote!(
						self.#ident.read(struct_array.column(#i).clone(), idx);
					)
				}
			);
		}

		tokens.extend(quote! {
			impl #impl_generics ::peppi_arrow::Arrow for #ident #ty_generics #where_clause {
				type Builder = ::arrow::array::StructBuilder;

				fn default() -> Self {
					Self {
						#arrow_defaults
					}
				}

				fn fields<C: ::peppi_arrow::Context>(context: C) -> Vec<::arrow::datatypes::Field> {
					let version = context.slippi_version();
					vec![#arrow_fields].into_iter().filter_map(|f| f).collect()
				}

				fn data_type<C: ::peppi_arrow::Context>(context: C) -> ::arrow::datatypes::DataType {
					::arrow::datatypes::DataType::Struct(Self::fields(context))
				}

				fn builder<C: ::peppi_arrow::Context>(len: usize, context: C) -> Self::Builder {
					let version = context.slippi_version();
					let fields = Self::fields(context);
					let builders: Vec<_> = vec![#arrow_builders].into_iter().filter_map(|f| f).collect();
					::arrow::array::StructBuilder::new(fields, builders)
				}

				fn write<C: ::peppi_arrow::Context>(&self, builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
					let builder = builder.as_any_mut().downcast_mut::<Self::Builder>()
						.expect(stringify!(Failed to downcast builder for: #ident));
					let num_fields = builder.num_fields();
					#arrow_writers
					builder.append(true)
				}

				fn write_null<C: ::peppi_arrow::Context>(builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
					let builder = builder.as_any_mut().downcast_mut::<Self::Builder>()
						.expect(stringify!(Failed to downcast null builder for: #ident));
					let num_fields = builder.num_fields();
					#arrow_null_writers
					builder.append(false)
				}

				fn read(&mut self, array: ::arrow::array::ArrayRef, idx: usize) {
					let struct_array = array.as_any().downcast_ref::<arrow::array::StructArray>()
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
