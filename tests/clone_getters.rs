#[macro_use]
extern crate zyc_getset;

use crate::submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(super::other)`
    pub mod other {
        #[derive(CloneGetters)]
        #[get_clone]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: usize,

            /// A doc comment.
            #[get_clone = "pub"]
            public_accessible: usize,
            // /// A doc comment.
            // #[get_copy = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super::other)"]
            // scope_accessible: usize,

            // Prefixed getter.
            #[get_clone = "with_prefix"]
            private_prefixed: usize,

            // Prefixed getter.
            #[get_clone = "pub with_prefix"]
            public_prefixed: usize,
        }

        impl Default for Plain {
            fn default() -> Plain {
                Plain {
                    private_accessible: 17,
                    public_accessible: 18,
                    private_prefixed: 19,
                    public_prefixed: 20,
                }
            }
        }

        #[derive(CloneGetters, Default)]
        #[get_clone]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[get_clone = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[get_copy = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super::other)"]
            // scope_accessible: usize,
        }

        #[derive(CloneGetters, Getters, Default)]
        #[get_clone]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[get_clone = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[get_copy = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get_copy = "pub(super::other)"]
            // scope_accessible: usize,
        }

        #[test]
        fn test_plain() {
            let val = Plain::default();
            val.private_accessible();
        }

        #[test]
        fn test_generic() {
            let val = Generic::<usize>::default();
            val.private_accessible();
        }

        #[test]
        fn test_where() {
            let val = Where::<usize>::default();
            val.private_accessible();
        }

        // #[test]
        // fn test_prefixed_plain() {
        //     let val = Plain::default();
        //     assert_eq!(19, val.get_private_prefixed());
        // }
    }
}

#[test]
fn test_plain() {
    let val = Plain::default();
    assert_eq!(18, val.public_accessible());
}

#[test]
fn test_generic() {
    let val = Generic::<usize>::default();
    assert_eq!(usize::default(), val.public_accessible());
}

#[test]
fn test_where() {
    let val = Where::<usize>::default();
    assert_eq!(usize::default(), val.public_accessible());
}

// #[test]
// fn test_prefixed_plain() {
//     let val = Plain::default();
//     assert_eq!(20, val.get_public_prefixed());
// }
