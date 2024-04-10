use crate::model::channel::{PollLayoutType, PollMedia, PollMediaEmoji};

#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsQuestion;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsAnswers;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsDuration;
#[derive(serde::Serialize, Clone, Debug)]
pub struct Ready;

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for NeedsQuestion {}
    impl Sealed for NeedsAnswers {}
    impl Sealed for NeedsDuration {}
    impl Sealed for Ready {}
}

use sealed::*;

/// "Only text is supported."
#[derive(serde::Serialize, Clone, Debug)]
struct CreatePollMedia {
    text: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[must_use = "Builders do nothing unless built"]
pub struct CreatePoll<Stage: Sealed> {
    question: CreatePollMedia,
    answers: Vec<CreatePollAnswer>,
    duration: u8,
    allow_multiselect: bool,
    layout_type: Option<PollLayoutType>,

    #[serde(skip)]
    _stage: Stage,
}

impl Default for CreatePoll<NeedsQuestion> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            question: CreatePollMedia {
                text: String::default(),
            },
            answers: Vec::default(),
            duration: u8::default(),
            allow_multiselect: false,
            layout_type: None,

            _stage: NeedsQuestion,
        }
    }
}

impl CreatePoll<NeedsQuestion> {
    /// Creates a builder for creating a Poll.
    ///
    /// This must be transitioned through in order, to provide all required fields.
    ///
    /// ```rust
    /// use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
    ///
    /// let poll = CreatePoll::new()
    ///     .question("Cats or Dogs?")
    ///     .answers(vec![
    ///         CreatePollAnswer::new().emoji("🐱".to_string()).text("Cats!"),
    ///         CreatePollAnswer::new().emoji("🐶".to_string()).text("Dogs!"),
    ///         CreatePollAnswer::new().text("Neither..."),
    ///     ])
    ///     .duration(std::time::Duration::from_secs(60 * 60 * 24 * 7));
    ///
    /// let message = CreateMessage::new().poll(poll);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the question to be polled.
    pub fn question(self, text: impl Into<String>) -> CreatePoll<NeedsAnswers> {
        CreatePoll {
            question: CreatePollMedia {
                text: text.into(),
            },
            answers: self.answers,
            duration: self.duration,
            allow_multiselect: self.allow_multiselect,
            layout_type: self.layout_type,
            _stage: NeedsAnswers,
        }
    }
}

impl CreatePoll<NeedsAnswers> {
    /// Sets the answers that can be picked from.
    pub fn answers(self, answers: Vec<CreatePollAnswer>) -> CreatePoll<NeedsDuration> {
        CreatePoll {
            question: self.question,
            answers,
            duration: self.duration,
            allow_multiselect: self.allow_multiselect,
            layout_type: self.layout_type,
            _stage: NeedsDuration,
        }
    }
}

impl CreatePoll<NeedsDuration> {
    /// Sets the duration for the Poll to run for.
    ///
    /// This must be less than a week, and will be rounded to hours towards zero.
    pub fn duration(self, duration: std::time::Duration) -> CreatePoll<Ready> {
        let hours = duration.as_secs() / 3600;

        CreatePoll {
            question: self.question,
            answers: self.answers,
            duration: hours.try_into().unwrap_or(168),
            allow_multiselect: self.allow_multiselect,
            layout_type: self.layout_type,
            _stage: Ready,
        }
    }
}

impl<Stage: Sealed> CreatePoll<Stage> {
    /// Sets the layout type for the Poll to take.
    ///
    /// This is currently only ever [`PollLayoutType::Default`], and is optional.
    pub fn layout_type(mut self, layout_type: PollLayoutType) -> Self {
        self.layout_type = Some(layout_type);
        self
    }

    /// Allows users to select multiple answers for the Poll.
    pub fn allow_multiselect(mut self) -> Self {
        self.allow_multiselect = true;
        self
    }
}

#[derive(serde::Serialize, Clone, Debug, Default)]
#[must_use = "Builders do nothing unless built"]
pub struct CreatePollAnswer {
    poll_media: PollMedia,
}

impl CreatePollAnswer {
    /// Creates a builder for a Poll answer.
    ///
    /// [`Self::text`] or [`Self::emoji`] must be provided.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.poll_media.text = Some(text.into());
        self
    }

    pub fn emoji(mut self, emoji: impl Into<PollMediaEmoji>) -> Self {
        self.poll_media.emoji = Some(emoji.into());
        self
    }
}
