use crate::{
	Player,
	ai::minimax::{
		Game,
		gametree::{GameTree, Node, StateInfo},
	},
	gui::{BoardGame, BoardMove},
	sgf::parse_tree::SGFTreeNode,
};
mod parse_tree;
mod parser;
mod scanner;
use regex::Regex;

pub fn filerank_to_coords(values: &str) -> Option<(u8, u8)> {
	let re = Regex::new(r"([a-zA-Z]{1,2})([0-9]{1,3})").unwrap();
	let cap = re.captures(values)?;

	let letters = cap.get(1)?.as_str().to_lowercase();
	let mut x: u16 = 0;

	for b in letters.bytes() {
		if !(b'a'..=b'z').contains(&b) {
			return None;
		}
		x = x * 26 + (b - b'a') as u16;
		if x > u8::MAX as u16 {
			return None;
		}
	}

	let y_val: u16 = cap.get(2)?.as_str().parse().ok()?;
	if y_val == 0 || y_val > 256 {
		return None;
	}

	Some((x as u8, (y_val - 1) as u8))
}
pub fn letters_to_index(values: &str) -> Option<u8> {
	let letters = values.to_lowercase();
	let mut x: u32 = 0;

	for b in letters.bytes() {
		if !(b'a'..=b'z').contains(&b) {
			return None;
		}
		x = x * 26 + (b - b'a') as u32;

		if x > u8::MAX as u32 {
			return None;
		}
	}

	Some(x as u8)
}
fn letters_group_to_index(s: &str) -> Option<u8> {
	let mut x: u32 = 0;

	for b in s.bytes() {
		if !b.is_ascii_alphabetic() {
			return None;
		}
		let c = b.to_ascii_lowercase();
		if !(b'a'..=b'z').contains(&c) {
			return None;
		}
		x = x * 26 + (c - b'a') as u32;
		if x > u8::MAX as u32 {
			return None;
		}
	}

	Some(x as u8)
}

pub fn letters_to_coords(values: &str) -> Option<(u8, u8)> {
	let bytes = values.as_bytes();
	if bytes.len() < 2 || bytes.len() > 4 {
		return None;
	}
	if !bytes.iter().all(|b| b.is_ascii_alphabetic()) {
		return None;
	}

	match bytes.len() {
		2 => {
			let x = (bytes[0].to_ascii_lowercase() - b'a') as u8;
			let y = (bytes[1].to_ascii_lowercase() - b'a') as u8;
			if x < 26 && y < 26 { Some((x, y)) } else { None }
		}
		3 | 4 => {
			let mut split = 1;
			while split < bytes.len()
				&& (bytes[split].is_ascii_lowercase() == bytes[0].is_ascii_lowercase())
			{
				split += 1;
			}

			if split == bytes.len() {
				// no case change -> error
				return None;
			}

			let (file, rank) = values.split_at(split);

			if file.len() == 0 || rank.len() == 0 || file.len() > 2 || rank.len() > 2 {
				return None;
			}

			let x = letters_group_to_index(file)?;
			let y = letters_group_to_index(rank)?;
			Some((x, y))
		}
		_ => None,
	}
}
pub trait GFSSerializableGame: BoardGame
where
	<Self as Game>::M: BoardMove<Self>,
{
	fn coord_to_sgf(x: u8, y: u8) -> String {
		let cx = (b'a' + x as u8) as char;
		let cy = (b'a' + y as u8) as char;
		format!("{}{}", cx, cy)
	}
	fn coord_to_filerank(x: u8, y: u8) -> String {
		let cx = (b'a' + x as u8) as char;
		let cy = (y + 1).to_string();
		format!("{}{}", cx, cy)
	}
	fn string_to_coord(values: &str) -> Option<(u8, u8)> {
		letters_to_coords(values)
	}
	fn sgf_to_coord_list(values: &str) -> Vec<(u8, u8)> {
		let mut res = vec![];
		let re_val = Regex::new(r"\[([a-z])([a-z])\]").unwrap();
		for v in re_val.captures_iter(values) {
			let x = (v[1].as_bytes()[0] - b'a') as u8;
			let y = (v[2].as_bytes()[0] - b'a') as u8;
			res.push((x, y));
		}
		res
	}

	fn filerank_to_coord_list(values: &str) -> Vec<(u8, u8)> {
		let mut res = vec![];
		let re = Regex::new(r"\[([a-zA-Z]{1,2})([0-9]{1-3})\]").unwrap();

		for cap in re.captures_iter(values) {
			let letters = cap[1].to_lowercase();
			let mut x = 0u8;

			for b in letters.bytes() {
				x = x * 26 + (b - b'a') as u8;
			}

			let y = cap[2].parse::<u8>().unwrap() - 1;

			res.push((x, y));
		}

		res
	}

	fn mnemo_player_move() -> Vec<String> {
		vec!["B".into(), "W".into()]
	}
	//TODO:
	//fn mnemo_add_pieces() -> std::collections::HashMap<Self::PieceType, String>;
	fn mnemo_add_pieces() -> Vec<String> {
		vec!["AB".into(), "AW".into()]
	}
	fn mnemo_player_turn() -> String {
		"PL".into()
	}
	fn str_to_coords(piece_type: &String) -> Vec<(u8, u8)>;
	//TODO: fn build(player_on_turn: Option<Player>, pieces: HashMap<String, Vec<(u8, u8)>>) -> Self;
	fn build(player_on_turn: Option<Player>, pieces: Vec<(String, Vec<(u8, u8)>)>) -> Self;
}
pub struct SGFReaderWriter {}
impl SGFReaderWriter {
	pub fn serialize_game_state<G>(game: &G) -> String
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let mut sgf = String::from("(;");

		// Size
		let width = G::width(&game);
		let height = G::height(&game);
		if width == height {
			sgf.push_str(format!("SZ[{}]", width).as_str());
		} else {
			sgf.push_str(format!("SZ[{}:{}]", width, height).as_str());
		}
		for t in G::mnemo_add_pieces() {
			let p = G::str_to_coords(&t);
			if !p.is_empty() {
				sgf.push_str(t.as_str());
				for (x, y) in p {
					sgf.push_str(&format!("[{}]", G::coord_to_sgf(x, y)));
				}
			}
		}

		// Player on turn
		sgf.push_str(&format!(
			"PL[{}]",
			match G::get_current_player(&game) {
				Player::PLAYER1 => "B",
				Player::PLAYER2 => "W",
				_ => unreachable!(),
			}
		));

		sgf.push_str(")");
		sgf
	}
	pub fn serialize_game_tree<G>(game_tree: &GameTree<G>) -> String
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let mut sgf = String::from("(;");
		let game = game_tree.get_node_state(game_tree.root_id).unwrap();
		// Size
		let width = G::width(game);
		let height = G::height(game);
		if width == height {
			sgf.push_str(format!("SZ[{}]", width).as_str());
		} else {
			sgf.push_str(format!("SZ[{}:{}]", width, height).as_str());
		}
		for t in G::mnemo_add_pieces() {
			let p = G::str_to_coords(&t);
			if !p.is_empty() {
				sgf.push_str(t.as_str());
				for (x, y) in p {
					sgf.push_str(&format!("[{}]", G::coord_to_sgf(x, y)));
				}
			}
		}

		// Player on turn
		sgf.push_str(&format!(
			"PL[{}]",
			match G::get_current_player(&game) {
				Player::PLAYER1 => "B",
				Player::PLAYER2 => "W",
				_ => unreachable!(),
			}
		));
		//TODO: serialize children (ex: ;W[a1][a5];B[g7][g5]

		sgf.push_str(")");
		sgf
	}
	/// Parse a game state. Root will be parsed using AW/AB or equivalent
	/// and principal sequence of move will be applied from root.
	/// The resulting state will be returned.
	/// Variations are ignored.
	pub fn parse_game_state<G>(input: &str) -> Result<G, String>
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		//TODO: use scanner/parser
		// Property SGF : AB[aa][bb], AW[cc], PL[B], PL[W], C[This is a comment], etc.
		let re_prop = Regex::new(r"([A-Z]+)((\[[A-Za-z]+\])+)").unwrap();
		//let re_prop = Regex::new(r"([A-Z]+)((\[(?:\\.|[^\]])*\])+)").unwrap();

		let mut turn = Player::PLAYER1;
		let mut pieces = vec![];
		let pieces_types = G::mnemo_add_pieces();

		for cap in re_prop.captures_iter(input) {
			let prop = &cap[1];
			let values = &cap[2];
			match prop {
				"PL" => {
					turn = Player(
						G::mnemo_player_move()
							.iter()
							.position(|r| r == values)
							.unwrap_or_default() as u8,
					)
				}
				prop_name => {
					if pieces_types.contains(&prop_name.into()) {
						pieces.push((prop_name.to_owned(), G::sgf_to_coord_list(values)));
					}
				}
			}
		}
		//TODO: apply move of the main sequence
		Ok(G::build(Some(turn), pieces))
	}
	/// Parse a hole game tree
	pub fn parse_game_tree<G>(input: &str) -> Result<GameTree<G>, String>
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let scanner = scanner::Scanner::new();
		let mut p = parser::Parser::new(scanner);
		let tree = p.parse(input);
		if !tree.errors.is_empty() {
			return Err(tree.errors[0].message.clone());
		}
		let tree = tree.root.unwrap().eval_start(&mut vec![]);

		let mut game_tree = GameTree::default();
		Self::parse_node(&mut game_tree, &tree, None);
		Ok(game_tree)
	}
	fn parse_node<G>(game_tree: &mut GameTree<G>, node: &SGFTreeNode, parent: Option<usize>)
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let mut pieces = vec![];
		for piece_name in G::mnemo_add_pieces() {
			let mut pieces_coords = vec![];
			while let Some(coords) = node.attributes.get(&piece_name) {
				for coord in coords {
					assert_eq!(coord.len(), 1);
					let coord = G::string_to_coord(&coord[0]).unwrap();
					pieces_coords.push(coord);
				}
			}
			pieces.push((piece_name, pieces_coords));
		}
		let state = G::build(None, pieces);
		let hash = G::get_hash(&state);
		let node_id = game_tree.nodes.len();
		for pl in G::mnemo_player_move() {
			let mut moves_coords = vec![];
			while let Some(coords) = node.attributes.get(&pl) {
				for coord in coords {
					assert_eq!(coord.len(), 1);
					let coord = G::string_to_coord(&coord[0]).unwrap();
					moves_coords.push(coord);
				}
			}
			//TODO: apply move
		}
		game_tree.nodes.push(Node::<G::M> {
			state_hash: hash,
			parent,
			children: vec![],
			visits: 0,
			wins: 0,
			draws: 0,
			untried_moves: vec![],
			player_to_move: G::get_current_player(&state),
			outcome: crate::GameOutcome::OnGoing,
			depth_to_end: u16::MAX,
			incoming_move: None,
		});
		game_tree.states.insert(
			hash,
			StateInfo {
				state,
				expanded_node: node_id,
			},
		);
		for c in &node.children {
			Self::parse_node(game_tree, c, Some(node_id));
		}
	}
}
