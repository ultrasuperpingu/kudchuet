#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::thread::JoinHandle;
use std::time::Duration;


use minimax::strategies::iterative::SearchStopSignal;

use crate::ai::uci::{UciInfoAttribute, UciMessage, UciOptionConfig, UciTimeControl};
use crate::gui::{BoardGame, BoardMove};
use crate::ConcreteStrategy;

pub struct UCILikeCLIEngine<G, AI>
	where G: BoardGame + Send + 'static,
	G::M: BoardMove<G> + Copy + Send + 'static,
	AI: ConcreteStrategy<G> + Send + 'static,
{
	ai: Option<AI>,
	current_search: Option<SearchHandle<AI>>,
	phantom: std::marker::PhantomData<G>,
}
struct SearchHandle<AI> {
	stop_flag: SearchStopSignal,
	thread: JoinHandle<AI>,
}
impl<G, AI> UCILikeCLIEngine<G, AI>
where
	G: BoardGame + Send + 'static,
	G::M: BoardMove<G> + Copy + Send + 'static,
	AI: ConcreteStrategy<G> + Send + 'static,
{
	pub fn new(ai: AI) -> Self {
		Self {
			ai: Some(ai),
			current_search: None,
			phantom: std::marker::PhantomData,
		}
	}
	pub fn process(&mut self) -> Result<(), std::io::Error> {
		let mut debug:bool = true; 
		let mut pos = G::default();

		
		let stdin = std::io::stdin();
		let mut quit = false;
		let mut opts = self.ai.as_ref().unwrap().get_options(); // AI is present at start (TODO: if process is launched twice, we should panic) 
		while !quit {
			let mut buffer = String::new();
			stdin.read_line(&mut buffer)?;
			let messages_str = buffer.lines();
			for m_str in messages_str {
				if m_str.is_empty() {
					continue;
				}
				let m = UciMessage::parse(m_str);
				if debug {
					eprintln!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("received: {}", m))]));
				}
				match m {
					UciMessage::Uci => {
						println!("{}", UciMessage::Id { name: Some("KudchuetChess".into()), author: None });
						println!("{}", UciMessage::Id { name: None, author: Some("Ping".into()) });
						println!("{}", UciMessage::Option(UciOptionConfig::Spin { name: "Hash".into(), default: Some(128), min: Some(1), max: Some(4096) }));
						println!("{}", UciMessage::Option(UciOptionConfig::Check { name: "Mtdf".into(), default: Some(false) }));
						println!("{}", UciMessage::Option(UciOptionConfig::Spin { name: "Threads".into(), default: None, min: Some(1), max: Some(2048) }));
						println!("{}", UciMessage::Option(UciOptionConfig::Button { name: "Clear Hash".into() }));
						
						println!("{}", UciMessage::UciOk);
					}
					UciMessage::UciNewGame => {
						pos=G::default();
						self.stop_search().reset_with_options(opts.clone());
					}
					UciMessage::SetOption { name, value } => {
						if debug {
							println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("setting option {}={:?}", name, value))]));
						}
						let mut ok = true;
						match name.as_str() {
							"Hash" => {
								if let Some(value) = &value {
									if let Ok(value) = value.parse() {
										if debug {
											println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("setting hash size to {}MB", value))]));
										}
										opts.table_megabyte_size = value;
									}
								}
							}
							"Mtdf" => {
								if let Some(value) = &value {
									if let Ok(value) = value.parse::<bool>() {
										if let Some(v) = opts.uci.get_mut("Mtdf") {
											v.set_bool(value);
										}
										if debug {
											println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("Setting mtdf to {}", value))]));
										}
									}
								}
							}
							"Threads" => {
								if let Some(value) = &value {
									let nb_threads = value.parse().ok();
									if debug {
										println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(
											format!("setting threads to {:?}", nb_threads)
										)]));
									}
									opts.threads = nb_threads;
								}
							}
							"Clear Hash" => {
								// just reset
							}
							_ => {
								ok = false;
							}
							
						}
						if ok {
							self.stop_search().reset_with_options(opts.clone());
						}
					}
					UciMessage::Position { startpos, fen, moves } => {
						if startpos {
							pos = G::default();
						}
						if let Some(fen_str) = fen {
							if let Ok(p) = pos.get_position_from_string(&fen_str.0) {
								pos = p;
							}
						}
						for m in moves {
							match G::M::from_uci(&m) {
								Ok(mv) => {
									if let Some(p) = G::apply(&mut pos, mv) {
										pos = p;
									} else {
										eprintln!("Invalid move: {}", mv.to_uci().expect("BoardMove::to_uci() must be implemented for UCI engines (used to output moves like e2e4)"));
									}
								},
								Err(e) => eprintln!("{}", e),
							}
						}
					}
					UciMessage::Go { time_control, search_control } => {
						let mut asked_time = None;
						let mut asked_depth = None;
						if let Some(search) = self.current_search.take() {
							search.stop_flag.stop_search();
    						self.ai = Some(search.thread.join().expect("Thread panicked"));
						}
						let mut ai = self.ai.take().expect("AI must exist");
						
						if let Some(tc) = time_control {
							match tc {
								UciTimeControl::Ponder => {
									// Put the engine into ponder mode ("think" on opponent's time)
								}
								UciTimeControl::MoveTime(t) => {
									asked_time = Some(t);
								}
								UciTimeControl::Infinite => {
									asked_time = Some(Duration::from_hours(99));
								}
								_ => {
									eprintln!("Unsupported TimeControl")
								}
							}
						}
						if let Some(sc) = search_control {
							if let Some(d) = sc.depth {
								asked_depth = Some(d);
							}
						}
						if let Some(time) = asked_time {
							if let Some(depth) = asked_depth {
								ai.set_depth_or_timeout(depth, time);
							} else {
								ai.set_timeout(time);
							}
						} else {
							if let Some(depth) = asked_depth {
								ai.set_max_depth(depth);
							} else {
								ai.set_max_depth(5);
							}
						}
						let pos_clone = pos.clone();
						let debug_clone = debug;
						let stop_flag = ai.stop_signal();

						let handle = std::thread::spawn(move || {
							let best_move = ai.choose_move(&pos_clone);

							if debug_clone {
								let pv_move = ai.principal_variation();
								let score = ai.root_value();
								let mut moves = vec![];
								let mut temp_pos = pos_clone.clone();

								let mut fail = false;
								for pv_m in pv_move {
									if let Some(pv_m_str) = temp_pos.move_to_string(&pv_m) {
										moves.push(pv_m_str);
										temp_pos.play(pv_m);
									} else {
										fail = true;
										break;
									}
								}

								if !fail {
									println!("{}", UciMessage::Info(vec![
										UciInfoAttribute::Pv(moves),
										UciInfoAttribute::Score {
											cp: Some(score as i32),
											mate: None,
											lower_bound: None,
											upper_bound: None,
										}
									]));
								}
							}

							if let Some(m) = best_move {
								let move_str = m.to_uci();
								println!(
									"bestmove {}",
									move_str.expect("BoardMove::to_uci() must be implemented for UCI engines (used to output moves like e2e4)")
								);
							} else {
								println!("bestmove (none)");
							}

							std::io::stdout().flush().unwrap(); // safety: panic on I/O error or EOF is ok

							ai
						});
						self.current_search = Some(SearchHandle {
							stop_flag,
							thread: handle,
						});
					},
					UciMessage::Stop => {
						self.stop_search();
					}
					UciMessage::Quit => {
						quit = true;
						break;
					},
					UciMessage::Debug(d) => { debug=d; },
					UciMessage::IsReady => {
						println!("{}", UciMessage::ReadyOk);
					},
					_ => {
						println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("Unknown Command: {}", m))]));
					}
				}
			}
			if quit {
				break;
			}
		}
		Ok(())
	}

	fn stop_search(&mut self) -> &mut AI {
		if let Some(search) = self.current_search.take() {
			search.stop_flag.stop_search();
			self.ai = Some(search.thread.join().expect("Thread panicked"));
		}
		self.ai.as_mut().expect("AI not present!!!")
	}

}
