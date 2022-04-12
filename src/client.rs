use hyper::client::HttpConnector;
use url::Url;
use tokio_tungstenite::{connect_async, tungstenite::Error};

use futures_util::{StreamExt, pin_mut};

use std::sync::Arc;

use crate::event_handler::EventHandler;
use crate::messages::Message;

#[derive(Default)]
pub struct ClientBuidler {
	site_base: Option<Url>,
	event_handler: Option<Arc<dyn EventHandler>>,
}

#[derive(Debug)]
pub enum ClientBuildError {
	MissingSite,
	MissingEventHandler,
}

impl ClientBuidler {
	pub fn site(mut self, base: Url) -> Self {
		self.site_base = Some(base);
		self
	}

	pub fn event_handler<H: EventHandler + 'static>(mut self, handler: H) -> Self {
		self.event_handler = Some(Arc::new(handler));
		self
	}

	pub fn build(self) -> Result<Client, ClientBuildError> {
		Ok(Client {
			site_base: self.site_base.ok_or(ClientBuildError::MissingSite)?,
			event_handler: self.event_handler.ok_or(ClientBuildError::MissingEventHandler)?,
			http_client: hyper::Client::new(),
		})
	}
}

#[derive(Debug)]
pub enum ConnectError {
	InvalidSiteScheme(String),
	WebsocketConnectFailed(Error),
}

pub struct Client {
	site_base: Url,
	event_handler: Arc<dyn EventHandler>,
	http_client: hyper::Client<HttpConnector>,
}

impl Client {
	pub fn builder() -> ClientBuidler {
		ClientBuidler::default()
	}

	pub async fn start(&self) -> Result<(), ConnectError> {
		let mut ws_url = self.site_base.join("ws").unwrap();

		match ws_url.scheme() {
			"http" => ws_url.set_scheme("ws").unwrap(),
			"https" => ws_url.set_scheme("wss").unwrap(),
			s => return Err(ConnectError::InvalidSiteScheme(s.to_owned())),
		};

		let (ws_stream, _) = connect_async(ws_url)
			.await
			.map_err(ConnectError::WebsocketConnectFailed)?;
			
		let (write, read) = ws_stream.split();

		let stream = read.for_each(|message| async {
			let text = message.unwrap().into_text().expect("Websocket to send text");

			match serde_json::from_str::<Message>(&text) {
				Ok(Message::Acknowledge { ack_for, x, y }) => {
					self.event_handler.handle_acknowledge(ack_for, x, y).await
				},
				Ok(Message::AdminPlacementOverrides { placement_overrides }) => {
					self.event_handler.handle_overrides(placement_overrides).await
				},
				Ok(Message::Alert { sender, message }) => {
					self.event_handler.handle_alert(sender, message).await
				},
				Ok(Message::CanUndo { time }) => {
					self.event_handler.handle_can_undo(time).await
				},
				Ok(Message::CaptchaRequired) => {
					self.event_handler.handle_captcha_required().await
				},
				Ok(Message::CaptchaStatus { success }) => {
					self.event_handler.handle_captcha_status(success).await
				},
				Ok(Message::ChatBan { permanent, reason, expiry }) => {
					self.event_handler.handle_chatban(permanent, reason, expiry).await
				},
				Ok(Message::ChatBanState { permanent, reason, expiry }) => {
					self.event_handler.handle_chatban_state(permanent, reason, expiry).await
				},
				Ok(Message::ChatHistory { messages }) => {
					self.event_handler.handle_chat_history(messages).await
				},
				Ok(Message::ChatLookup { target, history, chatbans }) => {
					self.event_handler.handle_chat_lookup(target, history, chatbans).await
				},
				Ok(Message::ChatMessage { message }) => {
					self.event_handler.handle_chat_message(message).await
				},
				Ok(Message::ChatPurge { target, initiator, amount, reason, announce }) => {
					self.event_handler.handle_chat_purge(target, initiator, amount, reason, announce).await
				},
				Ok(Message::ChatPurgeSpecific { target, initiator, IDs, reason, announce }) => {
					self.event_handler.handle_chat_purge_specific(target, initiator, IDs, reason, announce).await
				},
				Ok(Message::ChatUserUpdate { who, updates }) => {
					self.event_handler.handle_chat_user_update(who, updates).await
				},
				Ok(Message::Cooldown { wait }) => {
					self.event_handler.handle_cooldown(wait).await
				},
				Ok(Message::FactionClear { fid }) => {
					self.event_handler.handle_faction_clear(fid).await
				},
				Ok(Message::FactionUpdate { faction }) => {
					self.event_handler.handle_faction_update(faction).await
				},
				Ok(Message::MessageCooldown { diff, message }) => {
					self.event_handler.handle_message_cooldown(diff, message).await
				},
				Ok(Message::Notification { notification }) => {
					self.event_handler.handle_notification(notification).await
				},
				Ok(Message::Pixel { pixels }) => {
					self.event_handler.handle_board_update(pixels).await
				},
				Ok(Message::PixelCounts { pixel_count, pixel_count_all_time }) => {
					self.event_handler.handle_pixel_counts(pixel_count, pixel_count_all_time).await
				},
				Ok(Message::Pixels { count, cause }) => {
					self.event_handler.handle_pixels_available(count, cause).await
				},
				Ok(Message::ReceivedReport { report_id, report_type }) => {
					self.event_handler.handle_received_report(report_id, report_type).await
				},
				Ok(Message::Rename { requested }) => {
					self.event_handler.handle_rename(requested).await
				},
				Ok(Message::RenameSuccess { new_name }) => {
					self.event_handler.handle_rename_success(new_name).await
				},
				Ok(Message::Userinfo { username, roles, pixel_count, pixel_count_all_time, banned, ban_expiry, ban_reason, method, placement_overrides, chat_banned, chatban_reason, chatban_is_perma, chatban_expiry, rename_requested, discord_name, chat_name_color }) => {
					self.event_handler.handle_user_info(username, roles, pixel_count, pixel_count_all_time, banned, ban_expiry, ban_reason, method, placement_overrides, chat_banned, chatban_reason, chatban_is_perma, chatban_expiry, rename_requested, discord_name, chat_name_color).await
				},
				Ok(Message::Users { count }) => {
					self.event_handler.handle_user_count(count).await
				}
				Err(_) => {
					self.event_handler.handle_unknown(text).await
				},
			}
		});

		pin_mut!(stream);
		stream.await;
		Ok(())
	}
}