use crate::model::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum MessageFilter {
    After(MessageId),
    Around(MessageId),
    Before(MessageId),
    MostRecent,
}
