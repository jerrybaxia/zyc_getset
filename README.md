# zyc_getset

The current 'crate' is an extension to ' `getset 0.1.3`(https://crates.io/crates/getset) '.
Mainly added 'get_clone' to return cloned objects of fields,
Return 'Option<&T>' for 'Option`<T>`', and the method defaults to the 'pub' level.

> 当前 `crate`是对'getset' 的扩展。
>
> 主要添加了 `get_clone`来返回字段的克隆对象，
>
> 当为基础类型时 `get` 自动转为 `get_copy` ，
>
> 当字段有自定义注解时，自动对 struct 的注解进行 `skip` ，
>
> 当字段类型为 `Option<T>` 时，返回 `Option<&T>`，
>
> 同时方法默认为 `pub`级别。

## 使用示例


```rust
use zyc_getset::CloneGetters;
use zyc_getset::CopyGetters;
use zyc_getset::Getters;
use zyc_getset::MutGetters;
use zyc_getset::Setters;

type MyInt = i32; // 模拟自定义类型

#[derive(Getters, Setters, CloneGetters, CopyGetters, Default)]
#[getset(get, set)] // 自动默认生成 pub 级别（对外方法不需要 private）
pub struct Foo {
    id: i32,          // 生成 getCopy set,基础类型，会自动生成 getCopy
    name: String,     // 生成 get set
    bar: Option<Bar>, // 生成 get set, Option get 会返回 Option<&T>
    age: Option<i32>, // 生成 getCopy set, Option 进行了特殊处理，如果T为基础类型，也会生成 getCopy
    port: MyInt, // 生成 get set，现阶段 getset 中写死了 copy 只支持基础类型，自定义类型无法正确判断是否支持 Copy Trait，需要手动添加 get_copy
    #[getset(get_clone)] // 生成 getClone，但不会生成set，需要的话要显示添加 set
    email: String,
    #[getset(skip)] // skip 后不再生成
    password: String,
}

#[derive(Clone, Default, Getters, MutGetters, Setters, CloneGetters, CopyGetters)]
#[getset(get, set)]
pub struct Bar {
    // 生成 get_mut
    // #[getset(skip)]	// v0.0.4 版本添加支持不需要先写skip，当有自定义注解时，会自动 skip 掉 struct 的注解。
    #[getset(get_mut)]
    id: i32,
    // 生成 get clone set
    // #[getset(skip)]
    #[getset(get_clone, set)]
    name: String,
    // 生成 get copy set
    // #[getset(skip)]
    #[getset(get_copy, set)]
    ty: i32,
    // 生成 get set， 字段没有自定义getset时，使用struct的配置
    status: bool,
}

```

以下为生成的代码

```rust

mod foo {
    use zyc_getset::CloneGetters;
    use zyc_getset::CopyGetters;
    use zyc_getset::Getters;
    use zyc_getset::MutGetters;
    use zyc_getset::Setters;
    type MyInt = i32;
    #[getset(get, set)]
    pub struct Foo {
        id: i32,
        name: String,
        bar: Option<Bar>,
        age: Option<i32>,
        port: MyInt,
        email: String,
        #[getset(skip)]
        password: String,
    }
    impl Foo {
        #[inline(always)]
        pub fn id(&self) -> i32 {
            self.id  // 基础数据类型，由原来 get 返回 &i32 会自动生成 getCopy 返回 i32
        }
        #[inline(always)]
        pub fn name(&self) -> &String {
            &self.name
        }
        #[inline(always)]
        pub fn bar(&self) -> Option<&Bar> {
            self.bar.as_ref() // Option 属性，由原来的 &Option<T> 变成返回 Option<&T>
        }
        #[inline(always)]
        pub fn age(&self) -> Option<i32> {
            self.age // 基础数据类型的 Option 属性，由原来的 &Option<T> 直接返回 Copty trait 的 Option<T>
        }
        #[inline(always)]
        pub fn port(&self) -> &MyInt {
            &self.port // 自定义类型，未能自动识别 Copy Trait，如果需要返回 Myint 时，要手动进行 get_copy 注解
        }
    }
    impl Foo {
        #[inline(always)]
        pub fn set_id(&mut self, val: i32) -> &mut Self {
            self.id = val;
            self
        }
        #[inline(always)]
        pub fn set_name(&mut self, val: String) -> &mut Self {
            self.name = val;
            self
        }
        #[inline(always)]
        pub fn set_bar(&mut self, val: Option<Bar>) -> &mut Self {
            self.bar = val;
            self
        }
        #[inline(always)]
        pub fn set_age(&mut self, val: Option<i32>) -> &mut Self {
            self.age = val;
            self
        }
        #[inline(always)]
        pub fn set_port(&mut self, val: MyInt) -> &mut Self {
            self.port = val;
            self
        }
        // 因为 email 只手动注解了 #[getset(get_clone)] ，自定义注解没有显示设置 set，所以不会生成 emial 的 set
    }
    impl Foo {
        #[inline(always)]
        pub fn email(&self) -> String {
            self.email.clone()	// 生成 email 字段自定义注解的 get_clone
        }
    }
    
    ...
    
    #[getset(get, set)]
    pub struct Bar {
        #[getset(get_mut)]
        id: i32,
        #[getset(get_clone, set)]
        name: String,
        #[getset(get_copy, set)]
        ty: i32,
        status: bool,
    }
    
    ...
    
    impl Bar {
        #[inline(always)]
        pub fn status(&self) -> bool {
            self.status
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn id_mut(&mut self) -> &mut i32 {
            &mut self.id
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn set_name(&mut self, val: String) -> &mut Self {
            self.name = val;
            self
        }
        #[inline(always)]
        pub fn set_ty(&mut self, val: i32) -> &mut Self {
            self.ty = val;
            self
        }
        #[inline(always)]
        pub fn set_status(&mut self, val: bool) -> &mut Self {
            self.status = val;
            self
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn name(&self) -> String {
            self.name.clone()
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn ty(&self) -> i32 {
            self.ty
        }
    }
}
```

## 版本历史

### V0.0.4

1. 添加当为基础类型时 `get` 自动转为 `get_copy` ，
2. 当字段有自定义注解时，自动对 struct 的注解进行 `skip` ，

#### 主要修改代码

```rust
// generate.rs
// 修改基础类型时 `get` 自动转为 `get_copy` 
pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    ...
    
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
    ...
}
```

```rust

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
```

```rust
// lib.rs
// 修改当有自定义注解时对 struct 的注解进行自动 `skip`
fn parse_attr(attr: &syn::Attribute, mode: GenMode) -> Option<syn::Meta> {
    use syn::{punctuated::Punctuated, Token};

    if attr.path().is_ident("getset") {
        let meta_list =
            match attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated) {
                Ok(list) => list,
                Err(e) => abort!(attr.span(), "Failed to parse getset attribute: {}", e),
            };

        let (last, skip, mut collected) = meta_list
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("get")
                    || meta.path().is_ident("get_copy")
                    || meta.path().is_ident("get_clone")
                    || meta.path().is_ident("get_mut")
                    || meta.path().is_ident("set")
                    || meta.path().is_ident("skip"))
                {
                    abort!(meta.path().span(), "unknown setter or getter")
                }
            })
            .fold(
                (None, None, Vec::new()),
                |(last, skip, mut collected), meta| {
                    if meta.path().is_ident(mode.name()) {
                        // 如果当前 meta 匹配 mode.name()
                        // 说明在字段配置中找到配置
                        (Some(meta), skip, collected)
                    } else if meta.path().is_ident("skip") {
                        // 如果当前 meta 标识为 "skip"
                        // 则说明当前字段被标记为 skip
                        (last, Some(meta), collected)
                    } else {
                        collected.push(meta);
                        (
                            // 其他情况的话，说明字段进行自定义标注，那么就默认为 skip
                            last,
                            Some(Meta::NameValue(MetaNameValue {
                                path: syn::parse_quote!(skip),
                                eq_token: syn::token::Eq {
                                    spans: [Span::call_site()],
                                },
                                value: syn::parse_quote!(true),
                            })),
                            collected,
                        )
                    }
                },
            );

        if skip.is_some() {
            if last.is_some() {
                last // 说明有多个自定义注解，且其中有一个注解和当前 Mode 相同
            } else if last.is_none() && collected.is_empty() || 0 == 0 {
                // 说明需要跳过
                skip
            } else {
                abort!(
                    last.or_else(|| collected.pop()).unwrap().path().span(),
                    "use of setters and getters with skip is invalid"
                );
            }
        } else {
            last
        }
    } else if attr.path().is_ident(mode.name()) {
        // If skip is not used, return the last occurrence of matching
        // setter/getter, if there is any.
        attr.meta.clone().into()
    } else {
        None
    }
}
```

### V0.03

1. 当字段类型为 `Option<T>` 时，返回 `Option<&T>`

#### 主要修改代码

```rust
pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    ...

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
    
    ...
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
```

### V0.0.2

1. 添加了 `get_clone`来返回字段的克隆对象
2. 方法默认为 `pub`级别

#### 主要修改代码

```rust

pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    ...
    
    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    // 添加以下代码，当visibility 为 None 时，或为 private ，则设置为 public
    // 因为 get set 是对外的，没有必要私有，所以默认都为 public。
    let visibility = match visibility {
        None | Some(Visibility::Inherited) => Some(Visibility::Public(syn::token::Pub {
            span: Span::call_site(),
        })),
        Some(v) => Some(v),
    };
    
     ...
}
```

```rust

#[proc_macro_derive(CloneGetters, attributes(get_clone, with_prefix, getset))]
#[proc_macro_error]
pub fn clone_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::GetClone,
        global_attr: parse_global_attr(&ast.attrs, GenMode::GetClone),
    };
    produce(&ast, &params).into()
}
```

