# zyc_getset

The current 'crate' is an extension to ' `getset 0.1.3`(https://crates.io/crates/getset) '.
Mainly added 'get_clone' to return cloned objects of fields,
Return 'Option<&T>' for 'Option<T>', and the method defaults to the 'pub' level.

> 当前`crate`是对'getset' 的扩展。
>
> 主要添加了`get_clone`来返回字段的克隆对象，
>
> 为`Option<T>` 返回 `Option<&T>`,同时方法默认为`pub`级别。

```rust
use zyc_getset::CloneGetters;
use zyc_getset::CopyGetters;
use zyc_getset::Getters;
use zyc_getset::Setters;
// 在 getset 基本上添加 get_clone 返回对象的克隆
// 在 getset 基本上，当 get 返回的是 &Option<T> 类型时，自动转换为返回 Option<&T>
#[derive(Getters, Setters, CloneGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Foo {
    #[getset(skip)] // 跳过 struct getset
    #[getset(get_copy, set)] // 自定义 copy 返回
    id: i32,
    name: String,     // 生成 get set
    bar: Option<Bar>, // 生成 get set, Option get 会返回 Option<T>.as_ref()
}

#[derive(Clone, Default, Getters, Setters, CloneGetters, CopyGetters)]
pub struct Bar {
    // 生成 get set
    #[getset(get, set)]
    id: i32,
    // 生成 get clone set
    #[getset(get_clone, set)]
    name: String,
    // 生成 get copy set
    #[getset(get_copy, set)]
    ty: i32,
}
```

以下为生成的代码

```rust
mod foo {
    use zyc_getset::CloneGetters;
    use zyc_getset::CopyGetters;
    use zyc_getset::Getters;
    use zyc_getset::Setters;
    #[getset(get, set)]
    pub struct Foo {
        #[getset(skip)]
        #[getset(get_copy, set)]
        id: i32,
        name: String,
        bar: Option<Bar>,
    }
    impl Foo {
        #[inline(always)]
        pub fn name(&self) -> &String {
            &self.name 	// 正常的 get 返回对象的引用		
        }
        #[inline(always)]
        pub fn bar(&self) -> Option<&Bar> {
            self.bar.as_ref() // Option<T> 的 get 返回 Option<&Bar>
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
    }
    impl Foo {}
    impl Foo {
        #[inline(always)]
        pub fn id(&self) -> i32 {
            self.id   // copy 方式 get 返回 对象的 copy 
        }
    }
    
    pub struct Bar {
        #[getset(get, set)]
        id: i32,
        #[getset(get_clone, set)]
        name: String,
        #[getset(get_copy, set)]
        ty: i32,
    }
    impl Bar {
        #[inline(always)]
        pub fn id(&self) -> &i32 {
            &self.id  // 正常的 get 返回对象的引用，和 Foo.id 的 copy 不同，这里返回引用
        }
    }
    impl Bar {
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
        pub fn set_ty(&mut self, val: i32) -> &mut Self {
            self.ty = val;
            self
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn name(&self) -> String {
            self.name.clone() // 这里返回克隆
        }
    }
    impl Bar {
        #[inline(always)]
        pub fn ty(&self) -> i32 {
            self.ty  // 这里返回 copy
        }
    }
}
```

主要修改的代码

```rust
// generate.rs
pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    ...
    
	let ty = field.ty.clone(); // 获取字段的类型
    // 生成 ty_get_name 和 ty_get_return ，修改 get 方法的模板输出
    // 将 Option<T> 由 get 原来返回 &Option<T> 改为返回  Option<&T>
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
    
    GenMode::Get => {
        quote! {
            #(#doc)*
            #[inline(always)]
            #visibility fn #fn_name(&self) -> #ty_get_name {
                #ty_get_return
            }
        }
    }
    
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

