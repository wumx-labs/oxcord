mod from;
#[cfg(test)]
mod tests;

use serde::de::value::U8Deserializer;
use serde::de::{Deserializer, Error as DeError, IntoDeserializer, MapAccess, Unexpected, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::opcode::Opcode;
use crate::payload::*;

/// A representation of data received for voice gateway events.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    /// Used to begin a voice websocket connection.
    Identify(Identify),
    /// Used to select the voice protocol and encryption mechanism.
    SelectProtocol(SelectProtocol),
    /// Server's response to the client's Identify operation. Contains session-specific
    /// information, e.g. SSRC, and supported encryption modes.
    Ready(Ready),
    /// Periodic messages used to keep the websocket connection alive.
    Heartbeat(Heartbeat),
    /// Server's confirmation of a negotiated encryption scheme.
    SessionDescription(SessionDescription),
    /// A voice event denoting that someone is speaking.
    Speaking(Speaking),
    /// Acknowledgement from the server for a prior voice heartbeat.
    HeartbeatAck(HeartbeatAck),
    /// Sent by the client after a disconnect to attempt to resume a session.
    Resume(Resume),
    /// Used to determine how often the client must send a heartbeat.
    Hello(Hello),
    /// Message received if a Resume request was successful.
    Resumed,
    /// Status update in the current channel, indicating that a user has connected.
    ClientConnect(ClientConnect),
    /// Status update in the current channel, indicating that a user has disconnected.
    ClientDisconnect(ClientDisconnect),
    /// DAVE prepare transition event.
    DavePrepareTransition(serde_json::Value),
    /// DAVE execute transition event.
    DaveExecuteTransition(serde_json::Value),
    /// DAVE transition ready event.
    DaveTransitionReady(serde_json::Value),
    /// DAVE prepare epoch event.
    DavePrepareEpoch(serde_json::Value),
    /// DAVE MLS external sender event.
    DaveMlsExternalSender(serde_json::Value),
    /// DAVE MLS key package event.
    DaveMlsKeyPackage(serde_json::Value),
    /// DAVE MLS proposals event.
    DaveMlsProposals(serde_json::Value),
    /// DAVE MLS commit welcome event.
    DaveMlsCommitWelcome(serde_json::Value),
    /// DAVE MLS announce commit transition event.
    DaveMlsAnnounceCommitTransition(serde_json::Value),
    /// DAVE MLS welcome event.
    DaveMlsWelcome(serde_json::Value),
    /// DAVE MLS invalid commit welcome event.
    DaveMlsInvalidCommitWelcome(serde_json::Value),
}

impl Event {
    pub fn kind(&self) -> Opcode {
        use Event::*;
        match self {
            Identify(_) => Opcode::Identify,
            SelectProtocol(_) => Opcode::SelectProtocol,
            Ready(_) => Opcode::Ready,
            Heartbeat(_) => Opcode::Heartbeat,
            SessionDescription(_) => Opcode::SessionDescription,
            Speaking(_) => Opcode::Speaking,
            HeartbeatAck(_) => Opcode::HeartbeatAck,
            Resume(_) => Opcode::Resume,
            Hello(_) => Opcode::Hello,
            Resumed => Opcode::Resumed,
            ClientConnect(_) => Opcode::ClientConnect,
            ClientDisconnect(_) => Opcode::ClientDisconnect,
            DavePrepareTransition(_) => Opcode::DavePrepareTransition,
            DaveExecuteTransition(_) => Opcode::DaveExecuteTransition,
            DaveTransitionReady(_) => Opcode::DaveTransitionReady,
            DavePrepareEpoch(_) => Opcode::DavePrepareEpoch,
            DaveMlsExternalSender(_) => Opcode::DaveMlsExternalSender,
            DaveMlsKeyPackage(_) => Opcode::DaveMlsKeyPackage,
            DaveMlsProposals(_) => Opcode::DaveMlsProposals,
            DaveMlsCommitWelcome(_) => Opcode::DaveMlsCommitWelcome,
            DaveMlsAnnounceCommitTransition(_) => Opcode::DaveMlsAnnounceCommitTransition,
            DaveMlsWelcome(_) => Opcode::DaveMlsWelcome,
            DaveMlsInvalidCommitWelcome(_) => Opcode::DaveMlsInvalidCommitWelcome,
        }
    }
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Event", 2)?;

        s.serialize_field("op", &self.kind())?;

        use Event::*;
        match self {
            Identify(e) => s.serialize_field("d", e)?,
            SelectProtocol(e) => s.serialize_field("d", e)?,
            Ready(e) => s.serialize_field("d", e)?,
            Heartbeat(e) => s.serialize_field("d", e)?,
            SessionDescription(e) => s.serialize_field("d", e)?,
            Speaking(e) => s.serialize_field("d", e)?,
            HeartbeatAck(e) => s.serialize_field("d", e)?,
            Resume(e) => s.serialize_field("d", e)?,
            Hello(e) => s.serialize_field("d", e)?,
            Resumed => s.serialize_field("d", &None::<()>)?,
            ClientConnect(e) => s.serialize_field("d", e)?,
            ClientDisconnect(e) => s.serialize_field("d", e)?,
            DavePrepareTransition(e) => s.serialize_field("d", e)?,
            DaveExecuteTransition(e) => s.serialize_field("d", e)?,
            DaveTransitionReady(e) => s.serialize_field("d", e)?,
            DavePrepareEpoch(e) => s.serialize_field("d", e)?,
            DaveMlsExternalSender(e) => s.serialize_field("d", e)?,
            DaveMlsKeyPackage(e) => s.serialize_field("d", e)?,
            DaveMlsProposals(e) => s.serialize_field("d", e)?,
            DaveMlsCommitWelcome(e) => s.serialize_field("d", e)?,
            DaveMlsAnnounceCommitTransition(e) => s.serialize_field("d", e)?,
            DaveMlsWelcome(e) => s.serialize_field("d", e)?,
            DaveMlsInvalidCommitWelcome(e) => s.serialize_field("d", e)?,
        }

        s.end()
    }
}

struct EventVisitor;

impl<'de> Visitor<'de> for EventVisitor {
    type Value = Event;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with at least two keys ('d', 'op')")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut d = None;
        let mut op = None;

        loop {
            match map.next_key::<&str>()? {
                Some("op") => {
                    let raw = map.next_value::<u8>()?;
                    let des: U8Deserializer<A::Error> = raw.into_deserializer();
                    let valid_op = Opcode::deserialize(des).map_err(|_| {
                        DeError::invalid_value(
                            Unexpected::Unsigned(raw.into()),
                            &"opcode in [0--9] + [12--13] + [21--31]",
                        )
                    })?;
                    op = Some(valid_op);
                },
                // Idea: Op comes first, but missing it is not failure.
                // So, if order correct then we don't need to pass the RawValue back out.
                Some("d") => match op {
                    Some(Opcode::Identify) => return Ok(map.next_value::<Identify>()?.into()),
                    Some(Opcode::SelectProtocol) => {
                        return Ok(map.next_value::<SelectProtocol>()?.into());
                    },
                    Some(Opcode::Ready) => return Ok(map.next_value::<Ready>()?.into()),
                    Some(Opcode::Heartbeat) => return Ok(map.next_value::<Heartbeat>()?.into()),
                    Some(Opcode::HeartbeatAck) => {
                        return Ok(map.next_value::<HeartbeatAck>()?.into());
                    },
                    Some(Opcode::SessionDescription) => {
                        return Ok(map.next_value::<SessionDescription>()?.into());
                    },
                    Some(Opcode::Speaking) => return Ok(map.next_value::<Speaking>()?.into()),
                    Some(Opcode::Resume) => return Ok(map.next_value::<Resume>()?.into()),
                    Some(Opcode::Hello) => return Ok(map.next_value::<Hello>()?.into()),
                    Some(Opcode::Resumed) => {
                        let _ = map.next_value::<Option<()>>()?;
                        return Ok(Event::Resumed);
                    },
                    Some(Opcode::ClientConnect) => {
                        return Ok(map.next_value::<ClientConnect>()?.into());
                    },
                    Some(Opcode::ClientDisconnect) => {
                        return Ok(map.next_value::<ClientDisconnect>()?.into());
                    },
                    Some(Opcode::DavePrepareTransition) => {
                        return Ok(Event::DavePrepareTransition(map.next_value()?));
                    },
                    Some(Opcode::DaveExecuteTransition) => {
                        return Ok(Event::DaveExecuteTransition(map.next_value()?));
                    },
                    Some(Opcode::DaveTransitionReady) => {
                        return Ok(Event::DaveTransitionReady(map.next_value()?));
                    },
                    Some(Opcode::DavePrepareEpoch) => {
                        return Ok(Event::DavePrepareEpoch(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsExternalSender) => {
                        return Ok(Event::DaveMlsExternalSender(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsKeyPackage) => {
                        return Ok(Event::DaveMlsKeyPackage(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsProposals) => {
                        return Ok(Event::DaveMlsProposals(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsCommitWelcome) => {
                        return Ok(Event::DaveMlsCommitWelcome(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsAnnounceCommitTransition) => {
                        return Ok(Event::DaveMlsAnnounceCommitTransition(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsWelcome) => {
                        return Ok(Event::DaveMlsWelcome(map.next_value()?));
                    },
                    Some(Opcode::DaveMlsInvalidCommitWelcome) => {
                        return Ok(Event::DaveMlsInvalidCommitWelcome(map.next_value()?));
                    },
                    None => {
                        d = Some(map.next_value::<&RawValue>()?);
                    },
                },
                Some(_) => {},
                None =>
                    if d.is_none() {
                        return Err(DeError::missing_field("d"));
                    } else if op.is_none() {
                        return Err(DeError::missing_field("op"));
                    },
            }

            if d.is_some() && op.is_some() {
                break;
            }
        }

        let d = d.expect("Struct body known to exist if loop has been escaped.").get();
        let op = op.expect("Struct variant known to exist if loop has been escaped.");

        (match op {
            Opcode::Identify => serde_json::from_str::<Identify>(d).map(Into::into),
            Opcode::SelectProtocol => serde_json::from_str::<SelectProtocol>(d).map(Into::into),
            Opcode::Ready => serde_json::from_str::<Ready>(d).map(Into::into),
            Opcode::Heartbeat => serde_json::from_str::<Heartbeat>(d).map(Into::into),
            Opcode::HeartbeatAck => serde_json::from_str::<HeartbeatAck>(d).map(Into::into),
            Opcode::SessionDescription => {
                serde_json::from_str::<SessionDescription>(d).map(Into::into)
            },
            Opcode::Speaking => serde_json::from_str::<Speaking>(d).map(Into::into),
            Opcode::Resume => serde_json::from_str::<Resume>(d).map(Into::into),
            Opcode::Hello => serde_json::from_str::<Hello>(d).map(Into::into),
            Opcode::Resumed => Ok(Event::Resumed),
            Opcode::ClientConnect => serde_json::from_str::<ClientConnect>(d).map(Into::into),
            Opcode::ClientDisconnect => serde_json::from_str::<ClientDisconnect>(d).map(Into::into),
            Opcode::DavePrepareTransition => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DavePrepareTransition)
            },
            Opcode::DaveExecuteTransition => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveExecuteTransition)
            },
            Opcode::DaveTransitionReady => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveTransitionReady)
            },
            Opcode::DavePrepareEpoch => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DavePrepareEpoch)
            },
            Opcode::DaveMlsExternalSender => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsExternalSender)
            },
            Opcode::DaveMlsKeyPackage => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsKeyPackage)
            },
            Opcode::DaveMlsProposals => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsProposals)
            },
            Opcode::DaveMlsCommitWelcome => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsCommitWelcome)
            },
            Opcode::DaveMlsAnnounceCommitTransition => serde_json::from_str::<serde_json::Value>(d)
                .map(Event::DaveMlsAnnounceCommitTransition),
            Opcode::DaveMlsWelcome => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsWelcome)
            },
            Opcode::DaveMlsInvalidCommitWelcome => {
                serde_json::from_str::<serde_json::Value>(d).map(Event::DaveMlsInvalidCommitWelcome)
            },
        })
        .map_err(DeError::custom)
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EventVisitor)
    }
}
