
use async_trait::async_trait;

use crate::messages::{
	AcknowledgeType,
	PlacementOverrides, ChatMessage, ChatBan, UserUpdate, UserFaction, Notification, Pixel, Role, User,
};

#[async_trait]
pub trait EventHandler: Send + Sync {
	async fn handle_ready(&self) {}
	async fn handle_disconnect(&self) {}

	async fn handle_acknowledge(
		&self,
		acknowledge_for: AcknowledgeType,
		x: usize,
		y: usize,
	) {}

	async fn handle_overrides(
		&self,
		overrides: PlacementOverrides,
	) {}

	async fn handle_alert(
		&self,
		sender: String,
		message: String,
	) {}

	async fn handle_can_undo(
		&self,
		time: u64,
	) {}

	async fn handle_captcha_status(
		&self,
		success: bool,
	) {}

	async fn handle_captcha_required(
		&self,
	) {}
	
	async fn handle_chatban(
		&self,
		permanent: bool,
		reason: String,
		expiry: u64,
	) {}

	async fn handle_chatban_state(
		&self,
		permanent: bool,
		reason: String,
		expiry: u64,
	) {}

	async fn handle_chat_history(
		&self,
		messages: Vec<ChatMessage>,
	) {}

	async fn handle_chat_lookup(
		&self,
		target: User,
		history: Vec<ChatMessage>,
		chatbans: Vec<ChatBan>,
	) {}

	async fn handle_chat_message(
		&self,
		messages: ChatMessage,
	) {}

	async fn handle_chat_purge(
		&self,
		target: String,
		initiator: String,
		amount: usize,
		reason: String,
		announce: bool,
	) {}

	async fn handle_chat_purge_specific(
		&self,
		target: String,
		initiator: String,
		ids: Vec<usize>,
		reason: String,
		announce: bool,
	) {}

	async fn handle_chat_user_update(
		&self,
		who: String,
		updates: UserUpdate,
	) {}

	async fn handle_cooldown(
		&self,
		wait: f32,
	) {}

	async fn handle_faction_clear(
		&self,
		faction_id: usize,
	) {}

	async fn handle_faction_update(
		&self,
		faction: UserFaction,
	) {}

	async fn handle_message_cooldown(
		&self,
		diff: usize,
		message: String,
	) {}

	async fn handle_notification(
		&self,
		notification: Notification,
	) {}

	async fn handle_board_update(
		&self,
		pixels: Vec<Pixel>,
	) {}

	async fn handle_pixel_counts(
		&self,
		count: usize,
		all_time: usize,
	) {}

	async fn handle_pixels_available(
		&self,
		count: usize,
		cause: String,
	) {}

	async fn handle_received_report(
		&self,
		report_id: usize,
		report_type: String,
	) {}

	async fn handle_rename(
		&self,
		requested: bool,
	) {}

	async fn handle_rename_success(
		&self,
		new_name: String,
	) {}

	async fn handle_user_info(
		&self,
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
		count: usize,
	) {}

	async fn handle_unknown(
		&self,
		packet: String,
	) {}
}