use quote::quote;
use syn::{ext::IdentExt, spanned::Spanned, Error, Expr, ItemConst, Lit, Result};

use crate::{attr::{Attr, ConstantAttr, ContainerAttr}, utils::make_string_literal, Dependencies, DerivedTS};

pub(crate) fn constant_def(s: &ItemConst, attr: ConstantAttr) -> Result<DerivedTS> {
    let ts_name = make_string_literal(&s.ident.unraw().to_string(), s.ident.span());
    type_def(&attr, ts_name, &s.expr)
}

fn type_def(attr: &ConstantAttr, ts_name: Expr, value: &Expr) -> Result<DerivedTS> {
    attr.assert_validity(&())?;
    let crate_rename = attr.crate_rename();
    
    let text: String = match value {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Float(float) => float.base10_digits().to_string(),
            Lit::Int(int) => int.base10_digits().to_string(),
            Lit::Str(str) => to_typescript_syntax(str.value()),
            Lit::ByteStr(str) => {
                let token = str.token().to_string();
                let quote_index = token.find('"').unwrap();
                let last_quote_index = token.rfind('"').unwrap();
                let text = token[quote_index + 1..last_quote_index].to_string();
                to_typescript_syntax(text)
            },
            Lit::Bool(bool) => bool.value.to_string(),
            _ => return Err(Error::new(value.span(), "expected literal")),
        },
        _ => return Err(Error::new(value.span(), "expected literal")),
    };
    
    Ok(DerivedTS {
        crate_rename: crate_rename.clone(),
        inline: quote!(#text),
        inline_flattened: None,
        docs: attr.docs.clone(),
        dependencies: Dependencies::new(crate_rename),
        export: attr.export_to.is_some(),
        export_to: attr.export_to.clone(),
        ts_name,
        concrete: Default::default(),
        bound: None,
        is_ts_enum: false,
        is_constant: true,
    })
}

fn to_typescript_syntax(value: String) -> String {
    if value.contains('\n') {
        let text = value.replace("`", "\\`").replace("${", "\\${");
        return format!("`{text}`");
    }
    if value.contains('\"') && !value.contains('\'') {
        return format!("\'{value}\'");
    }
    if value.contains('\'') && !value.contains('\"') {
        return format!("\"{value}\"");
    }
    format!("\"{value}\"")
}