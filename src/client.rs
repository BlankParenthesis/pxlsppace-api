use hyper::client::HttpConnector;
use hyper_openssl::HttpsConnector;
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};
use url::Url;
use tokio_tungstenite::{connect_async, tungstenite::Error};

use futures_util::{StreamExt, pin_mut};

use std::sync::Arc;
use std::time::{SystemTime, Duration};

use crate::Pixel;
use crate::event_handler::EventHandler;
use crate::messages::Message;

type Cache<T> = Mutex<Option<Arc<RwLock<T>>>>;

#[derive(Default)]
pub struct ClientCache {
	info: Cache<BoardInfo>,
	colors: Cache<Vec<u8>>,
	initial: Cache<Vec<u8>>,
	mask: Cache<Vec<u8>>,
	timestamps: Cache<Vec<u32>>,
	created_at: Cache<SystemTime>,
	// TODO: user count can definitely be here
}

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

	pub fn event_handler<H: EventHandler + 'static>(mut self, handler: Arc<H>) -> Self {
		self.event_handler = Some(handler);
		self
	}

	pub fn build(self) -> Result<Client, ClientBuildError> {
		Ok(Client {
			site_base: self.site_base.ok_or(ClientBuildError::MissingSite)?,
			event_handler: self.event_handler.ok_or(ClientBuildError::MissingEventHandler)?,
			http_client: hyper::Client::builder()
				.build(hyper_openssl::HttpsConnector::new().unwrap()),
			cache: ClientCache::default(),
		})
	}
}

#[derive(Debug)]
pub enum ConnectError {
	InvalidSiteScheme(String),
	WebsocketConnectFailed(Error),
	InfoFailed(RequestError),
}

#[derive(Debug)]
pub enum RequestError {
	Http(hyper::Error),
	Buffer(hyper::Error),
	ParseUTF8(std::str::Utf8Error),
	ParseJSON(serde_json::Error),
}


#[derive(Deserialize, Debug)]
pub struct Color {
	pub name: String,
	pub value: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="lowercase")]
pub enum CooldownType {
	Activity,
	Static,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct ActivityCooldown {
	pub steepness: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct CooldownInfo {
	pub r#type: CooldownType,
	pub static_cooldown_seconds: usize,
	pub activity_cooldown: ActivityCooldown,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct AuthService {
	pub id: String,
	pub name: String,
	pub registration_enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct Emoji {
	pub emoji: String,
	pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct BoardInfo {
	pub canvas_code: String,
	pub width: usize,
	pub height: usize,
	pub palette: Vec<Color>,
	pub cooldown_info: CooldownInfo,
	pub captcha_key: String,
	pub heatmap_cooldown: usize,
	pub max_stacked: usize,
	pub auth_services: std::collections::HashMap<String, AuthService>,
	pub registration_enabled: bool,
	pub chat_enabled: bool,
	pub chat_respects_canvas_ban: bool,
	pub chat_character_limit: usize,
	pub chat_banner_text: Vec<String>,
	pub snip_mode: bool,
	pub custom_emoji: Vec<Emoji>,
	pub cors_base: String,
	#[serde(with = "serde_with::rust::string_empty_as_none")]
	pub cors_param: Option<String>,
	pub chat_ratelimit_message: String,
}

pub struct Client {
	site_base: Url,
	event_handler: Arc<dyn EventHandler>,
	http_client: hyper::Client<HttpsConnector<HttpConnector>>,
	cache: ClientCache,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").field("site_base", &self.site_base).finish()
    }
}

enum BufferType {
	Colormap,
	Placemap,
	Heatmap,
	Virginmap,
	InitialColormap,
}

impl From<BufferType> for &str {
	fn from(buffer: BufferType) -> Self {
		match buffer {
			BufferType::Colormap => "boarddata",
			BufferType::Placemap => "placemap",
			BufferType::Heatmap => "heatmap",
			BufferType::Virginmap => "virginmap",
			BufferType::InitialColormap => "initialboarddata",
		}
	}
}

impl Client {
	pub fn builder() -> ClientBuidler {
		ClientBuidler::default()
	}

	pub async fn info(&self) -> Result<Arc<RwLock<BoardInfo>>, RequestError> {
		let mut info = self.cache.info.lock().await;
		if info.is_none() {
			let location = self.site_base.join("info").unwrap();
			let request = self.http_client.get(location.as_str().parse().unwrap()).await;

			let info_data = match request {
				Ok(response) => {
					hyper::body::to_bytes(response.into_body())
						.await
						.map_err(RequestError::Buffer)
						.and_then(|body| {
							let text = std::str::from_utf8(&body)
								.map_err(RequestError::ParseUTF8)?;
							serde_json::from_str(text)
								.map_err(RequestError::ParseJSON)
						})
						
				},
				Err(e) => Err(RequestError::Http(e)),
			}?;

			*info = Some(Arc::new(RwLock::new(info_data)));
		}

		Ok(info.as_ref().unwrap().clone())
	}

	async fn fetch_buffer(&self, buffer: BufferType) -> Result<Vec<u8>, RequestError> {
		let location = self.site_base.join(buffer.into()).unwrap();
		let request = self.http_client.get(location.as_str().parse().unwrap()).await;

		match request {
			Ok(response) => {
				hyper::body::to_bytes(response.into_body()).await
					.map(|body| body.to_vec())
					.map_err(RequestError::Buffer)
			},
			Err(e) => Err(RequestError::Http(e)),
		}
	}

	pub async fn colors(&self) -> Result<Arc<RwLock<Vec<u8>>>, RequestError> {
		let mut colors = self.cache.colors.lock().await;
		if colors.is_none() {
			let buffer = self.fetch_buffer(BufferType::Colormap).await?;

			*colors = Some(Arc::new(RwLock::new(buffer)));
		}

		Ok(colors.as_ref().unwrap().clone())
	}

	pub async fn initial_colors(&self) -> Result<Arc<RwLock<Vec<u8>>>, RequestError> {
		let mut initial = self.cache.initial.lock().await;
		if initial.is_none() {
			let buffer = self.fetch_buffer(BufferType::InitialColormap).await?;

			*initial = Some(Arc::new(RwLock::new(buffer)));
		}

		Ok(initial.as_ref().unwrap().clone())
	}

	pub async fn mask(&self) -> Result<Arc<RwLock<Vec<u8>>>, RequestError> {
		let mut mask = self.cache.mask.lock().await;
		if mask.is_none() {
			let buffer = self.fetch_buffer(BufferType::Placemap).await?;

			*mask = Some(Arc::new(RwLock::new(buffer)));
		}

		Ok(mask.as_ref().unwrap().clone())
	}

	pub async fn timestamps(&self) -> Result<Arc<RwLock<Vec<u32>>>, RequestError> {
		// we can generate a somewhat accurate timestamp buffer by merging the
		// heatmap and the virginmap — the heatmap tells us somewhat accurate 
		// times from the last few hours. Heatmap values of 0 can be interpreted
		// as either untouched or as one higher than minimum based on virginmap.

		let mut timestamps = self.cache.timestamps.lock().await;
		if timestamps.is_none() {
			let info = self.info().await?;
			let info = info.read().await;

			let now = SystemTime::now();
			let mut created_at = self.cache.created_at.lock().await;
			let canvas_start = created_at.get_or_insert_with(|| {
				// We can compute the canvas start time as `now - heatmap_cooldown`.
				// This is not entirely accurate, but it will suffice. 
				// (We could be more accurate by accounting for the lowest
				// non-virgin heatmap value, but scanning the entire heatmap and
				// virginmap twice doesn't sound appealing.)

				// +1 because we need to distinguish the oldest known pixels from
				// pixels which 
				let canvas_age = u64::try_from(info.heatmap_cooldown).unwrap() + 1;
				Arc::new(RwLock::new(now - Duration::from_secs(canvas_age)))
			});
			let canvas_start = canvas_start.read().await;

			let heatmap = self.fetch_buffer(BufferType::Heatmap);
			let virginmap = self.fetch_buffer(BufferType::Virginmap);

			let (heatmap, virginmap) = futures_util::try_join!(heatmap, virginmap)?;

			let timestamps_data = std::iter::zip(heatmap, virginmap)
				.map(|(heat, virgin)| {
					if virgin == 0 {
						// pixel is non-virgin
						let pixel_time = now - Duration::from_secs(u64::try_from(heat).unwrap());
						u32::try_from(pixel_time.duration_since(*canvas_start).unwrap().as_secs())
							.expect("Canvas is too old") // 136 years is a pretty long time
					} else {
						// pixel is virgin
						0
					}
				})
				.collect();
			
			*timestamps = Some(Arc::new(RwLock::new(timestamps_data)));
		}

		Ok(timestamps.as_ref().unwrap().clone())
	}

	async fn update_buffers(&self, pixel: &Pixel) {
		let info = self.info().await
			.expect("Obtaining /info failed while updating buffers");
		let info = info.read().await;
		let colors = self.cache.colors.lock().await;
		// NOTE: lock must happen in this order, otherwise we risk deadlock with
		// timestamps().
		let timestamps = self.cache.timestamps.lock().await;
		let created_at = self.cache.created_at.lock().await;

		let index = pixel.y * info.width + pixel.x;

		if let Some(buffer) = colors.as_ref() {
			let mut buffer = buffer.write().await;
			buffer[index] = pixel.color;
		}
		drop(colors);

		if let Some(buffer) = timestamps.as_ref() {
			let mut buffer = buffer.write().await;
			let now = SystemTime::now();
			let canvas_epoch = created_at
				.as_ref()
				.expect("Timestamps exist but canvas has no start time")
				.read()
				.await;
			let timestamp = now.duration_since(*canvas_epoch).unwrap().as_secs();
			buffer[index] = u32::try_from(timestamp).expect("Canvas is too old");
		}
		drop(timestamps);
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

		self.info().await.map_err(ConnectError::InfoFailed)?;
		self.event_handler.handle_ready().await;

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
					for pixel in &pixels {
						self.update_buffers(pixel).await;
					}
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