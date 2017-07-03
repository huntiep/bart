use ast;
use syn;
use token;
use quote;

fn resolve(name: &token::Name, scope_depth: u32) -> syn::Ident {
    let root = match name.leading_dots {
        0 => "_s0".to_owned(),
        x => format!("_s{}", scope_depth.checked_sub(x).expect("Too many dots")),
    };

    let mut buf = root;
    for ref segment in &name.segments {
        buf.push('.');
        buf.push_str(segment);
    }

    syn::Ident::new(buf)
}

fn scope<'a>(name: token::Name, scope_level: u32, ast: ast::Ast<'a>) -> (syn::Ident, syn::Ident, quote::Tokens) {
    let name = resolve(&name, scope_level);
    let scope_variable = syn::Ident::new(format!("_s{}", scope_level));
    let nested_generated = generate(ast, scope_level + 1);

    (name, scope_variable, nested_generated)
}

pub fn generate(node: ast::Ast, scope_level: u32) -> quote::Tokens {
    use ast::Ast::*;
    match node {
        Sequence(seq) => {
            let items = seq.into_iter().map(|node| generate(node, scope_level));
            quote! { #(#items)* }
        },
        Literal(text) => {
            quote! { f.write_str(#text)?; }
        },
        Interpolation(name) => {
            let name = resolve(&name, scope_level);
            quote! { _bart::DisplayHtmlSafe::safe_fmt(&#name, f)?; }
        },
        UnescapedInterpolation(name) => {
            let name = resolve(&name, scope_level);
            quote! { ::std::fmt::Display::fmt(&#name, f)?; }
        },
        Iteration { name, nested } => {
            let (name, scope_variable, nested) = scope(name, scope_level, *nested);
            quote! {
                for ref #scope_variable in (&#name).into_iter() {
                    #nested
                }
            }
        },
        NegativeIteration { name, nested } => {
            let (name, scope_variable, nested) = scope(name, scope_level, *nested);
            quote! {
                for ref #scope_variable in _bart::NegativeIterator::neg_iter(&#name) {
                    #nested
                }
            }
        },
        Conditional { name, nested } => {
            let (name, scope_variable, nested) = scope(name, scope_level, *nested);
            quote! {
                if _bart::Conditional::val(&#name) {
                    let ref #scope_variable = #name;
                    #nested
                }
            }
        },
        NegativeConditional { name, nested } => {
            let (name, scope_variable, nested) = scope(name, scope_level, *nested);
            quote! {
                if !_bart::Conditional::val(&#name) {
                    let ref #scope_variable = #name;
                    #nested
                }
            }
        },
        Scope { name, nested } => {
            let (name, scope_variable, nested) = scope(name, scope_level, *nested);
            quote! {
                {
                    let ref #scope_variable = #name;
                    #nested
                }
            }
        },
    }
}
