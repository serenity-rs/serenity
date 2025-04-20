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
