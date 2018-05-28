use model::channel::ReactionType;

pub fn reaction_type_data(reaction_type: &ReactionType) -> String {
    match *reaction_type {
        ReactionType::Custom {
            id,
            ref name,
            ..
        } => format!("{}:{}", name.as_ref().map_or("", |s| s.as_str()), id),
        ReactionType::Unicode(ref unicode) => unicode.clone(),
    }
}
