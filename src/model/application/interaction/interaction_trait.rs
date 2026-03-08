#[cfg(feature = "http")]
use crate::builder::{
    CreateInteractionResponse,
    EditInteractionResponse,
};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::channel::Message;
use async_trait::async_trait;

#[async_trait]
#[cfg(feature = "http")]
pub trait InteractionResponse {
    async fn get_interaction_response<'a>(&'a self, http: impl AsRef<Http> + std::marker::Send + std::marker::Sync) -> Result<Message>;

    async fn create_interaction_response<'a, F>(
        &'a self,
        http: impl AsRef<Http> + std::marker::Send + std::marker::Sync,
        f: F,
    ) -> Result<()>
        where for<'b> F: FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a> + std::marker::Send;

    async fn edit_original_interaction_response<'a, F>(
        &'a self,
        http: impl AsRef<Http> + std::marker::Send + std::marker::Sync,
        f: F,
    ) -> Result<Message>
        where
            F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse + std::marker::Send;

    async fn delete_original_interaction_response<'a>(&'a self, http: impl AsRef<Http> + std::marker::Send + std::marker::Sync) -> Result<()>;

    async fn defer<'a>(&'a self, http: impl AsRef<Http> + std::marker::Send + std::marker::Sync) -> Result<()>;
    fn get_locale<'a>(&'a self) -> &str;
}