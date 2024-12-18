# zyc_getset

For personal projects using this crate, it is recommended to stick with the original `getset 0.1.3`(https://crates.io/crates/getset) crate. 

As an extension to the original crate, we've introduced a new method: get_clone.

Methods without an explicit visibility specifier are automatically declared as pub. This is a conscious design choice, as internal methods typically don't necessitate getset mechanisms.

```rust
#[derive(Getters, CloneGetters)]
pub struct Foo {
    #[get]
    id: i32,
    #[get_clone]
    name: String,
}
```

```rust
use zyc_getset::Getters;
pub struct Foo {
    #[get]
    id: i32,
    #[get_clone]
    name: String,
}
impl Foo {
    #[inline(always)]
    pub fn id(&self) -> &i32 {
        &self.id
    }
}
impl Foo {
    #[inline(always)]
    pub fn name(&self) -> String {  // default pub
        self.name.clone()   // add clone
    }
}
```

```rust
// generate.rs
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

