//#![allow(clippy::all)]

//! The `uci` module contains the definitions that represent UCI protocol messages.
//!
//! This code comes from mainly from vampirc_uci (https://github.com/vampirc/vampirc-uci)


use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};

use std::time::Duration;

/// Specifies whether a message is engine- or GUI-bound.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CommunicationDirection {
	/// An engine-bound message.
	GuiToEngine,

	/// A GUI-bound message.
	EngineToGui,
}

pub trait Serializable: Display {
	fn serialize(&self) -> String;
}

/// An enumeration type containing representations for all messages supported by the UCI protocol.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciMessage {
	/// The `uci` engine-bound message.
	Uci,

	/// The `debug` engine-bound message. Its internal property specifies whether debug mode should be enabled (`true`),
	/// or disabled (`false`).
	Debug(bool),

	/// The `isready` engine-bound message.
	IsReady,

	/// The `register` engine-bound message.
	Register {
		/// The `register later` engine-bound message.
		later: bool,

		/// The name part of the `register <code> <name>` engine-bound message.
		name: Option<String>,

		/// The code part of the `register <code> <name>` engine-bound message.
		code: Option<String>,
	},

	/// The `position` engine-bound message.
	Position {
		/// If `true`, it denotes the starting chess position. Generally, if this property is `true`, then the value of
		/// the `fen` property will be `None`.
		startpos: bool,

		/// The [FEN format](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) representation of a chess
		/// position.
		fen: Option<UciFen>,

		/// A list of moves to apply to the position.
		moves: Vec<String>,
	},

	/// The `setoption` engine-bound message.
	SetOption {
		/// The name of the option to set.
		name: String,

		/// The value of the option to set. If the option has no value, this should be `None`.
		value: Option<String>,
	},

	/// The `ucinewgame` engine-bound message.
	UciNewGame,

	/// The `stop` engine-bound message.
	Stop,

	/// The `ponderhit` engine-bound message.
	PonderHit,

	/// The `quit` engine-bound message.
	Quit,

	/// The `go` engine-bound message.
	Go {
		/// Time-control-related `go` parameters (sub-commands).
		time_control: Option<UciTimeControl>,

		/// Search-related `go` parameters (sub-commands).
		search_control: Option<UciSearchControl>,
	},

	// From this point on we have client-bound messages

	/// The `id` GUI-bound message.
	Id {
		/// The name of the engine, possibly including the version.
		name: Option<String>,

		/// The name of the author of the engine.
		author: Option<String>,
	},

	/// The `uciok` GUI-bound message.
	UciOk,

	/// The `readyok` GUI-bound message.
	ReadyOk,

	/// The `bestmove` GUI-bound message.
	BestMove {
		/// The move the engine thinks is the best one in the position.
		best_move: String,

		/// The move the engine would like to ponder on.
		ponder: Option<String>,
	},

	/// The `copyprotection` GUI-bound message.
	CopyProtection(ProtectionState),

	/// The `registration` GUI-bound message.
	Registration(ProtectionState),

	/// The `option` GUI-bound message.
	Option(UciOptionConfig),

	/// The `info` GUI-bound message.
	Info(Vec<UciInfoAttribute>),

	/// Indicating unknown message.
	Unknown(String)
}
impl UciMessage {
	pub fn parse(line: &str) -> UciMessage {
		let tokens: Vec<&str> = line.split_whitespace().collect();
		if tokens.is_empty() {
			return UciMessage::Unknown(line.to_string());
		}

		match tokens[0] {
			"uci" => UciMessage::Uci,
			"isready" => UciMessage::IsReady,
			"ucinewgame" => UciMessage::UciNewGame,
			"quit" => UciMessage::Quit,
			"stop" => UciMessage::Stop,
			"ponderhit" => UciMessage::PonderHit,
			"debug" => {
				let on = tokens.get(1).is_some_and(|v| *v == "on");
				UciMessage::Debug(on)
			}
			"register" => {
				if tokens.get(1).is_some_and(|v| *v == "later") {
					UciMessage::register_later()
				} else {
					let mut name = None;
					let mut code = None;
					let mut i = 1;
					while i < tokens.len() {
						match tokens[i] {
							"name" => { i += 1; name = tokens.get(i).map(|s| s.to_string()); },
							"code" => { i += 1; code = tokens.get(i).map(|s| s.to_string()); },
							_ => {}
						}
						i += 1;
					}
					UciMessage::Register { later: false, name, code }
				}
			}
			"position" => {
				let mut startpos = false;
				let mut fen = None;
				let mut moves = Vec::new();
				let mut i = 1;

				while i < tokens.len() {
					match tokens[i] {
						"startpos" => startpos = true,
						"fen" => {
							i += 1;
							let mut fen_parts = Vec::new();
							while i < tokens.len() && tokens[i] != "moves" {
								fen_parts.push(tokens[i]);
								i += 1;
							}
							fen = Some(UciFen(fen_parts.join(" ")));
						}
						"moves" => {
							i += 1;
							while i < tokens.len() {
								moves.push(tokens[i].to_string());
								i += 1;
							}
							break;
						}
						_ => {}
					}
					i += 1;
				}

				UciMessage::Position { startpos, fen, moves }
			}
			"setoption" => {
				let mut name = String::new();
				let mut value = None;
				let mut i = 1;
				while i < tokens.len() {
					match tokens[i] {
						"name" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() && tokens[i] != "value" {
								parts.push(tokens[i]);
								i += 1;
							}
							name = parts.join(" ");
							i -= 1;
						}
						"value" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() {
								parts.push(tokens[i]);
								i += 1;
							}
							value = Some(parts.join(" "));
							i -= 1;
						}
						_ => {}
					}
					i += 1;
				}
				UciMessage::SetOption { name, value }
			}
			"go" => {
				use UciTimeControl::*;
				let mut time_control = None;
				let mut has_search_control = false;
				let mut search_control = crate::common::ai::uci::UciSearchControl {
					depth: None,
					nodes: None,
					mate: None,
					search_moves: vec![],
				};

				let mut i = 1;
				while i < tokens.len() {
					match tokens[i] {
						"infinite" => time_control = Some(Infinite),
						"ponder" => time_control = Some(Ponder),
						"movetime" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(ms) = t.parse::<u64>() {
									time_control = Some(MoveTime(std::time::Duration::from_millis(ms)));
								}
								i += 1;
							}
						}
						"wtime" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(ms) = t.parse::<u64>() {
									if let Some(TimeLeft { white_time, .. }) = time_control.as_mut() {
										*white_time = Some(std::time::Duration::from_millis(ms));
									} else {
										time_control = Some(TimeLeft {
											white_time: Some(std::time::Duration::from_millis(ms)),
											black_time: None,
											white_increment: None,
											black_increment: None,
											moves_to_go: None,
										});
									}
								}
								i += 1;
							}
						}
						"btime" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(ms) = t.parse::<u64>() {
									if let Some(TimeLeft { black_time, .. }) = time_control.as_mut() {
										*black_time = Some(std::time::Duration::from_millis(ms));
									} else {
										time_control = Some(TimeLeft {
											white_time: None,
											black_time: Some(std::time::Duration::from_millis(ms)),
											white_increment: None,
											black_increment: None,
											moves_to_go: None,
										});
									}
								}
								i += 1;
							}
						}
						"winc" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(ms) = t.parse::<u64>() {
									if let Some(TimeLeft{white_increment, ..}) = &mut time_control {
										*white_increment = Some(std::time::Duration::from_millis(ms));
									}
								}
								i += 1;
							}
						}
						"binc" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(ms) = t.parse::<u64>() {
									if let Some(TimeLeft{black_increment, ..}) = &mut time_control {
										*black_increment = Some(std::time::Duration::from_millis(ms));
									}
								}
								i += 1;
							}
						}
						"movestogo" => {
							if let Some(t) = tokens.get(i + 1) {
								if let Ok(n) = t.parse::<u8>() {
									if let Some(TimeLeft{moves_to_go, ..}) = &mut time_control {
										*moves_to_go = Some(n);
									}
								}
								i += 1;
							}
						}
						"depth" => {
							if let Some(d) = tokens.get(i + 1) {
								search_control.depth = d.parse().ok();
								has_search_control = true;
								i += 1;
							}
						}
						"nodes" => {
							if let Some(d) = tokens.get(i + 1) {
								search_control.nodes = d.parse().ok();
								has_search_control = true;
								i += 1;
							}
						}
						"mate" => {
							if let Some(d) = tokens.get(i + 1) {
								search_control.mate = d.parse().ok();
								has_search_control = true;
								i += 1;
							}
						}
						"searchmoves" => {
							i += 1;
							while i < tokens.len() {
								search_control.search_moves.push(tokens[i].to_string());
								has_search_control = true;
								i += 1;
							}
						}
						_ => {}
					}
					i += 1;
				}
				if has_search_control {
					UciMessage::Go { time_control, search_control: Some(search_control) }
				} else {
					UciMessage::Go { time_control, search_control: None }
				}
			}
			"bestmove" => {
				let best_move = tokens.get(1).unwrap_or(&"").to_string();
				let mut ponder = None;

				let mut i = 2;
				while i < tokens.len() {
					if tokens[i] == "ponder" {
						if let Some(p) = tokens.get(i + 1) {
							ponder = Some((*p).to_string());
						}
						i += 1;
					}
					i += 1;
				}

				UciMessage::BestMove { best_move, ponder }
			}
			"id" => {
				let mut name = None;
				let mut author = None;

				let mut i = 1;
				while i < tokens.len() {
					match tokens[i] {
						"name" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() && tokens[i] != "author" {
								parts.push(tokens[i]);
								i += 1;
							}
							name = Some(parts.join(" "));
							continue;
						}
						"author" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() {
								parts.push(tokens[i]);
								i += 1;
							}
							author = Some(parts.join(" "));
							break;
						}
						_ => {}
					}
					i += 1;
				}

				UciMessage::Id { name, author }
			}
			"uciok" => UciMessage::UciOk,
			"readyok" => UciMessage::ReadyOk,
			"copyprotection" | "registration" => {
				let state = match tokens.get(1).copied() {
					Some("checking") => ProtectionState::Checking,
					Some("ok") => ProtectionState::Ok,
					Some("error") => ProtectionState::Error,
					_ => ProtectionState::Error,
				};

				if tokens[0] == "copyprotection" {
					UciMessage::CopyProtection(state)
				} else {
					UciMessage::Registration(state)
				}
			}
			"info" => {
				let mut infos = Vec::new();
				let mut i = 1;

				while i < tokens.len() {
					match tokens[i] {
						"depth" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(d) = v.parse() {
									infos.push(UciInfoAttribute::Depth(d));
								}
							}
							i += 2;
						}

						"seldepth" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(d) = v.parse() {
									infos.push(UciInfoAttribute::SelDepth(d));
								}
							}
							i += 2;
						}

						"time" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(ms) = v.parse::<u64>() {
									infos.push(UciInfoAttribute::Time(std::time::Duration::from_millis(ms)));
								}
							}
							i += 2;
						}

						"nodes" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::Nodes(n));
								}
							}
							i += 2;
						}

						"nps" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::Nps(n));
								}
							}
							i += 2;
						}

						"hashfull" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(h) = v.parse() {
									infos.push(UciInfoAttribute::HashFull(h));
								}
							}
							i += 2;
						}

						"tbhits" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::TbHits(n));
								}
							}
							i += 2;
						}

						"sbhits" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::SbHits(n));
								}
							}
							i += 2;
						}

						"cpuload" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::CpuLoad(n));
								}
							}
							i += 2;
						}

						"currmove" => {
							if let Some(m) = tokens.get(i + 1) {
								infos.push(UciInfoAttribute::CurrMove((*m).to_string()));
							}
							i += 2;
						}

						"currmovenum" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::CurrMoveNum(n));
								}
							}
							i += 2;
						}

						"multipv" => {
							if let Some(v) = tokens.get(i + 1) {
								if let Ok(n) = v.parse() {
									infos.push(UciInfoAttribute::MultiPv(n));
								}
							}
							i += 2;
						}

						"score" => {
							let mut cp = None;
							let mut mate = None;
							let mut lower_bound = None;
							let mut upper_bound = None;

							i += 1;

							while i < tokens.len() {
								match tokens[i] {
									"cp" => {
										if let Some(v) = tokens.get(i + 1) {
											cp = v.parse().ok();
										}
										i += 2;
									}
									"mate" => {
										if let Some(v) = tokens.get(i + 1) {
											mate = v.parse().ok();
										}
										i += 2;
									}
									"lowerbound" => {
										lower_bound = Some(true);
										i += 1;
									}
									"upperbound" => {
										upper_bound = Some(true);
										i += 1;
									}
									_ => break,
								}
							}

							infos.push(UciInfoAttribute::Score {
								cp,
								mate,
								lower_bound,
								upper_bound,
							});
						}

						"pv" => {
							i += 1;
							let mut moves = Vec::new();

							while i < tokens.len() {
								// stop si prochain keyword connu
								match tokens[i] {
									"depth" | "nodes" | "score" | "nps" | "time" | "multipv" => break,
									_ => {
										moves.push(tokens[i].to_string());
										i += 1;
									}
								}
							}

							infos.push(UciInfoAttribute::Pv(moves));
						}

						"string" => {
							i += 1;
							let s = tokens[i..].join(" ");
							infos.push(UciInfoAttribute::String(s));
							break;
						}

						"refutation" => {
							i += 1;
							let mut moves = Vec::new();

							while i < tokens.len() {
								moves.push(tokens[i].to_string());
								i += 1;
							}

							infos.push(UciInfoAttribute::Refutation(moves));
							break;
						}

						other => {
							let value = tokens.get(i + 1).unwrap_or(&"").to_string();
							infos.push(UciInfoAttribute::Any(other.to_string(), value));
							i += 2;
						}
					}
				}

				UciMessage::Info(infos)
			}
			"option" => {
				let mut name = String::new();
				let mut opt_type = "";
				let mut default: Option<String> = None;
				let mut min: Option<i64> = None;
				let mut max: Option<i64> = None;
				let mut vars: Vec<String> = Vec::new();

				let mut i = 1;

				while i < tokens.len() {
					match tokens[i] {
						"name" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() && tokens[i] != "type" {
								parts.push(tokens[i]);
								i += 1;
							}
							name = parts.join(" ");
							continue;
						}

						"type" => {
							opt_type = tokens.get(i + 1).copied().unwrap_or("");
							i += 2;
							continue;
						}

						"default" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() && tokens[i] != "var" {
								parts.push(tokens[i]);
								i += 1;
							}
							default = Some(parts.join(" "));
							continue;
						}

						"min" => {
							min = tokens.get(i + 1).and_then(|v| v.parse().ok());
							i += 2;
							continue;
						}

						"max" => {
							max = tokens.get(i + 1).and_then(|v| v.parse().ok());
							i += 2;
							continue;
						}

						"var" => {
							i += 1;
							let mut parts = Vec::new();
							while i < tokens.len() && tokens[i] != "var" {
								parts.push(tokens[i]);
								i += 1;
							}
							vars.push(parts.join(" "));
							continue;
						}

						_ => {}
					}

					i += 1;
				}

				let option = match opt_type {
					"check" => UciOptionConfig::Check {
						name,
						default: default.as_deref().map(|v| v == "true"),
					},

					"spin" => UciOptionConfig::Spin {
						name,
						default: default.and_then(|v| v.parse().ok()),
						min,
						max,
					},

					"combo" => UciOptionConfig::Combo {
						name,
						default,
						var: vars,
					},

					"button" => UciOptionConfig::Button {
						name,
					},

					"string" => UciOptionConfig::String {
						name,
						default,
					},

					_ => {
						return UciMessage::Unknown(line.to_string());
					}
				};

				UciMessage::Option(option)
			}
			_ => UciMessage::Unknown(line.to_string())
		}
	}
}
impl UciMessage{
	/// Constructs a `register later` [UciMessage::Register](enum.UciMessage.html#variant.Register)  message.
	pub fn register_later() -> UciMessage{
		UciMessage::Register {
			later: true,
			name: None,
			code: None,
		}
	}

	/// Constructs a `register <code> <name>` [UciMessage::Register](enum.UciMessage.html#variant.Register) message.
	pub fn register_code(name: &str, code: &str) -> UciMessage{
		UciMessage::Register {
			later: false,
			name: Some(name.to_string()),
			code: Some(code.to_string()),
		}
	}

	/// Constructs an empty [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
	pub fn go() -> UciMessage{
		UciMessage::Go {
			search_control: None,
			time_control: None,
		}
	}

	/// Construct a `go ponder` [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
	pub fn go_ponder() -> UciMessage{
		UciMessage::Go {
			search_control: None,
			time_control: Some(UciTimeControl::Ponder),
		}
	}

	/// Constructs a `go infinite` [UciMessage::Register](enum.UciMessage.html#variant.Go) message.
	pub fn go_infinite() -> UciMessage{
		UciMessage::Go {
			search_control: None,
			time_control: Some(UciTimeControl::Infinite)
		}
	}

	/// Constructs a `go movetime <milliseconds>` [UciMessage::Register](enum.UciMessage.html#variant.Go) message, with
	/// `milliseconds` as the argument.
	pub fn go_movetime(milliseconds: Duration) -> UciMessage{
		UciMessage::Go {
			search_control: None,
			time_control: Some(UciTimeControl::MoveTime(milliseconds)),
		}
	}

	/// Constructs an `id <name>` GUI-bound message.
	pub fn id_name(name: &str) -> UciMessage{
		UciMessage::Id {
			name: Some(name.to_string()),
			author: None,
		}
	}

	/// Constructs an `id <name>` GUI-bound message.
	pub fn id_author(author: &str) -> UciMessage{
		UciMessage::Id {
			name: None,
			author: Some(author.to_string()),
		}
	}

	/// Constructs a `bestmove` GUI-bound message without the ponder move.
	pub fn best_move(best_move: String) -> UciMessage{
		UciMessage::BestMove {
			best_move,
			ponder: None,
		}
	}

	/// Constructs a `bestmove` GUI-bound message _with_ the ponder move.
	pub fn best_move_with_ponder(best_move: String, ponder: String) -> UciMessage{
		UciMessage::BestMove {
			best_move,
			ponder: Some(ponder),
		}
	}

	/// Constructs an `info string ...` message.
	pub fn info_string(s: String) -> UciMessage{
		UciMessage::Info(vec![UciInfoAttribute::String(s)])
	}

	/// Returns whether the command was meant for the engine or for the GUI.
	pub fn direction(&self) -> CommunicationDirection {
		match self {
			UciMessage::Uci |
			UciMessage::Debug(..) |
			UciMessage::IsReady |
			UciMessage::Register { .. } |
			UciMessage::Position { .. } |
			UciMessage::SetOption { .. } |
			UciMessage::UciNewGame |
			UciMessage::Stop |
			UciMessage::PonderHit |
			UciMessage::Quit |
			UciMessage::Go { .. } => CommunicationDirection::GuiToEngine,
			_ => CommunicationDirection::EngineToGui
		}
	}

	/// If this `UciMessage` is a `UciMessage::SetOption` and the value of that option is a `bool`, this method returns
	/// the `bool` value, otherwise it returns `None`.
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			UciMessage::SetOption { value, .. } => {
				if let Some(val) = value {
					let pr = str::parse(val.as_str());
					if let Ok(v) = pr {
						return Some(v);
					}
				}

				None
			}
			_ => None
		}
	}

	/// If this `UciMessage` is a `UciMessage::SetOption` and the value of that option is an integer, this method
	/// returns the `i32` value of the integer, otherwise it returns `None`.
	pub fn as_i32(&self) -> Option<i32> {
		match self {
			UciMessage::SetOption { value, .. } => {
				if let Some(val) = value {
					let pr = str::parse(val.as_str());
					if let Ok(v) = pr {
						return Some(v);
					}
				}

				None
			}
			_ => None
		}
	}

	/// Return `true` if this `UciMessage` is of variant `UnknownMessage`.
	pub fn is_unknown(&self) -> bool {
		matches!(self, UciMessage::Unknown(..))
	}
}

impl Display for UciMessage{
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.serialize())
	}
}

impl Serializable for UciMessage{
	/// Serializes the command into a String.
	///
	/// # Examples
	/// ```
	/// use vampirc_uci::{UciMessage, Serializable};
	///
	/// println!("{}", UciMessage::Uci.serialize()); // Should print `uci`.
	/// ```
	fn serialize(&self) -> String {
		match self {
			UciMessage::Debug(on) => if *on { String::from("debug on") } else { String::from("debug off") },
			UciMessage::Register { later, name, code } => {
				if *later {
					return String::from("register later");
				}

				let mut s: String = String::from("register ");
				if let Some(n) = name {
					s += format!("name {}", *n).as_str();
					if code.is_some() {
						s += " ";
					}
				}
				if let Some(c) = code {
					s += format!("code {}", *c).as_str();
				}

				s
			}
			UciMessage::Position { startpos, fen, moves } => {
				let mut s = String::from("position ");
				if *startpos {
					s += String::from("startpos").as_str();
				} else if let Some(uci_fen) = fen {
					s += format!("fen {}", uci_fen.as_str()).as_str();
				}

				if !moves.is_empty() {
					s += String::from(" moves").as_str();

					for m in moves {
						s += format!(" {}", *m).as_str();
					}
				}

				s
			}
			UciMessage::SetOption { name, value } => {
				let mut s: String = format!("setoption name {}", name);

				if let Some(val) = value {
					if val.is_empty() {
						s += " value <empty>";
					} else {
						s += format!(" value {}", *val).as_str();
					}
				} else {
					s += " value <empty>";
				}

				s
			}
			UciMessage::Go { time_control, search_control } => {
				let mut s = String::from("go ");

				if let Some(tc) = time_control {
					match tc {
						UciTimeControl::Infinite => { s += "infinite "; }
						UciTimeControl::Ponder => { s += "ponder "; }
						UciTimeControl::MoveTime(duration) => {
							s += format!("movetime {} ", duration.as_millis()).as_str();
						}
						UciTimeControl::TimeLeft { white_time, black_time, white_increment, black_increment, moves_to_go } => {
							if let Some(wt) = white_time {
								s += format!("wtime {} ", wt.as_millis()).as_str();
							}

							if let Some(bt) = black_time {
								s += format!("btime {} ", bt.as_millis()).as_str();
							}

							if let Some(wi) = white_increment {
								s += format!("winc {} ", wi.as_millis()).as_str();
							}

							if let Some(bi) = black_increment {
								s += format!("binc {} ", bi.as_millis()).as_str();
							}

							if let Some(mtg) = moves_to_go {
								s += format!("movestogo {} ", *mtg).as_str();
							}
						}
					}
				}

				if let Some(sc) = search_control {
					if let Some(depth) = sc.depth {
						s += format!("depth {} ", depth).as_str();
					}

					if let Some(nodes) = sc.nodes {
						s += format!("nodes {} ", nodes).as_str();
					}

					if let Some(mate) = sc.mate {
						s += format!("mate {} ", mate).as_str();
					}

					if !sc.search_moves.is_empty() {
						s += " searchmoves ";
						for m in &sc.search_moves {
							s += format!("{} ", m).as_str();
						}
					}
				}

				s.trim_end().to_string()
			}
			UciMessage::Uci => "uci".to_string(),
			UciMessage::IsReady => "isready".to_string(),
			UciMessage::UciNewGame => "ucinewgame".to_string(),
			UciMessage::Stop => "stop".to_string(),
			UciMessage::PonderHit => "ponderhit".to_string(),
			UciMessage::Quit => "quit".to_string(),


			// GUI-bound from this point on

			UciMessage::Id { name, author } => {
				let mut s = String::from("id ");
				if let Some(n) = name {
					s += &format!("name {}", n);
				}
				if let Some(a) = author {
					if !s.is_empty() { s += " "; }
					s += &format!("author {}", a);
				}

				s
			},
			UciMessage::UciOk => String::from("uciok"),
			UciMessage::ReadyOk => String::from("readyok"),
			UciMessage::BestMove { best_move, ponder } => {
				let mut s = format!("bestmove {}", *best_move);

				if let Some(p) = ponder {
					s += format!(" ponder {}", *p).as_str();
				}

				s
			},
			UciMessage::CopyProtection(cp_state) | UciMessage::Registration(cp_state) => {
				let mut s = match self {
					UciMessage::CopyProtection(..) => String::from("copyprotection "),
					UciMessage::Registration(..) => String::from("registration "),
					_ => unreachable!()
				};

				match cp_state {
					ProtectionState::Checking => s += "checking",
					ProtectionState::Ok => s += "ok",
					ProtectionState::Error => s += "error",
				}

				s
			},
			UciMessage::Option(config) => config.serialize(),
			UciMessage::Info(info_line) => {
				let mut s = String::from("info");

				for a in info_line {
					s += &format!(" {}", a.serialize());
				}

				s
			},
			UciMessage::Unknown(msg, ..) => {
				format!("UNKNOWN MESSAGE: {}", msg)

			}
		}
	}
}



/// This enum represents the possible variants of the `go` UCI message that deal with the chess game's time controls
/// and the engine's thinking time.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciTimeControl {
	/// The `go ponder` message.
	Ponder,

	/// The `go infinite` message.
	Infinite,

	/// The information about the game's time controls.
	TimeLeft {
		/// White's time on the clock, in milliseconds.
		white_time: Option<Duration>,

		/// Black's time on the clock, in milliseconds.
		black_time: Option<Duration>,

		/// White's increment per move, in milliseconds.
		white_increment: Option<Duration>,

		/// Black's increment per move, in milliseconds.
		black_increment: Option<Duration>,

		/// The number of moves to go to the next time control.
		moves_to_go: Option<u8>,
	},

	/// Specifies how much time the engine should think about the move, in milliseconds.
	MoveTime(Duration)
}

impl UciTimeControl {
	/// Returns a `UciTimeControl::TimeLeft` with all members set to `None`.
	pub fn time_left() -> UciTimeControl {
		UciTimeControl::TimeLeft {
			white_time: None,
			black_time: None,
			white_increment: None,
			black_increment: None,
			moves_to_go: None
		}
	}
}

/// A struct that controls the engine's (non-time-related) search settings.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct UciSearchControl {
	/// Limits the search to these moves.
	pub search_moves: Vec<String>,

	/// Search for mate in this many moves.
	pub mate: Option<u8>,

	/// Search to this ply depth.
	pub depth: Option<u8>,

	/// Search no more than this many nodes (positions).
	pub nodes: Option<u64>,
}

impl UciSearchControl{
	/// Creates an `UciSearchControl` with `depth` set to the parameter and everything else set to empty or `None`.
	pub fn depth(depth: u8) -> UciSearchControl{
		UciSearchControl {
			search_moves: vec![],
			mate: None,
			depth: Some(depth),
			nodes: None,
		}
	}

	/// Creates an `UciSearchControl` with `mate` set to the parameter and everything else set to empty or `None`.
	pub fn mate(mate: u8) -> UciSearchControl{
		UciSearchControl {
			search_moves: vec![],
			mate: Some(mate),
			depth: None,
			nodes: None,
		}
	}

	/// Creates an `UciSearchControl` with `nodes` set to the parameter and everything else set to empty or `None`.
	pub fn nodes(nodes: u64) -> UciSearchControl{
		UciSearchControl {
			search_moves: vec![],
			mate: None,
			depth: None,
			nodes: Some(nodes),
		}
	}

	/// Returns `true` if all of the struct's settings are either `None` or empty.
	pub fn is_empty(&self) -> bool {
		self.search_moves.is_empty() && self.mate.is_none() && self.depth.is_none() && self.nodes.is_none()
	}
}

impl Default for UciSearchControl{
	/// Creates an empty `UciSearchControl`.
	fn default() -> Self {
		UciSearchControl {
			search_moves: vec![],
			mate: None,
			depth: None,
			nodes: None,
		}
	}
}

/// Represents the copy protection or registration state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ProtectionState {
	/// Signifies the engine is checking the copy protection or registration.
	Checking,

	/// Signifies the copy protection or registration has been validated.
	Ok,

	/// Signifies error in copy protection or registratin validation.
	Error,
}

/// Represents a UCI option definition.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciOptionConfig {
	/// The option of type `check` (a boolean).
	Check {
		/// The name of the option.
		name: String,

		/// The default value of this `bool` property.
		default: Option<bool>,
	},

	/// The option of type `spin` (a signed integer).
	Spin {
		/// The name of the option.
		name: String,

		/// The default value of this integer property.
		default: Option<i64>,

		/// The minimal value of this integer property.
		min: Option<i64>,

		/// The maximal value of this integer property.
		max: Option<i64>,
	},

	/// The option of type `combo` (a list of strings).
	Combo {
		/// The name of the option.
		name: String,

		/// The default value for this list of strings.
		default: Option<String>,

		/// The list of acceptable strings.
		var: Vec<String>,
	},

	/// The option of type `button` (an action).
	Button {
		/// The name of the option.
		name: String
	},

	/// The option of type `string` (a string, unsurprisingly).
	String {
		/// The name of the option.
		name: String,

		/// The default value of this string option.
		default: Option<String>,
	},
}
impl Default for UciOptionConfig {
	fn default() -> Self {
		Self::Button {name:"".into()}
	}
}
impl UciOptionConfig {
	/// Returns the name of the option.
	pub fn get_name(&self) -> &str {
		match self {
			UciOptionConfig::Check { name, .. } | UciOptionConfig::Spin { name, .. } | UciOptionConfig::Combo { name, .. } | UciOptionConfig::Button { name } |
			UciOptionConfig::String { name, .. } => name.as_str()
		}
	}

	/// Returns the type string of the option (ie. `"check"`, `"spin"` ...)
	pub fn get_type_str(&self) -> &'static str {
		match self {
			UciOptionConfig::Check { .. } => "check",
			UciOptionConfig::Spin { .. } => "spin",
			UciOptionConfig::Combo { .. } => "combo",
			UciOptionConfig::Button { .. } => "button",
			UciOptionConfig::String { .. } => "string"
		}
	}
}

impl Serializable for UciOptionConfig {
	/// Serializes this option config into a full UCI message string.
	///
	/// # Examples
	///
	/// ```
	/// use vampirc_uci::{UciMessage, UciOptionConfig, Serializable};
	///
	/// let m = UciMessage::Option(UciOptionConfig::Check {
	///     name: String::from("Nullmove"),
	///     default: Some(true)
	/// });
	///
	/// assert_eq!(m.serialize(), "option name Nullmove type check default true");
	/// ```
	fn serialize(&self) -> String {
		let mut s = format!("option name {} type {}", self.get_name(), self.get_type_str());
		match self {
			UciOptionConfig::Check { default, .. } => {
				if let Some(def) = default {
					s += format!(" default {}", *def).as_str();
				}
			},
			UciOptionConfig::Spin { default, min, max, .. } => {
				if let Some(def) = default {
					s += format!(" default {}", *def).as_str();
				}

				if let Some(m) = min {
					s += format!(" min {}", *m).as_str();
				}

				if let Some(m) = max {
					s += format!(" max {}", *m).as_str();
				}
			}
			UciOptionConfig::Combo { default, var, .. } => {
				if let Some(def) = default {
					s += format!(" default {}", *def).as_str();
				}

				for v in var {
					s += format!(" var {}", *v).as_str();
				}
			}
			UciOptionConfig::String { default, .. } => {
				if let Some(def) = default {
					s += format!(" default {}", *def).as_str();
				}
			}
			UciOptionConfig::Button { .. } => {
				// Do nothing, we're already good
			}
		}

		s
	}
}

impl Display for UciOptionConfig {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.serialize())
	}
}

/// The representation of various info messages. For an info attribute that is not listed in the protocol specification,
/// the `UciInfoAttribute::Any(name, value)` variant can be used.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UciInfoAttribute {
	/// The `info depth` message.
	Depth(u8),

	/// The `info seldepth` message.
	SelDepth(u8),

	/// The `info time` message.
	Time(Duration),

	/// The `info nodes` message.
	Nodes(u64),

	/// The `info pv` message (best line move sequence).
	Pv(Vec<String>),

	/// The `info pv ... multipv` message (the pv line number in a multi pv sequence).
	MultiPv(u16),

	/// The `info score ...` message.
	Score {
		/// The score in centipawns.
		cp: Option<i32>,

		/// Mate coming up in this many moves. Negative value means the engine is getting mated.
		mate: Option<i8>,

		/// The value sent is the lower bound.
		lower_bound: Option<bool>,

		/// The value sent is the upper bound.
		upper_bound: Option<bool>,
	},

	/// The `info currmove` message (current move).
	CurrMove(String),

	/// The `info currmovenum` message (current move number).
	CurrMoveNum(u16),

	/// The `info hashfull` message (the occupancy of hashing tables in permills).
	HashFull(u16),

	/// The `info nps` message (nodes per second).
	Nps(u64),

	/// The `info tbhits` message (end-game table-base hits).
	TbHits(u64),

	/// The `info sbhits` message (I guess some Shredder-specific end-game table-base stuff. I dunno, probably best to
	/// ignore).
	SbHits(u64),

	/// The `info cpuload` message (CPU load in permills).
	CpuLoad(u16),

	/// The `info string` message (a string the GUI should display).
	String(String),

	/// The `info refutation` message (the first move is the move being refuted).
	Refutation(Vec<String>),

	/// The `info currline` message (current line being calculated on a CPU).
	CurrLine {
		/// The CPU number calculating this line.
		cpu_nr: Option<u16>,

		/// The line being calculated.
		line: Vec<String>,
	},

	/// Any other info line in the format `(name, value)`.
	Any(String, String),
}

impl UciInfoAttribute{
	/// Creates a `UciInfoAttribute::Score` with the `cp` attribute set to the value of the parameter and all other
	/// fields set to `None`.
	pub fn from_centipawns(cp: i32) -> UciInfoAttribute{
		UciInfoAttribute::Score {
			cp: Some(cp),
			mate: None,
			lower_bound: None,
			upper_bound: None,
		}
	}

	/// Creates a `UciInfoAttribute::Score` with the `mate` attribute set to the value of the parameter and all other
	/// fields set to `None`. A negative value indicates it is the engine that is getting mated.
	pub fn from_mate(mate: i8) -> UciInfoAttribute{
		UciInfoAttribute::Score {
			cp: None,
			mate: Some(mate),
			lower_bound: None,
			upper_bound: None,
		}
	}

	/// Returns the name of the info attribute.
	pub fn get_name(&self) -> &str {
		match self {
			UciInfoAttribute::Depth(..) => "depth",
			UciInfoAttribute::SelDepth(..) => "seldepth",
			UciInfoAttribute::Time(..) => "time",
			UciInfoAttribute::Nodes(..) => "nodes",
			UciInfoAttribute::Pv(..) => "pv",
			UciInfoAttribute::MultiPv(..) => "multipv",
			UciInfoAttribute::Score { .. } => "score",
			UciInfoAttribute::CurrMove(..) => "currmove",
			UciInfoAttribute::CurrMoveNum(..) => "currmovenum",
			UciInfoAttribute::HashFull(..) => "hashfull",
			UciInfoAttribute::Nps(..) => "nps",
			UciInfoAttribute::TbHits(..) => "tbhits",
			UciInfoAttribute::SbHits(..) => "sbhits",
			UciInfoAttribute::CpuLoad(..) => "cpuload",
			UciInfoAttribute::String(..) => "string",
			UciInfoAttribute::Refutation(..) => "refutation",
			UciInfoAttribute::CurrLine { .. } => "currline",
			UciInfoAttribute::Any(name, ..) => name.as_str()
		}
	}
}

impl Serializable for UciInfoAttribute{
	/// Returns the attribute serialized as a String.
	fn serialize(&self) -> String {
		let mut s = self.get_name().to_string();
		match self {
			UciInfoAttribute::Depth(depth) => s += format!(" {}", *depth).as_str(),
			UciInfoAttribute::SelDepth(depth) => s += format!(" {}", *depth).as_str(),
			UciInfoAttribute::Time(time) => s += format!(" {}", time.as_millis()).as_str(),
			UciInfoAttribute::Nodes(nodes) => s += format!(" {}", *nodes).as_str(),
			UciInfoAttribute::Pv(moves) | UciInfoAttribute::Refutation(moves) => {
				if !moves.is_empty() {
					for m in moves {
						s += format!(" {}", m).as_str();
					}
				}
			},
			UciInfoAttribute::MultiPv(num) => s += format!(" {}", *num).as_str(),
			UciInfoAttribute::Score { cp, mate, lower_bound, upper_bound } => {
				if let Some(c) = cp {
					s += format!(" cp {}", *c).as_str();
				}

				if let Some(m) = mate {
					s += format!(" mate {}", *m).as_str();
				}

				if lower_bound.is_some() {
					s += " lowerbound";
				} else if upper_bound.is_some() {
					s += " upperbound";
				}
			},
			UciInfoAttribute::CurrMove(uci_move) => s += &format!(" {}", *uci_move),
			UciInfoAttribute::CurrMoveNum(num) => s += &format!(" {}", *num),
			UciInfoAttribute::HashFull(permill) => s += &format!(" {}", *permill),
			UciInfoAttribute::Nps(nps) => s += &format!(" {}", *nps),
			UciInfoAttribute::TbHits(hits) | UciInfoAttribute::SbHits(hits) => s += &format!(" {}", *hits),
			UciInfoAttribute::CpuLoad(load) => s += &format!(" {}", *load),
			UciInfoAttribute::String(string) => s += &format!(" {}", string),
			UciInfoAttribute::CurrLine { cpu_nr, line } => {
				if let Some(c) = cpu_nr {
					s += &format!(" cpunr {}", *c);
				}

				if !line.is_empty() {
					for m in line {
						s += &format!(" {}", m);
					}
				}
			},
			UciInfoAttribute::Any(_, value) => {
				s += &format!(" {}", value);
			}
		}

		s
	}
}

impl Display for UciInfoAttribute{
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.serialize())
	}
}
/*
/// An enum representing the chess piece types.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[cfg(not(feature = "chess"))]
pub enum UciPiece {
	Pawn,
	Knight,
	Bishop,
	Rook,
	Queen,
	King,
}

#[cfg(not(feature = "chess"))]
impl UciPiece {
	/// Returns a character representing a piece in UCI move notation. Used for specifying promotion in moves.
	///
	/// `n` – knight
	/// `b` - bishop
	/// `r` - rook
	/// `q` - queen
	/// `k` - king
	/// `None` - pawn
	pub fn as_char(self) -> Option<char> {
		match self {
			UciPiece::Pawn => None,
			UciPiece::Knight => Some('n'),
			UciPiece::Bishop => Some('b'),
			UciPiece::Rook => Some('r'),
			UciPiece::Queen => Some('q'),
			UciPiece::King => Some('k')
		}
	}
}

#[cfg(not(feature = "chess"))]
impl FromStr for UciPiece {
	type Err = FmtError;

	/// Creates a `UciPiece` from a `&str`, according to these rules:
	///
	/// `"n"` - Knight
	/// `"p"` - Pawn
	/// `"b"` - Bishop
	/// `"r"` - Rook
	/// `"k"` - King
	/// `"q"` - Queen
	///
	/// Works with uppercase letters as well.
	fn from_str(s: &str) -> Result<UciPiece, FmtError> {
		match s.to_ascii_lowercase().as_str() {
			"n" => Ok(UciPiece::Knight),
			"p" => Ok(UciPiece::Pawn),
			"b" => Ok(UciPiece::Bishop),
			"r" => Ok(UciPiece::Rook),
			"k" => Ok(UciPiece::King),
			"q" => Ok(UciPiece::Queen),
			_ => Err(FmtError)
		}
	}
}

/// A representation of a chessboard square.
#[cfg(not(feature = "chess"))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UciSquare {
	/// The file. A character in the range of `a..h`.
	pub file: char,

	/// The rank. A number in the range of `1..8`.
	pub rank: u8,
}

#[cfg(not(feature = "chess"))]
impl UciSquare {
	/// Create a `UciSquare` from file character and a rank number.
	pub fn from(file: char, rank: u8) -> UciSquare {
		UciSquare {
			file,
			rank,
		}
	}
}

#[cfg(not(feature = "chess"))]
impl Display for UciSquare {
	/// Formats the square in the regular notation (as in, `e4`).
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}{}", self.file, self.rank)
	}
}

#[cfg(not(feature = "chess"))]
impl Default for UciSquare {
	/// Default square is an invalid square with a file of `\0` and the rank of `0`.
	fn default() -> Self {
		UciSquare {
			file: '\0',
			rank: 0,
		}
	}
}
*/
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
/// A representation of the notation in the [FEN notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
pub struct UciFen(pub String);

impl UciFen {
	/// Returns the FEN string.
	#[inline]
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl From<&str> for UciFen {
	/// Constructs an UciFen object from a `&str` containing a [FEN](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation)
	/// position. Does not validate the FEN.
	fn from(s: &str) -> Self {
		UciFen(s.to_string())
	}
}

impl Display for UciFen {
	/// Outputs the FEN string.
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		write!(f, "{}", self.0)
	}
}


/// A vector containing several `UciMessage`s.
pub type MessageList = Vec<UciMessage>;

/// A wrapper that keeps the serialized form in a byte vector. Mostly useful to provide an `AsRef<[u8]>` implementation for
/// quick conversion to an array of bytes. Use the `::from(m: UciMessage)` to construct it. It will add the newline
/// character `\n` to the serialized message.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct ByteVecUciMessage {
	pub message: UciMessage,
	pub bytes: Vec<u8>,
}

impl Display for ByteVecUciMessage{
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		write!(f, "{}", self.message)
	}
}

impl From<UciMessage> for ByteVecUciMessage{
	fn from(m: UciMessage) -> Self {
		let b = Vec::from((m.serialize() + "\n").as_bytes());
		ByteVecUciMessage {
			message: m,
			bytes: b,
		}
	}
}
impl From<ByteVecUciMessage> for UciMessage{
	fn from(val: ByteVecUciMessage) -> Self {
		val.message
	}
}
impl AsRef<UciMessage> for ByteVecUciMessage{
	fn as_ref(&self) -> &UciMessage{
		&self.message
	}
}

impl AsRef<[u8]> for ByteVecUciMessage{
	fn as_ref(&self) -> &[u8] {
		self.bytes.as_ref()
	}
}
