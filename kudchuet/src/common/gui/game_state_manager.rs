use crate::common::gui::{BoardGame, BoardMove};

pub struct GameStateManager<G: BoardGame>
	where G::M: BoardMove<G>
{
	current_game: G,
	move_history: Vec<G>,
	redo_history: Vec<G>,
	legal_moves: Vec<G::M>,
}

impl<G: BoardGame> GameStateManager<G> 
where G::M: BoardMove<G>
{
	pub fn new(initial_game: G) -> Self {
		let legals = initial_game.legal_moves();
		Self {
			current_game: initial_game,
			move_history: Vec::new(),
			redo_history: Vec::new(),
			legal_moves: legals,
		}
	}

	pub fn game(&self) -> &G {
		&self.current_game
	}

	pub fn legal_moves(&self) -> &[G::M] {
		&self.legal_moves
	}

	pub fn can_undo(&self) -> bool {
		!self.move_history.is_empty()
	}

	pub fn can_redo(&self) -> bool {
		!self.redo_history.is_empty()
	}

	pub fn apply_move(&mut self, mv: G::M) {
		self.move_history.push(self.current_game.clone());
		self.redo_history.clear();
		
		self.current_game.play(mv);
		self.update_legals();
	}

	pub fn undo(&mut self) -> bool {
		if let Some(prev_game) = self.move_history.pop() {
			self.redo_history.push(self.current_game.clone());
			self.current_game = prev_game;
			self.update_legals();
			return true;
		}
		false
	}

	pub fn redo(&mut self) -> bool {
		if let Some(next_game) = self.redo_history.pop() {
			self.move_history.push(self.current_game.clone());
			self.current_game = next_game;
			self.update_legals();
			return true;
		}
		false
	}

	pub fn reset(&mut self, new_game: G) {
		self.current_game = new_game;
		self.move_history.clear();
		self.redo_history.clear();
		self.update_legals();
	}

	pub fn set_game_state(&mut self, new_game: G) {
		self.current_game = new_game;
		self.update_legals();
	}

	fn update_legals(&mut self) {
		self.legal_moves = self.current_game.legal_moves();
	}
	pub fn play_random(&mut self) {
		self.current_game.play_random();
		self.update_legals();
	}
}