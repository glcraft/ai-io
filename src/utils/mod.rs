#![allow(dead_code)]

pub mod split_bytes;
pub mod flatten_stream;

pub use split_bytes::SplitBytesFactory;
pub use flatten_stream::FlattenTrait;

macro_rules! hashmap {
    ($($name:ident => $value:expr),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert({stringify!($name)}.into(), {$value}.into());)*
        map
    }};
}

pub(crate) use hashmap;

pub fn append_to_vec<T: Copy>(vec: &mut Vec<T>, other: &[T]) {
    vec.reserve(other.len());
    other.iter().for_each(|v| vec.push(*v));
}

macro_rules! vec_merge {
    ($tokens:ident, $($other_tokens:expr),*) => {{
        let arrs = [$($other_tokens),*];
        $tokens.reserve(arrs.iter().map(|arr| arr.len()).sum());
        arrs.iter().map(|arr| arr.iter()).flatten().for_each(|v| $tokens.push(*v));
    }};
}

pub(crate) use vec_merge;