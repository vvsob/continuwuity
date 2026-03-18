use conduwuit::Result;
use ruma::{
	OwnedRoomId, OwnedUserId,
	api::appservice::event::push_events::v1::EphemeralData,
	events::{
		presence::PresenceEvent,
		receipt::{ReceiptEvent, ReceiptEventContent},
		typing::{TypingEvent, TypingEventContent},
	},
};

use super::EduBuf;

pub(crate) fn serialize(ephemeral: &EphemeralData) -> Result<EduBuf> {
	let mut buf = EduBuf::new();
	serde_json::to_writer(&mut buf, ephemeral)?;
	Ok(buf)
}

#[must_use]
pub(crate) fn typing(room_id: OwnedRoomId, user_ids: Vec<OwnedUserId>) -> EphemeralData {
	EphemeralData::Typing(TypingEvent {
		content: TypingEventContent { user_ids },
		room_id,
	})
}

#[must_use]
pub(crate) fn receipt(content: ReceiptEventContent, room_id: OwnedRoomId) -> EphemeralData {
	EphemeralData::Receipt(ReceiptEvent { content, room_id })
}

#[must_use]
pub(crate) fn presence(event: PresenceEvent) -> EphemeralData { EphemeralData::Presence(event) }

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use ruma::{
		event_id, owned_room_id, owned_user_id,
		events::{
			presence::{PresenceEvent, PresenceEventContent},
			receipt::{Receipt, ReceiptEventContent, ReceiptType},
		},
		presence::PresenceState,
	};
	use serde_json::{json, to_value as to_json_value};

	use super::{presence, receipt, serialize, typing};

	#[test]
	fn typing_serializes_as_appservice_ephemeral() {
		let ephemeral = typing(
			owned_room_id!("!room:example.org"),
			vec![owned_user_id!("@alice:example.org")],
		);

		assert_eq!(
			to_json_value(ephemeral).unwrap(),
			json!({
				"type": "m.typing",
				"room_id": "!room:example.org",
				"content": {
					"user_ids": ["@alice:example.org"],
				},
			})
		);
	}

	#[test]
	fn receipt_serializes_as_appservice_ephemeral() {
		let content = ReceiptEventContent(BTreeMap::from([(
			event_id!("$event:example.org").to_owned(),
			BTreeMap::from([(
				ReceiptType::Read,
				BTreeMap::from([(owned_user_id!("@alice:example.org"), Receipt::default())]),
			)]),
		)]));
		let ephemeral = receipt(content, owned_room_id!("!room:example.org"));

		assert_eq!(
			to_json_value(ephemeral).unwrap(),
			json!({
				"type": "m.receipt",
				"room_id": "!room:example.org",
				"content": {
					"$event:example.org": {
						"m.read": {
							"@alice:example.org": {},
						},
					},
				},
			})
		);
	}

	#[test]
	fn presence_serializes_as_appservice_ephemeral() {
		let ephemeral = presence(PresenceEvent {
			content: PresenceEventContent {
				avatar_url: None,
				currently_active: Some(true),
				displayname: None,
				last_active_ago: None,
				presence: PresenceState::Online,
				status_msg: Some("available".to_owned()),
			},
			sender: owned_user_id!("@alice:example.org"),
		});

		assert_eq!(
			to_json_value(ephemeral).unwrap(),
			json!({
				"type": "m.presence",
				"sender": "@alice:example.org",
				"content": {
					"currently_active": true,
					"presence": "online",
					"status_msg": "available",
				},
			})
		);
	}

	#[test]
	fn serialize_returns_appservice_event_bytes() {
		let bytes = serialize(&typing(
			owned_room_id!("!room:example.org"),
			vec![owned_user_id!("@alice:example.org")],
		))
		.unwrap();

		assert_eq!(
			serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
			json!({
				"type": "m.typing",
				"room_id": "!room:example.org",
				"content": {
					"user_ids": ["@alice:example.org"],
				},
			})
		);
	}
}
