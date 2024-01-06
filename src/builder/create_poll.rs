use std::borrow::Cow;

use crate::model::channel::{PollLayoutType, PollMediaEmoji};

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
struct CreatePollMedia<'a> {
    text: Cow<'a, str>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[must_use = "Builders do nothing unless built"]
pub struct CreatePoll<'a, Stage: Sealed> {
    question: CreatePollMedia<'a>,
    answers: Cow<'a, [CreatePollAnswer<'a>]>,
    duration: u8,
    allow_multiselect: bool,
    layout_type: Option<PollLayoutType>,

    #[serde(skip)]
    _stage: Stage,
}

impl Default for CreatePoll<'_, NeedsQuestion> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            question: CreatePollMedia {
                text: Cow::default(),
            },
            answers: Cow::default(),
            duration: u8::default(),
            allow_multiselect: false,
            layout_type: None,

            _stage: NeedsQuestion,
        }
    }
}

impl<'a> CreatePoll<'a, NeedsQuestion> {
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
    pub fn question(self, text: impl Into<Cow<'a, str>>) -> CreatePoll<'a, NeedsAnswers> {
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

impl<'a> CreatePoll<'a, NeedsAnswers> {
    /// Sets the answers that can be picked from.
    pub fn answers(
        self,
        answers: impl Into<Cow<'a, [CreatePollAnswer<'a>]>>,
    ) -> CreatePoll<'a, NeedsDuration> {
        CreatePoll {
            question: self.question,
            answers: answers.into(),
            duration: self.duration,
            allow_multiselect: self.allow_multiselect,
            layout_type: self.layout_type,
            _stage: NeedsDuration,
        }
    }
}

impl<'a> CreatePoll<'a, NeedsDuration> {
    /// Sets the duration for the Poll to run for.
    ///
    /// This must be less than a week, and will be rounded to hours towards zero.
    pub fn duration(self, duration: std::time::Duration) -> CreatePoll<'a, Ready> {
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

impl<Stage: Sealed> CreatePoll<'_, Stage> {
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
struct CreatePollAnswerMedia<'a> {
    text: Option<Cow<'a, str>>,
    emoji: Option<PollMediaEmoji>,
}

#[derive(serde::Serialize, Clone, Debug, Default)]
#[must_use = "Builders do nothing unless built"]
pub struct CreatePollAnswer<'a> {
    poll_media: CreatePollAnswerMedia<'a>,
}

impl<'a> CreatePollAnswer<'a> {
    /// Creates a builder for a Poll answer.
    ///
    /// [`Self::text`] or [`Self::emoji`] must be provided.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.poll_media.text = Some(text.into());
        self
    }

    pub fn emoji(mut self, emoji: impl Into<PollMediaEmoji>) -> Self {
        self.poll_media.emoji = Some(emoji.into());
        self
    }
}
