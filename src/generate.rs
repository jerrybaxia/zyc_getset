use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_error2::abort;
use syn::{
    self, ext::IdentExt, spanned::Spanned, Expr, Field, GenericArgument, Lit, Meta, MetaNameValue,
    PathArguments, Type, TypePath, Visibility,
};

use self::GenMode::{Get, GetClone, GetCopy, GetMut, Set};
use super::parse_attr;

pub struct GenParams {
    pub mode: GenMode,
    pub global_attr: Option<Meta>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum GenMode {
    Get,
    GetCopy,
    GetClone,
    Set,
    GetMut,
}

impl GenMode {
    pub fn name(self) -> &'static str {
        match self {
            Get => "get",
            GetCopy => "get_copy",
            GetClone => "get_clone",
            Set => "set",
            GetMut => "get_mut",
        }
    }

    pub fn prefix(self) -> &'static str {
        match self {
            Get | GetCopy | GetClone | GetMut => "",
            Set => "set_",
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Get | GetCopy | GetClone | Set => "",
            GetMut => "_mut",
        }
    }

    fn is_get(self) -> bool {
        match self {
            GenMode::Get | GenMode::GetCopy | GenMode::GetClone | GenMode::GetMut => true,
            GenMode::Set => false,
        }
    }
}

// Helper function to extract string from Expr
fn expr_to_string(expr: &Expr) -> Option<String> {
    if let Expr::Lit(expr_lit) = expr {
        if let Lit::Str(s) = &expr_lit.lit {
            Some(s.value())
        } else {
            None
        }
    } else {
        None
    }
}

// Helper function to parse visibility
fn parse_vis_str(s: &str, span: proc_macro2::Span) -> Visibility {
    match syn::parse_str(s) {
        Ok(vis) => vis,
        Err(e) => abort!(span, "Invalid visibility found: {}", e),
    }
}

// Helper function to parse visibility attribute
pub fn parse_visibility(attr: Option<&Meta>, meta_name: &str) -> Option<Visibility> {
    let meta = attr?;
    let Meta::NameValue(MetaNameValue { value, path, .. }) = meta else {
        return None;
    };

    if !path.is_ident(meta_name) {
        return None;
    }

    let value_str = expr_to_string(value)?;
    let vis_str = value_str.split(' ').find(|v| *v != "with_prefix")?;

    Some(parse_vis_str(vis_str, value.span()))
}

/// Some users want legacy/compatibility.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field, params: &GenParams) -> bool {
    // helper function to check if meta has `with_prefix` attribute
    let meta_has_prefix = |meta: &Meta| -> bool {
        if let Meta::NameValue(name_value) = meta {
            if let Some(s) = expr_to_string(&name_value.value) {
                return s.split(" ").any(|v| v == "with_prefix");
            }
        }
        false
    };

    let field_attr_has_prefix = f
        .attrs
        .iter()
        .filter_map(|attr| parse_attr(attr, params.mode))
        .find(|meta| {
            meta.path().is_ident("get")
                || meta.path().is_ident("get_copy")
                || meta.path().is_ident("get_clone")
        })
        .as_ref()
        .map_or(false, meta_has_prefix);

    let global_attr_has_prefix = params.global_attr.as_ref().map_or(false, meta_has_prefix);

    field_attr_has_prefix || global_attr_has_prefix
}

pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    let field_name = field
        .ident
        .clone()
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    let fn_name = if !has_prefix_attr(field, params)
        && (params.mode.is_get())
        && params.mode.suffix().is_empty()
        && field_name.to_string().starts_with("r#")
    {
        field_name.clone()
    } else {
        Ident::new(
            &format!(
                "{}{}{}{}",
                if has_prefix_attr(field, params) && (params.mode.is_get()) {
                    "get_"
                } else {
                    ""
                },
                params.mode.prefix(),
                field_name.unraw(),
                params.mode.suffix()
            ),
            Span::call_site(),
        )
    };
    let ty = field.ty.clone(); // 获取字段的类型 生成 ty_get_name 作为 TokenStream

    // 判断是否为基础类型？如果是的话将 get 自动转为 getCopy
    let mut mode = params.mode;
    if mode == GenMode::Get
        && (check_type_is_copy(&field.ty)
            || extract_option_type(&field.ty)
                .map_or(false, |inner_ty| check_type_is_copy(inner_ty)))
    {
        // 如果当前属性是 copy 类型，那么当 get 时，自动转为 getCopy
        mode = GenMode::GetCopy;
    }

    let ty_get_name = if let Some(inner_ty) = extract_option_type(&ty) {
        quote! { Option<&#inner_ty> }
    } else {
        quote! { &#ty }
    };

    let ty_get_return = if extract_option_type(&ty).is_some() {
        quote! { self.#field_name.as_ref() }
    } else {
        quote! { &self.#field_name }
    };

    let doc = field.attrs.iter().filter(|v| v.meta.path().is_ident("doc"));

    // 取出是否有 skip
    let attr = field
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .last() // 取出自定义注解
        .or_else(|| params.global_attr.clone()); // 没有自定义注解时，使用 struct 全局注解

    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    // 添加以下代码，当visibility 为 None 时，或为 private ，则设置为 public
    // 因为 get set 是对外的，没有必要私有，所以默认都为 public。
    let visibility = match visibility {
        None | Some(Visibility::Inherited) => Some(Visibility::Public(syn::token::Pub {
            span: Span::call_site(),
        })),
        Some(v) => Some(v),
    };
    // 以上是添加代码
    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => {
            quote! {}
        }
        Some(_) => match mode {
            GenMode::Get => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty_get_name {
                        #ty_get_return
                    }
                }
            }
            GenMode::GetCopy => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name
                    }
                }
            }
            GenMode::GetClone => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name.clone()
                    }
                }
            }
            GenMode::Set => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.#field_name = val;
                        self
                    }
                }
            }
            GenMode::GetMut => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.#field_name
                    }
                }
            }
        },
        None => quote! {},
    }
}
fn extract_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    for arg in args.args.iter() {
                        if let GenericArgument::Type(inner_ty) = arg {
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}

//
fn check_type_is_copy(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            if path.segments.is_empty() {
                return false;
            }
            let last_segment = path.segments.last().unwrap();
            is_copy_ident(&last_segment.ident)
        }
        Type::Array(array_type) => check_type_is_copy(&*array_type.elem),
        Type::Tuple(tuple_type) => tuple_type.elems.iter().all(check_type_is_copy),
        Type::Group(group) => check_type_is_copy(&*group.elem),
        Type::Paren(paren) => check_type_is_copy(&*paren.elem),
        Type::BareFn(_) => true, // 函数指针默认实现了 Copy
        _ => false,
    }
}

fn is_copy_ident(ident: &Ident) -> bool {
    // TODO 未能找到直接从类型T获取当前是否实现Copy trait的方法，这里写死了常用的基础数据类型。
    match ident.to_string().as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "isize"
        | "usize" | "f32" | "f64" | "bool" | "char" | "Copy" => true,
        _ => false,
    }
}
