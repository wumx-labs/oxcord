use serde_cow::CowStr;

#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::is_false;

pub(crate) fn base64_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::Engine as _;
    use serde::de::Error;

    let base64 = <Option<CowStr<'de>>>::deserialize(deserializer)?;
    let bytes = match base64 {
        Some(CowStr(base64)) => {
            Some(base64::prelude::BASE64_STANDARD.decode(&*base64).map_err(D::Error::custom)?)
        },
        None => None,
    };
    Ok(bytes)
}

/// A file uploaded with a message. Not to be confused with [`Embed`]s.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#attachment-object).
///
/// [`Embed`]: super::Embed
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: AttachmentId,
    /// The filename of the file that was uploaded. This is equivalent to what the uploader had
    /// their file named.
    pub filename: String,
    /// Description for the file (max 1024 characters).
    pub description: Option<String>,
    /// If the attachment is an image, then the height of the image is provided.
    pub height: Option<u32>,
    /// The proxy URL.
    pub proxy_url: String,
    /// The size of the file in bytes.
    pub size: u32,
    /// The URL of the uploaded attachment.
    pub url: String,
    /// If the attachment is an image, then the width of the image is provided.
    pub width: Option<u32>,
    /// The attachment's [media type].
    ///
    /// [media type]: https://en.wikipedia.org/wiki/Media_type
    pub content_type: Option<String>,
    /// Whether this attachment is ephemeral.
    ///
    /// Ephemeral attachments will automatically be removed after a set period of time.
    ///
    /// Ephemeral attachments on messages are guaranteed to be available as long as the message
    /// itself exists.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ephemeral: bool,
    /// The duration of the audio file (present if [`MessageFlags::IS_VOICE_MESSAGE`]).
    pub duration_secs: Option<f64>,
    /// List of bytes representing a sampled waveform (present if
    /// [`MessageFlags::IS_VOICE_MESSAGE`]).
    ///
    /// The waveform is intended to be a preview of the entire voice message, with 1 byte per
    /// datapoint. Clients sample the recording at most once per 100 milliseconds, but will
    /// downsample so that no more than 256 datapoints are in the waveform.
    ///
    /// The waveform details are a Discord implementation detail and may change without warning or
    /// documentation.
    #[serde(default, deserialize_with = "base64_bytes")]
    pub waveform: Option<Vec<u8>>,
}
