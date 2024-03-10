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
