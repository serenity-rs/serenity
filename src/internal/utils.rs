use std::borrow::Cow;
use std::fmt::Write;

pub fn join_to_string(
    sep: impl std::fmt::Display,
    iter: impl IntoIterator<Item = impl std::fmt::Display>,
) -> String {
    let mut buf = String::new();
    for item in iter {
        write!(buf, "{item}{sep}").expect("writing to a string should never fail");
    }

    buf.truncate(buf.len() - 1);
    buf
}

// The `Deserialize` derive macro has an opt-in specialization for borrowed deserialization of
// `Cow<str>` and `Cow<[u8]>`, enabled by decorating the field with `#[serde(borrow)]`.
#[derive(Deserialize)]
pub struct CowStr<'a>(#[serde(borrow)] pub Cow<'a, str>);
