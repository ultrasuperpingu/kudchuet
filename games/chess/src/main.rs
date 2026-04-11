use crate::gui::create_board;

extern crate kudchuet;

mod bitboard;
mod chess2;
mod chess;
mod evaluation;
mod fen;
mod gui;
mod magic_tables;
mod mychess;
mod pext_tables;
mod san;
mod rules;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
	
	use kudchuet::ai::cli_engine::UCILikeCLIEngine;
	use kudchuet::new_move_searcher_static;
	use mychess::{ChessMaterialEval, ChessBoard};

	let args: Vec<String> = std::env::args().collect();

	if args.iter().any(|arg| arg == "--uci") {
		let searcher = new_move_searcher_static::<ChessBoard, _>(ChessMaterialEval {}, 5);
		let mut cli_engine = UCILikeCLIEngine::new(searcher);
		cli_engine.process().unwrap();
		Ok(())
	} else {
		eframe::run_native(
			"Chess",
			eframe::NativeOptions::default(),
			Box::new(|_cc| Ok(Box::new(create_board()))),
		)
	}
}

#[cfg(target_arch = "wasm32")]
use eframe::web_sys;
#[cfg(target_arch = "wasm32")]
fn main() {
	use wasm_bindgen::JsCast;

	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document");
	let canvas = document
		.get_element_by_id("canvas_id")
		.expect("canvas not found")
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.expect("element is not a canvas");

	wasm_bindgen_futures::spawn_local(async move {
		eframe::WebRunner::new()
			.start(
				canvas,
				eframe::WebOptions::default(),
				Box::new(|_cc| Ok(Box::new(create_board()))),
			)
			.await
			.expect("failed to start eframe");
	});
}

/*
fn main2() -> io::Result<()> {
	let mut parser = uci_parser::parser::Parser::new(uci_parser::scanner::Scanner::new());
	let mut chess_variant = ChessVariant::None;
	let mut debug:bool = true; 
	let mut pos = Chess::default();
	//let mut strategy = minimax::Negamax::new(ChessPosEval{}, 5);
	let mut strategy = minimax::ParallelSearch::new(ChessPosEval::new(), IterativeOptions::new().with_table_byte_size(128*1024*1024),ParallelOptions::new());
	
	//let mut iterative_opts: IterativeOptions = IterativeOptions::new();
	//let mut parallele_opts: ParallelOptions = ParallelOptions::new();
	

	let mut hash_size = 128usize;
	let mut nb_threads = None;
	let mut mtdf = false;
	let stdin = io::stdin(); // We get `Stdin` here.
	let mut is960   = false;
	loop {
		let mut buffer = String::new();
		stdin.read_line(&mut buffer)?;
		let parsed = parser.parse(&buffer.trim());
		println!("{:?}", parsed.errors.iter().count());
		let messages = parse(&buffer);
		let mut quit = false;
		for m in messages {
			if debug {
				println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("received: {}", m))]));
			}
			match m {
				UciMessage::Uci => {
					println!("{}", UciMessage::Id { name: Some("MyChess".into()), author: None });
					println!("{}", UciMessage::Id { name: None, author: Some("Ping".into()) });
					println!("{}", UciMessage::Option(UciOptionConfig::Spin {name:"Hash".into(), default: Some(128), min: Some(1), max: Some(33554432) }));
					println!("{}", UciMessage::Option(UciOptionConfig::Spin {name:"Threads".into(), default: None, min: Some(1), max: Some(64) }));
					println!("{}", UciMessage::Option(UciOptionConfig::Button { name: "Clear Hash".into() }));
					println!("{}", UciMessage::Option(UciOptionConfig::Check { name: "UCI_Chess960".into(), default:Some(false) }));
					println!("{}", UciMessage::Option(UciOptionConfig::Combo {
						name: "Chess Variant".into(),
						default: Some("None".into()),
						var: vec![
							"None".into(),
							"Chess960".into(),
							"KingOfTheHill".into(),
							"Antichess".into(),
							"ThreeCheck".into(),
							"Crazyhouse".into(),
							"RacingKings".into(),
							"Horde".into(),
						]
					}));
					println!("{}", UciMessage::Option(UciOptionConfig::Check { name: "Mtdf".into(), default:Some(false) }));
					
					println!("{}", UciMessage::UciOk);
				}
				UciMessage::UciNewGame => {
					strategy = minimax::ParallelSearch::new(ChessPosEval::new(), IterativeOptions::new(),ParallelOptions::new());
				}
				UciMessage::SetOption { name, value } => {
					if debug {
						println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("setting option {}={:?}", name, value))]));
					}
					let mut par_opts = ParallelOptions::new();
					par_opts.num_threads = nb_threads;
					let mut opts = IterativeOptions::new().with_table_byte_size(hash_size * 1024 * 1024);
					let mut ok = true;
					match name.as_str() {
						"Hash" => {
							hash_size = value.unwrap().parse().unwrap();
							if debug {
								println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("setting hash size to {}MB", hash_size))]));
							}
							opts = opts.with_table_byte_size(hash_size * 1024 * 1024);
						}
						"Clear Hash" => {
						}
						"Threads" => {
							nb_threads = value.unwrap().parse().ok();
							par_opts.num_threads = nb_threads;
						}
						"Mtdf" => {
							mtdf = value.unwrap().parse().is_ok_and(|b| b);
						}
						"UCI_Chess960" => {
							is960 = value.unwrap().parse().is_ok_and(|f| f);
						}
						"Chess Variant" => {
							chess_variant = ChessVariant::from_str(value.unwrap().as_str()).unwrap();
						}
						_ => {
							ok = false;
						}
					}
					if mtdf {
						opts = opts.with_mtdf();
					}
					if ok {
						strategy = minimax::ParallelSearch::new(ChessPosEval::new(), opts, par_opts);
					}
				}
				UciMessage::Position { startpos, fen, moves } => {
					if startpos {
						pos = Chess::default();
					}
					if let Some(fen_str) = fen {
						if let Ok(fen) = fen_str.0.parse::<Fen>() {
							if let Ok(p) = fen.into_position(if is960 {CastlingMode::Chess960 } else { CastlingMode::Standard}) {
								pos = p;
							}
						}
					}
					for m in moves {
						let ucimov : shakmaty::uci::UciMove = m.to_string().parse().unwrap();
						let mv = ucimov.to_move(&pos);
						pos.play_unchecked(mv.unwrap());
					}
				}
				UciMessage::Go { time_control, search_control } => {
					if let Some(tc) = time_control {
						match tc {
							UciTimeControl::Ponder => {
								// Put the engine into ponder mode ("think" on opponent's time)
							}
							UciTimeControl::MoveTime(t) => {
								strategy.set_timeout(core::time::Duration::from_millis(t.num_milliseconds() as u64));
							}
							_ => {}
						}
					}
					if let Some(sc) = search_control {
						if let Some(d) = sc.depth {
							strategy.set_max_depth(d);
						}
					}
					//strategy.set_timeout(timeout);
					let best_move = strategy.choose_move(&pos).unwrap();
					if debug {
						let pv_move=strategy.principal_variation();
						let pv = pv_move.iter().map(|m| m.to_uci(if is960 {CastlingMode::Chess960 } else { CastlingMode::Standard}));
						let score = strategy.root_value();
						println!("{}", UciMessage::Info(vec![UciInfoAttribute::Pv(pv.collect()), UciInfoAttribute::Score{ cp: Some(score as i32), mate: None, lower_bound: None, upper_bound: None }]));
					}
					let move_str : shakmaty::uci::UciMove = shakmaty::uci::UciMove::from_move(best_move, if is960 {CastlingMode::Chess960 } else { CastlingMode::Standard});
					println!("bestmove {}", move_str);
				},
				UciMessage::Quit => {
					quit = true;
					break;
				},
				UciMessage::Debug(d) => { debug=d; },
				UciMessage::IsReady => {
					println!("{}", UciMessage::ReadyOk);
				},
				_ => {
					println!("{}", UciMessage::Info(vec![UciInfoAttribute::String(format!("unknown command: {}", m))]));
				}
			}
		}
		if quit {
			break;
		}
	}
	Ok(())
}
*/