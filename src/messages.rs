use serde::{Deserialize, de::{Visitor, MapAccess}, Deserializer};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Pixel {
	pub x: usize,
	pub y: usize,
	pub color: u8,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
	id: usize,
	time: u64,
	expiry: Option<u64>,
	who: String,
	title: String,
	content: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Purge {
	initiator: String,
	reason: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
	display_name: String,
	tooltip: String,
	css_icon: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StrippedFaction {
	id: usize,
	name: String,
	tag: Option<String>,
	color: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
	id: u64,
	author: String,
	date: u64,
	#[serde(rename = "message_raw")]
	message_raw: String,
	purge: Option<Purge>,
	badges: Vec<Badge>,
	author_name_color: i32,
	author_was_shadow_banned: Option<bool>,
	stripped_faction: Option<StrippedFaction>,
}

#[derive(Debug)]
pub struct UserUpdate {
	name_color: Option<isize>,
	displayed_faction: Option<Option<UserFaction>>,
}

struct MapPropVisitor;

impl<'de> Visitor<'de> for MapPropVisitor {
	type Value = UserUpdate;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a map of keys to values")
	}

	fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error> where M: MapAccess<'de> {
		let mut update = UserUpdate {
			name_color: None,
			displayed_faction: None,
		};

		while let Some(key) = access.next_key()? {
			match key {
				"NameColor" => update.name_color = Some(access.next_value()?),
				"DisplayedFaction" => update.displayed_faction = Some(access.next_value()?),
				// TODO: maybe warn otherwise
				_ => (),
			}
		}

		Ok(update)
	}
}

impl<'de> Deserialize<'de> for UserUpdate {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		deserializer.deserialize_map(MapPropVisitor)
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFaction {
	id: usize,
	color: u32,
	name: String,
	tag: String,
	owner: String,
	canvas_code: String,
	#[serde(rename = "creation_ms")]
	creation_ms: u64,
	member_count: usize,
	user_joined: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: usize,
    stacked: usize,
    chat_name_color: isize,
	#[serde(rename = "signup_time")]
    signup_time: u64,
    username: String,
    cooldown_expiry: u64,
    login_with_IP: bool,
    signup_IP: String,
    pixel_count: usize,
    pixel_count_all_time: usize,
    ban_expiry: Option<u64>,
    is_perma_chatbanned: bool,
    shadow_banned: bool,
    chatban_expiry: u64,
    is_rename_requested: bool,
    discord_name: String,
    chatban_reason: String,
    displayed_faction: Option<usize>,
    faction_blocked: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatBan {
	id: usize,
	target: usize,
	initiator: usize,
	when: u64,
	r#type: String,
	expiry: u64,
	reason: String,
	purged: bool,
	#[serde(rename = "target_name")]
	target_name: String,
	#[serde(rename = "initiator_name")]
	initiator_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum AcknowledgeType {
	Place,
	Undo,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlacementOverrides {
	ignore_cooldown: Option<bool>,
	can_place_any_color: Option<bool>,
	ignore_placemap: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Role {
	id: usize,
	name: String,
	guest: bool,
	default_role: bool,
	inherits: Vec<Role>,
	badges: Vec<Badge>,
	permissions: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
	Pixel { pixels: Vec<Pixel> },
	Users { count: usize },
	Alert { sender: String, message: String },
	Notification { notification: Notification },
	ChatMessage { message: ChatMessage },
	ChatUserUpdate { who: String, updates: UserUpdate },
	FactionUpdate { faction: UserFaction },
	FactionClear { fid: usize },
	ChatHistory { messages: Vec<ChatMessage> },
	MessageCooldown { diff: usize, message: String },
	ChatLookup { target: User, history: Vec<ChatMessage>, chatbans: Vec<ChatBan> },
	ChatBan { permanent: bool, reason: String, expiry: u64 },
	ChatBanState { permanent: bool, reason: String, expiry: u64 },
	ChatPurge { target: String, initiator: String, amount: usize, reason: String, announce: bool },
	ChatPurgeSpecific { target: String, initiator: String, IDs: Vec<usize>, reason: String, announce: bool },
	#[serde(rename = "ACK")]
	#[serde(rename_all = "camelCase")]
	Acknowledge { ack_for: AcknowledgeType, x: usize, y: usize },
	#[serde(rename_all = "camelCase")]
	AdminPlacementOverrides { placement_overrides: PlacementOverrides },
	CaptchaRequired,
	CaptchaStatus { success: bool },
	CanUndo { time: u64 },
	Cooldown { wait: f32 },
	ReceivedReport { report_id: usize, report_type: String },
	Pixels { count: usize, cause: String },
	#[serde(rename_all = "camelCase")]
	Userinfo {
		username: String,
		roles: Vec<Role>,
		pixel_count: usize,
		pixel_count_all_time: usize,
		banned: bool,
		ban_expiry: Option<u64>,
		ban_reason: Option<String>,
		method: String,
		placement_overrides: PlacementOverrides,
		chat_banned: bool,
		chatban_reason: Option<String>,
		chatban_is_perma: Option<bool>,
		chatban_expiry: Option<u64>,
		rename_requested: bool,
		discord_name: Option<String>,
		chat_name_color: isize,
	},
	#[serde(rename = "pixelCounts")]
	#[serde(rename_all = "camelCase")]
	PixelCounts { pixel_count: usize, pixel_count_all_time: usize },
	Rename { requested: bool },
	#[serde(rename_all = "camelCase")]
	RenameSuccess { new_name: String },
}