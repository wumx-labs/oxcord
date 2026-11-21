use crate::model::prelude::*;

#[allow(async_fn_in_trait)]
pub trait EmojiExt: Sized {
    #[cfg(feature = "model")]
    /// Generates a URL to the emoji's image.
    ///
    /// # Examples
    ///
    /// Print the direct link to the given emoji:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::model::guild::Emoji;
    /// #
    /// # fn run(emoji: Emoji) {
    /// // assuming emoji has been set already
    /// println!("Direct link to emoji image: {}", emoji.url());
    /// # }
    /// ```
    #[must_use]
    fn url(&self) -> String;
}

impl EmojiExt for Emoji {
    #[cfg(feature = "model")]
    #[inline]
    fn url(&self) -> String {
        let extension = if self.animated { "gif" } else { "png" };
        cdn!("/emojis/{}.{}", self.id, extension)
    }
}
