use crate::{
	Player,
	ai::move_search::{
		Game,
		gametree::{GameTree, Node},
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
		if !b.is_ascii_lowercase() {
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
fn letters_to_index(s: &str) -> Option<u8> {
	let mut x: u32 = 0;

	for b in s.bytes() {
		if !b.is_ascii_alphabetic() {
			return None;
		}

		let c = b.to_ascii_lowercase();
		x = x.checked_mul(26)?.checked_add((c - b'a') as u32)?;
	}

	x.try_into().ok()
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
			let x = bytes[0].to_ascii_lowercase() - b'a';
			let y = bytes[1].to_ascii_lowercase() - b'a';
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

			if file.is_empty() || rank.is_empty() || file.len() > 2 || rank.len() > 2 {
				return None;
			}

			let x = letters_to_index(file)?;
			let y = letters_to_index(rank)?;
			Some((x, y))
		}
		_ => None,
	}
}
fn index_to_letters(mut x: u8) -> String {
	let mut s = Vec::new();

	loop {
		let c = (b'a' + (x % 26)) as char;
		s.push(c);

		if x < 26 {
			break;
		}

		x = x / 26 - 1;
	}

	s.reverse();
	s.into_iter().collect()
}
pub fn coord_to_letters(x: u8, y: u8) -> String {
	let mut cx = index_to_letters(x);
	let cy = index_to_letters(y);
	if cx.len() > 1 || cy.len() > 1 {
		cx = cx.to_ascii_uppercase();
	}
	format!("{}{}", cx, cy)
}
pub fn coord_to_filerank(x: u8, y: u8) -> String {
	let cx = index_to_letters(x);
	let cy = (y + 1).to_string();
	format!("{}{}", cx, cy)
}
pub trait GFSSerializableGame: BoardGame
where
	<Self as Game>::M: BoardMove<Self>,
{
	fn string_to_coord(values: &str) -> Option<(u8, u8)> {
		let coords = filerank_to_coords(values);
		if coords.is_some() {
			return coords;
		}
		letters_to_coords(values)
	}
	fn coord_to_string(coord: (u8, u8)) -> String {
		coord_to_letters(coord.0, coord.1)
	}

	fn mnemo_players() -> Vec<&'static str> {
		vec!["B", "W"]
	}
	//TODO:
	//fn mnemo_add_pieces() -> std::collections::HashMap<Self::PieceType, String>;
	fn mnemo_add_pieces() -> Vec<&'static str> {
		vec!["AB", "AW"]
	}
	fn mnemo_player_turn() -> &'static str {
		"PL"
	}
	fn get_pieces_coords(game: &Self::S, piece_type: &str) -> Vec<(u8, u8)>;
	//TODO: fn build(player_on_turn: Option<Player>, pieces: HashMap<String, Vec<(u8, u8)>>) -> Self;
	fn build(player_on_turn: Option<Player>, pieces: Vec<(&'static str, Vec<(u8, u8)>)>) -> Self;
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
		let width = G::width(game);
		let height = G::height(game);
		if width == height {
			sgf.push_str(format!("SZ[{}]", width).as_str());
		} else {
			sgf.push_str(format!("SZ[{}:{}]", width, height).as_str());
		}
		for t in G::mnemo_add_pieces() {
			let p = G::get_pieces_coords(game, t);
			if !p.is_empty() {
				sgf.push_str(t);
				for coord in p {
					sgf.push_str(&format!("[{}]", G::coord_to_string(coord)));
				}
			}
		}

		// Player on turn
		sgf.push_str(&format!(
			"{}[{}]",
			G::mnemo_player_turn(),
			match G::get_current_player(game) {
				Player(i) => G::mnemo_players()[i as usize],
			}
		));

		sgf.push(')');
		sgf
	}
	pub fn serialize_game_tree<G, Data: Default>(game_tree: &GameTree<G, Data>) -> String
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
			let p = G::get_pieces_coords(game, t);
			if !p.is_empty() {
				sgf.push_str(t);
				for coord in p {
					sgf.push_str(&format!("[{}]", G::coord_to_string(coord)));
				}
			}
		}

		// Player on turn
		sgf.push_str(&format!(
			"PL[{}]",
			match G::get_current_player(game) {
				Player::PLAYER1 => "B",
				Player::PLAYER2 => "W",
				_ => unreachable!(),
			}
		));
		//TODO: serialize children (ex: ;W[a1][a5];B[g7][g5]

		sgf.push(')');
		sgf
	}
	/// Parse a game state. Root will be parsed using AW/AB or equivalent
	/// and principal sequence of move will be applied from root.
	/// The resulting state will be returned.
	/// Variations are ignored.
	pub fn parse_game_state<G, Data: Default>(input: &str) -> Result<G, String>
	where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let tree: GameTree<G, Data> = Self::parse_game_tree(input)?;
		let mut node = tree.get_root();
		while !node.children.is_empty() {
			node = &tree.nodes[node.children[0].child];
		}
		Ok(node.state.clone())
	}
	/// Parse a hole game tree
	pub fn parse_game_tree<G, Data: Default>(input: &str) -> Result<GameTree<G, Data>, String>
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
	fn parse_node<G, Data: Default>(
		game_tree: &mut GameTree<G, Data>,
		node: &SGFTreeNode,
		parent: Option<usize>,
	) where
		G: GFSSerializableGame,
		G::M: BoardMove<G>,
	{
		let mut pieces = vec![];
		for piece_name in G::mnemo_add_pieces() {
			let mut pieces_coords = vec![];
			while let Some(coords) = node.attributes.get(piece_name) {
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
		for pl in G::mnemo_players() {
			let mut moves_coords = vec![];
			while let Some(coords) = node.attributes.get(pl) {
				for coord in coords {
					assert_eq!(coord.len(), 1);
					let coord = G::string_to_coord(&coord[0]).unwrap();
					moves_coords.push(coord);
				}
			}
			//TODO: apply move
		}
		let player = G::get_current_player(&state);
		game_tree.nodes.push(Node::<G, Data> {
			state,
			parent,
			children: vec![],
			data: Data::default(),
			untried_moves: vec![],
			player_to_move: player,
			outcome: crate::GameOutcome::OnGoing,
			depth_to_end: u16::MAX,
		});
		game_tree.state_to_node.insert(hash, node_id);
		for c in &node.children {
			Self::parse_node(game_tree, c, Some(node_id));
		}
	}
}
