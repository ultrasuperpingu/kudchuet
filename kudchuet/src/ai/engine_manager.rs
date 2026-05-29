use crate::Player;
#[cfg(not(target_arch = "wasm32"))]
use crate::ai::external_engine::{ExternalEngine, ExternalEngineEntry};
use crate::ai::uci::UciValue;
use crate::ai::{AIEngine, AIEngineProvider};
use crate::gui::GUIGame;
use crate::utils::short_type_name;
use std::collections::HashMap;
use std::pin::Pin;
pub enum ThinkingResult<M> {
	NotThinking,
	Thinking,
	FinishedThinking(Option<M>),
}
type FutureHandler<M> = Pin<Box<dyn Future<Output = Option<M>> + Send>>;
pub struct EngineManager<G: GUIGame + Sync>
//where
//	G::M: BoardMove<G>,
{
	engines: HashMap<String, Box<dyn AIEngine<G>>>,
	engine_options: Option<HashMap<String, HashMap<String, UciValue>>>,
	#[cfg(not(target_arch = "wasm32"))]
	external_providers: Vec<ExternalEngineEntry>,
	internal_providers: Vec<Box<dyn AIEngineProvider<G>>>,
	player_engines: HashMap<Player, Option<String>>,
	paused: bool,
	ai_future: Option<FutureHandler<G::M>>,
}
impl<G: GUIGame + Clone + Sync + Send + 'static> Default for EngineManager<G>
where
//	G::M: BoardMove<G> + Send,
	G::M: Send,
{
	fn default() -> Self {
		Self::new()
	}
}
impl<G: GUIGame + Clone + Sync + Send + 'static> EngineManager<G>
where
//	G::M: BoardMove<G> + Send,
	G::M: Send,
{
	pub fn new() -> Self {
		Self {
			engines: HashMap::new(),
			engine_options: None,
			#[cfg(not(target_arch = "wasm32"))]
			external_providers: Vec::new(),
			internal_providers: Vec::new(),
			player_engines: HashMap::new(),
			paused: false,
			ai_future: None,
		}
	}
	pub fn new_with_internals(internal_providers: Vec<Box<dyn AIEngineProvider<G>>>) -> Self {
		Self {
			engines: HashMap::new(),
			engine_options: None,
			#[cfg(not(target_arch = "wasm32"))]
			external_providers: Vec::new(),
			internal_providers,
			player_engines: HashMap::new(),
			paused: false,
			ai_future: None,
		}
	}

	pub fn add_internal_provider(&mut self, provider: Box<dyn AIEngineProvider<G>>) {
		self.internal_providers.push(provider);
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn add_external_provider(&mut self, provider: ExternalEngineEntry) {
		self.external_providers.push(provider);
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn get_external_providers_mut(&mut self) -> &mut Vec<ExternalEngineEntry> {
		&mut self.external_providers
	}

	pub fn list_internal_provider_names(&self) -> Vec<String> {
		self.internal_providers
			.iter()
			.map(|p| p.get_name().to_owned())
			.collect()
	}

	pub fn has_internal_provider(&self, name: &str) -> bool {
		self.internal_providers.iter().any(|p| p.get_name() == name)
	}

	pub fn get_internal_engine_count(&self) -> usize {
		self.internal_providers.len()
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn get_external_engine_count(&self) -> usize {
		self.external_providers.len()
	}

	pub fn ensure_engine(&mut self, name: &str) -> Result<&mut Box<dyn AIEngine<G>>, String> {
		match self.engines.entry(name.to_string()) {
			std::collections::hash_map::Entry::Occupied(o) => Ok(o.into_mut()),
			std::collections::hash_map::Entry::Vacant(v) => {
				let engine = if let Some(provider) = self
					.internal_providers
					.iter()
					.find(|p| p.get_name() == name)
				{
					let mut engine = provider.build_engine();
					if let Some(opts) = self.engine_options.as_mut() {
						if let Some(opt) = opts.get_mut(name) {
							if let Some(opt_engine) = engine.get_options() {
								opt.extend(opt_engine.clone());
							}
							engine.set_options(opt.clone());
						}
					} else {
						if let Some(opts) = engine.get_options() {
							self.engine_options
								.get_or_insert_default()
								.insert(name.into(), opts.clone());
						}
					}
					engine
				} else {
					#[cfg(not(target_arch = "wasm32"))]
					if let Some(ext) = self.external_providers.iter().find(|e| e.name == name) {
						let mut engine =
							Box::new(ExternalEngine::new(&ext.path, &ext.args.clone())?);
						if let Some(opts) =
							self.engine_options.get_or_insert_default().get_mut(name)
						{
							if let Some(engine_opts) = AIEngine::<G>::get_options(&*engine) {
								opts.extend(engine_opts.clone());
								AIEngine::<G>::set_options(&mut *engine, opts.clone());
							}
						} else {
							if let Some(opts) = AIEngine::<G>::get_options(&*engine) {
								self.engine_options
									.get_or_insert_default()
									.insert(name.into(), opts.clone());
							}
						}
						engine
					} else {
						return Err(format!("Engine '{}' not found", name));
					}
					#[cfg(target_arch = "wasm32")]
					return Err(format!("Engine '{}' not found", name));
				};
				Ok(v.insert(engine))
			}
		}
		/*if !self.engines.contains_key(name) {
			if let Some(provider) = self
				.internal_providers
				.iter()
				.find(|p| p.get_name() == name)
			{
				let mut engine = provider.build_engine();
				if let Some(opts) = self.engine_options.as_mut() {
					if let Some(opt) = opts.get_mut(name) {
						if let Some(opt_engine) = engine.get_options() {
							opt.extend(opt_engine.clone());
						}
						engine.set_options(opt.clone());
					}
				} else {
					if let Some(opts) = engine.get_options() {
						self.engine_options
							.get_or_insert_default()
							.insert(name.into(), opts.clone());
					}
				}
				self.engines.insert(name.to_string(), engine);
			} else {
				#[cfg(not(target_arch = "wasm32"))]
				if let Some(ext) = self.external_providers.iter().find(|e| e.name == name) {
					let mut engine = Box::new(ExternalEngine::new(&ext.path, &ext.args.clone())?);
					if let Some(opts) = self.engine_options.get_or_insert_default().get_mut(name) {
						if let Some(engine_opts) = AIEngine::<G>::get_options(&*engine) {
							opts.extend(engine_opts.clone());
							AIEngine::<G>::set_options(&mut *engine, opts.clone());
						}
					} else {
						if let Some(opts) = AIEngine::<G>::get_options(&*engine) {
							self.engine_options
								.get_or_insert_default()
								.insert(name.into(), opts.clone());
						}
					}
					self.engines.insert(name.to_string(), engine);
				} else {
					return Err(format!("Engine '{}' not found", name));
				}
				#[cfg(target_arch = "wasm32")]
				return Err(format!("Engine '{}' not found", name));
			}
		}
		self.engines
			.get_mut(name)
			.ok_or_else(|| format!("Engine '{}' missing", name))*/
	}
	pub fn get_engine(&self, engine: &str) -> Option<&dyn AIEngine<G>> {
		self.engines.get(engine).map(Box::as_ref)
	}
	pub fn get_player_engine(&self, p: Player) -> Option<&String> {
		self.player_engines.get(&p)?.as_ref()
	}
	pub fn set_player_engine(&mut self, p: Player, engine: Option<String>) {
		self.player_engines.insert(p, engine);
	}
	pub fn get_engine_mut(&mut self, engine: &str) -> Option<&mut Box<dyn AIEngine<G>>> {
		self.engines.get_mut(engine)
	}
	pub fn choose_move(&mut self, game: &G) -> Result<G::M, String> {
		let eng = self.get_player_engine(game.current_player()).cloned();
		self.choose_move_with(eng.unwrap_or_else(|| "default".into()), game)
	}

	fn choose_move_with(&mut self, ai: String, game: &G) -> Result<G::M, String> {
		if self.paused {
			return Err("Engine is paused".into());
		}

		let engine = self.ensure_engine(&ai)?;
		engine
			.choose_move(game)
			.ok_or_else(|| "No valid move found".into())
	}
	pub fn choose_move_async(&mut self, game: &G) -> Result<(), String> {
		let game = game.clone();
		let eng = self.get_player_engine(game.current_player()).cloned();
		self.choose_move_async_with(eng.unwrap_or_else(|| "default".into()), game)
	}

	pub fn is_thinking(&self) -> bool {
		self.ai_future.is_some()
	}
	pub fn poll_ai_result(&mut self) -> ThinkingResult<G::M> {
		if let Some(fut) = self.ai_future.as_mut() {
			use futures::task::noop_waker;
			use futures::task::{Context, Poll};
			use std::pin::Pin;

			let waker = noop_waker();
			let mut cx = Context::from_waker(&waker);

			match Pin::new(fut).poll(&mut cx) {
				Poll::Ready(mv) => {
					println!("ai_future finished");
					self.ai_future = None;
					ThinkingResult::FinishedThinking(mv)
				}
				Poll::Pending => ThinkingResult::Thinking,
			}
		} else {
			ThinkingResult::NotThinking
		}
	}
	pub fn stop_thinking(&mut self) {
		//TODO: find currently thinking engine
		for e in &self.engines {
			e.1.stop_thinking();
		}
	}
	pub fn cancel_thinking(&mut self) {
		self.stop_thinking();
		// TODO: avoid doing this (thread leak??)
		self.ai_future = None
	}
	//#[cfg(not(target_arch = "wasm32"))]
	fn choose_move_async_with(&mut self, ai: String, game: G) -> Result<(), String> {
		if self.paused {
			return Err("Engine is paused".into());
		}

		let engine = self.ensure_engine(&ai)?;
		self.ai_future = Some(engine.choose_move_async(game));
		Ok(())
	}
	pub fn set_paused(&mut self, paused: bool) {
		self.paused = paused;
	}

	pub fn is_paused(&self) -> bool {
		self.paused
	}
	pub fn load_engines_parameters(&mut self, ctx: &egui::Context) {
		#[cfg(not(target_arch = "wasm32"))]
		if let Some(json) = ctx.data_mut(|d| {
			d.get_persisted::<String>(
				(short_type_name::<G>().to_owned() + "_external_engines").into(),
			)
		}) {
			eprintln!("loading externals: {}", json);
			if let Ok(settings) = serde_json::from_str(&json) {
				self.external_providers = settings;
			}
		}
		if let Some(json) = ctx.data_mut(|d| {
			d.get_persisted::<String>(
				(short_type_name::<G>().to_owned() + "_engines_options").into(),
			)
		}) {
			eprintln!("loading settings: {}", json);
			if let Ok(settings) = serde_json::from_str(&json) {
				self.engine_options = settings;
			}
		}
	}
	/*pub fn get_engine_options(&self, engine: &String) -> Option<&HashMap<String, UciValue>> {
		self.engine_options.get(engine)
	}
	pub fn get_engine_options_mut(&mut self, engine: &String) -> Option<&mut HashMap<String, UciValue>> {
		self.engine_options.get_mut(engine)
	}*/
	#[cfg(not(target_arch = "wasm32"))]
	pub fn save_external_engines(&self, ctx: &egui::Context) {
		let json = serde_json::to_string(&self.external_providers).unwrap();
		eprintln!("saving externals: {}", json);
		ctx.data_mut(|d| {
			d.insert_persisted(
				(short_type_name::<G>().to_owned() + "_external_engines").into(),
				json,
			)
		});
	}
	pub fn save_all_engine_options(&self, ctx: &egui::Context) {
		let json = serde_json::to_string(&self.engine_options).unwrap();
		eprintln!("saving settings: {}", json);
		ctx.data_mut(|d| {
			d.insert_persisted(
				(short_type_name::<G>().to_owned() + "_engines_options").into(),
				json,
			)
		});
	}
	pub fn get_all_engine_names(&self) -> Vec<String> {
		let mut names = Vec::new();

		for internal in self.internal_providers.iter() {
			names.push(internal.get_name().to_owned());
		}
		#[cfg(not(target_arch = "wasm32"))]
		for entry in self.external_providers.iter() {
			names.push(entry.name.clone());
		}

		names
	}
}
