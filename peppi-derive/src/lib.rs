use darling::{ast, FromDeriveInput, FromField, FromMeta};
use proc_macro;
use proc_macro2;
use quote::{quote, ToTokens};
//use syn::parse_str;

type Result<T> = std::result::Result<T, darling::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version (u8, u8);

impl FromMeta for Version {
	fn from_string(value: &str) -> Result<Self> {
		if let Ok(re) = regex::Regex::new(r"^(\d)\.(\d)$") {
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
#[darling(attributes(peppi), supports(struct_any))]
pub(crate) struct MyInputReceiver {
	ident: syn::Ident,
	generics: syn::Generics,
	data: ast::Data<(), MyFieldReceiver>,
}

fn if_ver(version: Option<Version>, inner: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
	match version {
		Some(version) => {
			let Version(major, minor) = version;
			quote!(match context.slippi_version() >= ::peppi_arrow::SlippiVersion(#major, #minor, 0) {
				true => Some(#inner),
				_ => None,
			},)
		},
		_ => quote!(Some(#inner),),
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

		let mut arrow_fields = quote!();
		let mut arrow_builders = quote!();
		let mut arrow_appenders = quote!();
		let mut arrow_null_appenders = quote!();
		for (i, f) in fields.into_iter().enumerate() {
			let ident = &f.ident;
			let name = ident.as_ref()
				.map(|n| n.to_string().trim_start_matches("r#").to_string())
				.unwrap_or(format!("{}", i));
			let ty = &f.ty;
			arrow_fields.extend(if_ver(
				f.version,
				quote!(::arrow::datatypes::Field::new(
					#name,
					<#ty>::data_type(context),
					<#ty>::is_nullable(),
				))
			));
			arrow_builders.extend(if_ver(
				f.version,
				quote!(Box::new(<#ty>::builder(len, context)) as Box<dyn ::arrow::array::ArrayBuilder>)
			));
			arrow_appenders.extend(
				quote!(
					if builder.num_fields() > #i {
						self.#ident.append(
							builder.field_builder::<<#ty as ::peppi_arrow::Arrow>::Builder>(#i).unwrap(),
							context,
						);
					}
				)
			);
			arrow_null_appenders.extend(
				quote!(
					if builder.num_fields() > #i {
						<#ty>::append_null(
							builder.field_builder::<<#ty as ::peppi_arrow::Arrow>::Builder>(#i).unwrap(),
							context,
						);
					}
				)
			);
		}

		tokens.extend(quote! {
			impl #impl_generics ::peppi_arrow::Arrow for #ident #ty_generics #where_clause {
				type Builder = ::arrow::array::StructBuilder;

				fn fields<C: ::peppi_arrow::Context>(context: C) -> Vec<::arrow::datatypes::Field> {
					vec![#arrow_fields].into_iter().filter_map(|f| f).collect()
				}

				fn data_type<C: ::peppi_arrow::Context>(context: C) -> ::arrow::datatypes::DataType {
					::arrow::datatypes::DataType::Struct(Self::fields(context))
				}

				fn builder<C: ::peppi_arrow::Context>(len: usize, context: C) -> Self::Builder {
					let fields = Self::fields(context);
					let builders = vec![#arrow_builders].into_iter().filter_map(|f| f).collect();
					::arrow::array::StructBuilder::new(fields, builders)
				}

				fn append<C: ::peppi_arrow::Context>(&self, builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
					let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
					#arrow_appenders
					builder.append(true).unwrap();
				}

				fn append_null<C: ::peppi_arrow::Context>(builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
					let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
					#arrow_null_appenders
					builder.append(false).unwrap();
				}
			}
		});
	}
}

#[derive(Debug, FromField)]
#[darling(attributes(peppi))]
pub(crate) struct MyFieldReceiver {
	ident: Option<syn::Ident>,
	ty: syn::Type,
	#[darling(default)]
	version: Option<Version>,
}

#[proc_macro_derive(Peppi, attributes(peppi))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = syn::parse(input).expect("Couldn't parse item");
	build_converters(ast).unwrap()
}

fn build_converters(ast: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
	let receiver = MyInputReceiver::from_derive_input(&ast).map_err(|e| e.flatten())?;
	Ok(quote!(#receiver).into())
}
