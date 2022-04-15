
use async_trait::async_trait;

use crate::{messages::{
	AcknowledgeType,
	PlacementOverrides, ChatMessage, ChatBan, UserUpdate, UserFaction, Notification, Pixel, Role, User,
}, Client};

#[async_trait]
pub trait EventHandler: Send + Sync {
	async fn handle_ready(&self, client: &Client) {}
	async fn handle_disconnect(&self, client: &Client) {}

	async fn handle_acknowledge(
		&self,
		client: &Client,
		acknowledge_for: AcknowledgeType,
		x: usize,
		y: usize,
	) {}

	async fn handle_overrides(
		&self,
		client: &Client,
		overrides: PlacementOverrides,
	) {}

	async fn handle_alert(
		&self,
		client: &Client,
		sender: String,
		message: String,
	) {}

	async fn handle_can_undo(
		&self,
		client: &Client,
		time: u64,
	) {}

	async fn handle_captcha_status(
		&self,
		client: &Client,
		success: bool,
	) {}

	async fn handle_captcha_required(
		&self,
		client: &Client,
	) {}
	
	async fn handle_chatban(
		&self,
		client: &Client,
		permanent: bool,
		reason: String,
		expiry: u64,
	) {}

	async fn handle_chatban_state(
		&self,
		client: &Client,
		permanent: bool,
		reason: String,
		expiry: u64,
	) {}

	async fn handle_chat_history(
		&self,
		client: &Client,
		messages: Vec<ChatMessage>,
	) {}

	async fn handle_chat_lookup(
		&self,
		client: &Client,
		target: User,
		history: Vec<ChatMessage>,
		chatbans: Vec<ChatBan>,
	) {}

	async fn handle_chat_message(
		&self,
		client: &Client,
		messages: ChatMessage,
	) {}

	async fn handle_chat_purge(
		&self,
		client: &Client,
		target: String,
		initiator: String,
		amount: usize,
		reason: String,
		announce: bool,
	) {}

	async fn handle_chat_purge_specific(
		&self,
		client: &Client,
		target: String,
		initiator: String,
		ids: Vec<usize>,
		reason: String,
		announce: bool,
	) {}

	async fn handle_chat_user_update(
		&self,
		client: &Client,
		who: String,
		updates: UserUpdate,
	) {}

	async fn handle_cooldown(
		&self,
		client: &Client,
		wait: f32,
	) {}

	async fn handle_faction_clear(
		&self,
		client: &Client,
		faction_id: usize,
	) {}

	async fn handle_faction_update(
		&self,
		client: &Client,
		faction: UserFaction,
	) {}

	async fn handle_message_cooldown(
		&self,
		client: &Client,
		diff: usize,
		message: String,
	) {}

	async fn handle_notification(
		&self,
		client: &Client,
		notification: Notification,
	) {}

	async fn handle_board_update(
		&self,
		client: &Client,
		pixels: Vec<Pixel>,
	) {}

	async fn handle_pixel_counts(
		&self,
		client: &Client,
		count: usize,
		all_time: usize,
	) {}

	async fn handle_pixels_available(
		&self,
		client: &Client,
		count: usize,
		cause: String,
	) {}

	async fn handle_received_report(
		&self,
		client: &Client,
		report_id: usize,
		report_type: String,
	) {}

	async fn handle_rename(
		&self,
		client: &Client,
		requested: bool,
	) {}

	async fn handle_rename_success(
		&self,
		client: &Client,
		new_name: String,
	) {}

	async fn handle_user_info(
		&self,
		client: &Client,
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
	) {}

	async fn handle_user_count(
		&self,
		client: &Client,
		count: usize,
	) {}

	async fn handle_unknown(
		&self,
		client: &Client,
		packet: String,
	) {}
}