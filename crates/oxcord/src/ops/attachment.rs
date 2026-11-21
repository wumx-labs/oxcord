#[cfg(feature = "model")]
use reqwest::Client as ReqwestClient;

use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait AttachmentExt: Sized {
    #[cfg(feature = "model")]
    /// If this attachment is an image, then a tuple of the width and height in pixels is returned.
    #[must_use]
    fn dimensions(&self) -> Option<(u32, u32)>;
    #[cfg(feature = "model")]
    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// use oxcord::model::prelude::*;
    /// use oxcord::prelude::*;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// # struct Handler;
    ///
    /// #[oxcord::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, mut message: Message) {
    ///         for attachment in message.attachments {
    ///             let content = match attachment.download().await {
    ///                 Ok(content) => content,
    ///                 Err(why) => {
    ///                     println!("Error downloading attachment: {:?}", why);
    ///                     let _ =
    ///                         message.channel_id.say(&context, "Error downloading attachment").await;
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let mut file = match File::create(&attachment.filename).await {
    ///                 Ok(file) => file,
    ///                 Err(why) => {
    ///                     println!("Error creating file: {:?}", why);
    ///                     let _ = message.channel_id.say(&context, "Error creating file").await;
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             if let Err(why) = file.write_all(&content).await {
    ///                 println!("Error writing to file: {:?}", why);
    ///
    ///                 return;
    ///             }
    ///
    ///             let _ = message
    ///                 .channel_id
    ///                 .say(&context, format!("Saved {:?}", attachment.filename))
    ///                 .await;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents of the HTTP response.
    ///
    /// Returns an [`Error::Http`] when there is a problem retrieving the attachment.
    ///
    /// [`Message`]: super::Message
    async fn download(&self) -> Result<Vec<u8>>;
}
impl AttachmentExt for Attachment {
    #[cfg(feature = "model")]
    fn dimensions(&self) -> Option<(u32, u32)> {
        self.width.zip(self.height)
    }
    #[cfg(feature = "model")]
    async fn download(&self) -> Result<Vec<u8>> {
        let reqwest = ReqwestClient::new();
        let bytes = reqwest.get(&self.url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}
