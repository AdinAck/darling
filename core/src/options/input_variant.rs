use syn;

use {FromMetaItem, Error, Result};
use codegen;
use options::{Core, InputField, ParseAttribute};
use util::VariantData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputVariant {
    ident: syn::Ident,
    attr_name: Option<String>,
    data: VariantData<InputField>,
}

impl InputVariant {
    pub fn as_codegen_variant<'a>(&'a self, ty_ident: &'a syn::Ident) -> codegen::Variant<'a> {
        codegen::Variant {
            name_in_attr: self.attr_name.as_ref().map(|s| s.as_str()).unwrap_or(self.ident.as_ref()),
            variant_ident: &self.ident,
            ty_ident,
            data: self.data.as_ref().map(InputField::as_codegen_field)
        }
    }

    pub fn from_variant(v: &syn::Variant, parent: Option<&Core>) -> Result<Self> {
        let mut starter = (InputVariant {
            ident: v.ident.clone(),
            attr_name: Default::default(),
            data: VariantData::empty_from(&v.data),
        }).parse_attributes(&v.attrs)?;

        starter.data = match v.data {
            syn::VariantData::Struct(ref fields) => {
                let mut items = Vec::with_capacity(fields.len());
                for item in fields {
                    items.push(InputField::from_field(item, parent)?);
                }

                VariantData::Struct(items)
            }
            syn::VariantData::Tuple(ref fields) => {
                let mut items = Vec::with_capacity(fields.len());
                for item in fields {
                    items.push(InputField::from_field(item, parent)?);
                }

                VariantData::Tuple(items)
            }
            syn::VariantData::Unit => VariantData::Unit
        };

        Ok(if let Some(p) = parent {
            starter.with_inherited(p)
        } else {
            starter
        })
    }

    fn with_inherited(mut self, parent: &Core) -> Self {
        if self.attr_name.is_none() {
            self.attr_name = Some(parent.rename_rule.apply_to_variant(&self.ident));
        }

        self
    }
}

impl ParseAttribute for InputVariant {
    fn parse_nested(&mut self, mi: &syn::MetaItem) -> Result<()> {
        let name = mi.name().to_string();
        match name.as_str() {
            "rename" => { self.attr_name = FromMetaItem::from_meta_item(mi)?; Ok(()) }
            n => Err(Error::unknown_field(n)),
        }
    }
}