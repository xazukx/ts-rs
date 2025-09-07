use quote::quote;
use syn::{ext::IdentExt, spanned::Spanned, Error, Expr, Ident, ItemConst, ItemStruct, Lit, Result};

use crate::{attr::{Attr, ConstantAttr, ContainerAttr}, utils::make_string_literal, Dependencies, DerivedTS};

pub(crate) fn constant_def(s: &ItemConst, attr: ConstantAttr) -> Result<DerivedTS> {
    let ts_name = match &attr.rename {
        Some(rename) => rename.clone(),
        _ => make_string_literal(&s.ident.unraw().to_string(), s.ident.span()),
    };
    type_def(&attr, ts_name, &s.expr)
}

pub(crate) fn constant_def_struct(strct: &ItemStruct, attr: ConstantAttr) -> Result<DerivedTS> {
    let ident_name = match &attr.rename {
        Some(rename) => {
            if let Expr::Lit(lit_str) = rename {
                if let Lit::Str(lit_str) = &lit_str.lit {
                    Ident::new(&lit_str.value(), lit_str.span())
                } else {
                    return Err(Error::new(rename.span(), "expected string literal"));
                }
            } else {
                return Err(Error::new(rename.span(), "expected string literal"));
            }
        },
        _ => strct.ident.unraw(),
    };
    let ts_name = match &attr.rename {
        Some(rename) => rename.clone(),
        _ => {
            let mut name = strct.ident.unraw().to_string();
            name.insert_str(0, "Default");
            make_string_literal(&name, strct.ident.span())
        },
    };
    attr.assert_validity(&())?;
    let crate_rename = attr.crate_rename();
    
    let inline = quote! {
        format!(
            "{} as const",
            serde_json::to_string_pretty(&#ident_name::default())
                .expect(&format!("Failed to serialize {} constant for ts_constant(default)", stringify!(#ident_name)))
        )
    };
    
    Ok(DerivedTS {
        crate_rename: crate_rename.clone(),
        inline: quote!(#inline),
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

fn type_def(attr: &ConstantAttr, ts_name: Expr, value: &Expr) -> Result<DerivedTS> {
    attr.assert_validity(&())?;
    let crate_rename = attr.crate_rename();
    
    let text = extract_expr_literal(value, attr)?;
    
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

fn extract_expr_literal(value: &Expr, attr: &ConstantAttr) -> Result<String> {
    let text: String = match value {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Float(float) => float.base10_digits().to_string(),
            Lit::Int(int) => int.base10_digits().to_string(),
            Lit::Str(str) => to_typescript_syntax(str.value()),
            Lit::ByteStr(str) => {
                if attr.array {
                    let arr = str.value();
                    return Ok(format!("{:?}", arr));
                }
                let token = str.token().to_string();
                let quote_index = token.find('"').unwrap();
                let last_quote_index = token.rfind('"').unwrap();
                let text = token[quote_index + 1..last_quote_index].to_string();
                to_typescript_syntax(text)
            },
            Lit::Bool(bool) => bool.value.to_string(),
            _ => return Err(Error::new(value.span(), "expected literal")),
        },
        Expr::Call(call) => {
            for arg in &call.args {
                match extract_expr_literal(arg, attr) {
                    Ok(text) => return Ok(text),
                    _ => {},
                }
            }
            return Err(Error::new(value.span(), format!("could not extract a literal from {:?}", value)));
        }
        _ => return Err(Error::new(value.span(), format!("expected expression literal not {:?}", value))),
    };
    Ok(text)
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