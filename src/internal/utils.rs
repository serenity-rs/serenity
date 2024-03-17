use std::fmt::Write;

pub(crate) fn join_to_string(
    sep: impl std::fmt::Display,
    iter: impl IntoIterator<Item = impl std::fmt::Display>,
) -> String {
    let mut buf = String::new();
    for item in iter {
        write!(buf, "{item}{sep}").unwrap();
    }

    buf.truncate(buf.len() - 1);
    buf
}

// Required because of https://github.com/Crazytieguy/gat-lending-iterator/issues/31
macro_rules! lending_for_each {
    ($iter:expr, |$item:ident| $body:expr ) => {
        while let Some(mut $item) = $iter.next() {
            $body
        }
    };
}

pub(crate) use lending_for_each;
