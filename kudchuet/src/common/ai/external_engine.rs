
#![cfg(not(target_arch = "wasm32"))]

use egui_field_editor::EguiInspect;

use crate::common::ai::uci::{Serializable, UciFen, UciInfoAttribute, UciMessage, UciOptionConfig, UciSearchControl, UciTimeControl};
use crate::common::ai::{AIEngine, AIEngineProvider, AIOptions};
use crate::common::gui::{BoardGame, BoardMove};
use std::collections::VecDeque;
use std::path::PathBuf;
use futures::channel::oneshot;

use std::pin::Pin;
use std::sync::RwLock;
use std::{
	io::{BufRead, BufReader, Write},
	process::{Child, Command, Stdio},
	sync::mpsc::{self, Receiver, Sender},
	thread,
	time::Duration,
};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::JoinHandle;


#[derive(Debug, Clone, Default)]
pub struct EngineInfo {
	pub name: Option<String>,
	pub author: Option<String>,
	pub options: Vec<UciOptionConfig>,
}

pub struct ExternalEngine {
	_child: Child,

	options: AIOptions,

	cmd_tx: Sender<UciMessage>,
	result_rx: Arc<Mutex<Receiver<String>>>,

	stop_flag: Arc<AtomicBool>,

	writer_handle: Option<JoinHandle<()>>,
	reader_handle: Option<JoinHandle<()>>,

	messages: Arc<RwLock<VecDeque<UciMessage>>>,
	info: Arc<RwLock<EngineInfo>>,
}
impl ExternalEngine
{
	pub fn new(path: &PathBuf, arguments: &str) -> Result<Self, String> {
		let mut child = Command::new(path)
			.args(arguments.split_whitespace())
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.map_err(|e| format!("Failed to launch engine '{:?}': {}", path.to_str(), e))?;

		let stdin = child.stdin.take().ok_or("stdin")?;
		let stdout = child.stdout.take().ok_or("stdout")?;

		let (cmd_tx, cmd_rx) = mpsc::channel::<UciMessage>();
		let (result_tx, result_rx) = mpsc::channel::<String>();

		let stop_flag = Arc::new(AtomicBool::new(false));

		let messages = Arc::new(RwLock::new(VecDeque::new()));
		let messages_reader = messages.clone();

		let info = Arc::new(RwLock::new(EngineInfo::default()));

		// WRITER THREAD
		let stop_w = stop_flag.clone();
		let writer_handle = thread::spawn(move || {
			let mut stdin = stdin;

			while !stop_w.load(Ordering::SeqCst) {
				match cmd_rx.recv_timeout(Duration::from_millis(50)) {
					Ok(msg) => {
						let line = msg.serialize();
						if writeln!(stdin, "{}", line).is_err() {
							break;
						}
						let _ = stdin.flush();
					}
					Err(mpsc::RecvTimeoutError::Timeout) => continue,
					Err(mpsc::RecvTimeoutError::Disconnected) => {
						eprintln!("Write thread disconnected");
						break
					},
				}
			}
		});


		// READER THREAD
		let stop_r = stop_flag.clone();
		let info_ref = info.clone();
		let reader_handle = thread::spawn(move || {
			let mut reader = BufReader::new(stdout);
			let mut line = String::new();

			while !stop_r.load(Ordering::SeqCst) {
				line.clear();

				match reader.read_line(&mut line) {
					Ok(0) => {
						//EOF
						eprintln!("received EOF");
						break
					},
					Ok(_) => {
						eprintln!("received line: {}", line.trim());
						let msg = UciMessage::parse(line.trim());
						if let Ok(mut msgs) = messages_reader.write() {
							msgs.push_back(msg.clone());

							if msgs.len() > 10_000 {
								msgs.pop_front();
							}
						}
						match &msg {
							UciMessage::BestMove { best_move, .. } => {
								eprintln!("received bestmove: {}", best_move);
								if let Err(e) = result_tx.send(best_move.clone()) {
									eprintln!("Failed to send bestmove to main thread: {:?}", e);
								}
							}
							UciMessage::ReadyOk => {
								eprintln!("received ready");
							}
							UciMessage::UciOk => {
							}
							UciMessage::Id { name, author } => {
								eprintln!("received id");
								if let Ok(mut info) = info_ref.write() {
									if let Some(n) = name {
										info.name = Some(n.clone());
									}
									if let Some(a) = author {
										info.author = Some(a.clone());
									}
								}
							}

							UciMessage::Option(opt) => {
								eprintln!("received option");
								if let Ok(mut info) = info_ref.write() {
									eprintln!("writing option");
									if let Some(existing) = info.options.iter_mut().find(|o| o.get_name() == opt.get_name()) {
										*existing = opt.clone();
									} else {
										info.options.push(opt.clone());
									}
								}
							}
							_ => {}
						}
					}
					Err(_) => break,
				}
			}
		});

		let mut engine = Self {
			_child: child,
			options: AIOptions::default(),
			cmd_tx,
			result_rx: Arc::new(Mutex::new(result_rx)),
			stop_flag,
			writer_handle: Some(writer_handle),
			reader_handle: Some(reader_handle),

			messages,
			info
		};

		// Handshake UCI
		engine.send_msg(UciMessage::Uci)?;

		engine.wait_for(
			|m| matches!(m, UciMessage::UciOk),
			Duration::from_secs(2),
		)?;
		engine.init_uci_options();
		engine.send_msg(UciMessage::IsReady)?;

		engine.wait_for(
			|m| matches!(m, UciMessage::ReadyOk),
			Duration::from_secs(2),
		)?;
		
		Ok(engine)
	}
	fn init_uci_options(&mut self) {
		let info = self.get_engine_info();

		for opt in info.options {
			let name = opt.get_name();

			let value = opt.clone();

			self.options.uci.insert(name.into(), value.into());
			
		}
	}
	fn wait_for<F>(&self, predicate: F, timeout: Duration) -> Result<(), String>
		where
			F: Fn(&UciMessage) -> bool,
		{
			let start = std::time::Instant::now();

			while start.elapsed() < timeout {
				if let Ok(msgs) = self.messages.read() {
					if msgs.iter().any(&predicate) {
						return Ok(());
					}
				}
				std::thread::sleep(Duration::from_millis(10));
			}

			Err("Timeout waiting for UCI response".into())
		}
	pub fn get_engine_info(&self) -> EngineInfo {
		self.info.read().map(|i| i.clone()).unwrap_or_default()
	}
	pub fn get_options_mut(&mut self) -> &mut AIOptions {
		&mut self.options
	}

	pub fn send_msg(&self, msg: UciMessage) -> Result<(), String> {
		println!("send msg: {}", msg);
		self.cmd_tx
			.send(msg)
			.map_err(|e| format!("Send Error: {}", e))
	}
	pub fn get_messages(&self) -> Vec<UciMessage> {
		self.messages
			.read()
			.map(|v| v.iter().cloned().collect())
			.unwrap_or_default()
	}
	pub fn last_bestmove(&self) -> Option<String> {
		self.messages
			.read()
			.ok()?
			.iter()
			.rev()
			.find_map(|m| {
				if let UciMessage::BestMove { best_move, .. } = m {
					Some(best_move.clone())
				} else {
					None
				}
			})
	}
	pub fn last_info(&self) -> Option<Vec<UciInfoAttribute>> {
		self.messages
			.read()
			.ok()?
			.iter()
			.rev()
			.find_map(|m| {
				if let UciMessage::Info(info) = m {
					Some(info.clone())
				} else {
					None
				}
			})
	}
	pub fn clear_messages(&self) {
		if let Ok(mut msgs) = self.messages.write() {
			msgs.clear();
		}
	}
	pub fn stop(&mut self) {
		self.stop_flag.store(true, Ordering::SeqCst);

		let _ = self.send_msg(UciMessage::Quit);

		if let Some(h) = self.writer_handle.take() {
			let _ = h.join();
		}

		if let Some(h) = self.reader_handle.take() {
			let _ = h.join();
		}

		let _ = self._child.wait();
	}
	pub fn is_stopped(&self) -> bool {
		self.stop_flag.load(Ordering::SeqCst)
	}
}

impl<G: BoardGame+Sync+Send+'static> AIEngine<G> for ExternalEngine
where
	G::M: BoardMove<G>+Send
{
	fn get_options(&self) -> Option<AIOptions> {
		Some(self.options.clone())
	}

	fn reset_with_options(&mut self, opts: AIOptions) {
		self.options = opts;
		for (name, value) in &self.options.uci {
			let _ = self.send_msg(UciMessage::SetOption { name: name.clone(), value: value.to_option_string() });
		}
	}

	fn set_position(&self, game: &G) {
		if let Some(pos) = game.position_to_string() {
			let e=self.send_msg(UciMessage::Position { startpos: false, fen: Some(UciFen(pos)), moves: vec![] } );
			if e.is_err() {
				eprintln!("error in set_position: {}", e.err().unwrap());
			}
		}
	}

	fn choose_move(&self, game: &G) -> Option<G::M> {
		self.set_position(game);
		let go = if self.options.max_time > 0.0 {
			if self.options.max_depth == 0 {
				UciMessage::Go {
					time_control: Some(UciTimeControl::MoveTime(
						Duration::from_secs_f32(self.options.max_time)
					)),
					search_control: None,
				}
			} else {
				UciMessage::Go {
					time_control: Some(UciTimeControl::MoveTime(
						Duration::from_secs_f32(self.options.max_time)
					)),
					search_control: Some(UciSearchControl::depth(self.options.max_depth)),
				}
			}
		} else {
			if self.options.max_depth == 0 {
				UciMessage::Go {
					time_control: None,
					search_control: Some(UciSearchControl::depth(self.options.max_depth)),
				}
			} else {
				UciMessage::Go {
					time_control: None,
					search_control: None,
				}
			}
		};
		

		self.send_msg(go).ok()?;

		//let res = match self.result_rx.recv_timeout(Duration::from_secs_f32(3600.0)) {
		let res = match self.result_rx.lock().unwrap().recv() {
			Ok(mv_str) => {
				eprintln!("choose_move: received in thread {}", mv_str);
				game.move_from_string(&mv_str).ok()
			},
			Err(e) => {
				eprintln!("Engine recv error {}", e);
				None
			}
		};
		if let Some(res) = res {
			return Some(res);
		}
		None
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> 
	//where
	//	G: BoardGame + Send + 'static,
	//	G::M: BoardMove<G> + Send + 'static,
	{
		self.set_position(&game);
		let go = if self.options.max_time > 0.0 {
			if self.options.max_depth == 0 {
				UciMessage::Go {
					time_control: Some(UciTimeControl::MoveTime(
						Duration::from_secs_f32(self.options.max_time)
					)),
					search_control: None,
				}
			} else {
				UciMessage::Go {
					time_control: Some(UciTimeControl::MoveTime(
						Duration::from_secs_f32(self.options.max_time)
					)),
					search_control: Some(UciSearchControl::depth(self.options.max_depth)),
				}
			}
		} else {
			if self.options.max_depth == 0 {
				UciMessage::Go {
					time_control: None,
					search_control: Some(UciSearchControl::depth(self.options.max_depth)),
				}
			} else {
				UciMessage::Go {
					time_control: None,
					search_control: None,
				}
			}
		};
		

		let res = self.send_msg(go);
		if let Err(e) = res {
			eprintln!("Error sending Go message: {}", e);
		}
		// Clonage des arcs pour pouvoir les déplacer dans le thread
		let result_rx = self.result_rx.clone();
		let game = Arc::new(game);

		let (tx, rx) = oneshot::channel();

		std::thread::spawn({
			let result_rx = result_rx.clone();
			let game = game.clone();

			move || {
				let res = match result_rx.lock().unwrap().recv() {
					Ok(mv_str) => {
						eprintln!("choose_move_async: received in thread {}", mv_str);
						game.move_from_string(&mv_str).ok()
					}
					Err(e) => {
						eprintln!("Engine recv error {}", e);
						None
					}
				};
				let _ = tx.send(res);
			}
		});

		Box::pin(async move { rx.await.ok().flatten() })
	}
	fn stop_thinking(&self) {
		if let Err(err) = self.send_msg(UciMessage::Stop) {
			eprintln!("Error sending Stop Message: {}", err);
		}
	}
	#[cfg(target_arch = "wasm32")]
	fn choose_move_async(
		&self,
		game: G,
	) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
		let ai = self.ai.clone();

		Box::pin(async move {
			let mut ai = ai.lock().unwrap();
			ai.choose_move(&game)
		})
	}
	fn set_depth_or_timeout(&mut self, depth:u8, timeout: Duration) {
		self.options.max_depth = depth;
		self.options.max_time = timeout.as_secs_f32();
	}
}
impl Drop for ExternalEngine {
	fn drop(&mut self) {
		self.stop();
	}
}
#[derive(EguiInspect)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExternalEngineEntry {
	pub name: String,
	#[inspect(file())]
	pub path: PathBuf,
	pub args: String,
}

impl<G: BoardGame+Sync+Send+'static> AIEngineProvider<G> for ExternalEngineEntry
where
	G::M: BoardMove<G>+Send,
{
	type Engine = ExternalEngine;

	fn get_name(&self) -> &String {
		&self.name
	}

	fn build_engine(&self) -> Self::Engine {
		match ExternalEngine::new(&self.path, &self.args) {
			Ok(engine) => engine,
			Err(e) => {
				eprintln!("Failed to create engine '{}': {}", self.name, e);
				panic!("Failed to create engine '{}': {}", self.name, e);
			}
		}
	}
}