use syn::{parse_quote, Expr, Path, Ident, Result};

use super::{parse_assign_expr, parse_assign_from_str, Attr, ContainerAttr};

#[derive(Default, Clone)]
pub struct ConstantAttr {
    crate_rename: Option<Path>,
    pub export_to: Option<Expr>,
    pub docs: Vec<Expr>,
    pub rename: Option<Expr>,
    pub array: bool,
}

impl Attr for ConstantAttr {
    type Item = ();

    fn merge(self, other: Self) -> Self {
        Self {
            crate_rename: self.crate_rename.or(other.crate_rename),
            export_to: self.export_to.or(other.export_to),
            docs: other.docs,
            rename: self.rename.or(other.rename),
            array: self.array || other.array,
        }
    }

    fn assert_validity(&self, _: &Self::Item) -> Result<()> {
        Ok(())
    }
}

impl ContainerAttr for ConstantAttr {
    fn crate_rename(&self) -> Path {
        self.crate_rename
            .clone()
            .unwrap_or_else(|| parse_quote!(::ts_rs))
    }
}

impl_parse! {
    ConstantAttr(input, out) {
        "crate" => out.crate_rename = Some(parse_assign_from_str(input)?),
        "export_to" => out.export_to = Some(parse_assign_expr(input)?),
        "rename" => out.rename = Some(parse_assign_expr(input)?),
        "array" => out.array = true,
    }
}
