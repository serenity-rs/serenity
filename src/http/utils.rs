use model::channel::ReactionType;
use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};

pub fn reaction_type_data(reaction_type: &ReactionType) -> String {
    match *reaction_type {
        ReactionType::Custom {
            id,
            ref name,
            ..
        } => format!("{}:{}", name.as_ref().map_or("", |s| s.as_str()), id),
        ReactionType::Unicode(ref unicode) => {
            percent_encode(unicode.as_bytes(), DEFAULT_ENCODE_SET).to_string()
        }
    }
}
