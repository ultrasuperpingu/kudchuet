
use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::bitboard_table;

#[bitboard(width=9,height=13)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard9x13;
bitboard_table!(RAY_N, ray_n, ray_n_mask, Bitboard9x13, Bitboard9x13::generate_ray_n_table());
bitboard_table!(RAY_S, ray_s, ray_s_mask, Bitboard9x13, Bitboard9x13::generate_ray_s_table());
bitboard_table!(RAY_W, ray_w, ray_w_mask, Bitboard9x13, Bitboard9x13::generate_ray_w_table());
bitboard_table!(RAY_E, ray_e, ray_e_mask, Bitboard9x13, Bitboard9x13::generate_ray_e_table());
bitboard_table!(RAY_NE, ray_ne, ray_ne_mask, Bitboard9x13, Bitboard9x13::generate_ray_ne_table());
bitboard_table!(RAY_NW, ray_nw, ray_nw_mask, Bitboard9x13, Bitboard9x13::generate_ray_nw_table());
bitboard_table!(RAY_SE, ray_se, ray_se_mask, Bitboard9x13, Bitboard9x13::generate_ray_se_table());
bitboard_table!(RAY_SW, ray_sw, ray_sw_mask, Bitboard9x13, Bitboard9x13::generate_ray_sw_table());
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard9x13, Bitboard9x13::generate_neighbors_ortho_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard9x13, Bitboard9x13::generate_neighbors_8_table());
impl Bitboard9x13 {
	/// Precomputed masks of all squares between any two squares on the bitboard.
	#[cfg(not(debug_assertions))]
	pub const RAY_BETWEEN_MASKS: [[Self; Self::NB_SQUARES]; Self::NB_SQUARES] = Self::generate_ray_between_table();
	/// Returns the precomputed bitboard mask of squares between `from` and `to`.
	#[inline(always)]
	pub fn ray_between_mask(from: usize, to: usize) -> Self {
		#[cfg(not(debug_assertions))]
		{
			Self::RAY_BETWEEN_MASKS[from][to]
		}
		#[cfg(debug_assertions)]
		{
		RAY_BETWEEN_MASKS[from][to]
		}
	}
	#[inline(always)]
	pub fn ray_between(&self, from: usize, to: usize) -> Self {
		#[cfg(not(debug_assertions))]
		{
			*self & Self::RAY_BETWEEN_MASKS[from][to]
		}
		#[cfg(debug_assertions)]
		{
			*self & RAY_BETWEEN_MASKS[from][to]
		}
	}
}
#[cfg(debug_assertions)]
pub static RAY_BETWEEN_MASKS: [[Bitboard9x13; Bitboard9x13::NB_SQUARES]; Bitboard9x13::NB_SQUARES] = Bitboard9x13::generate_ray_between_table();

impl Bitboard9x13 {
	pub const fn compute_shoot_mask(sq:u8) -> Self {
		let index = sq as usize;
		let mut mask = Self::compute_diag_inc_mask(index).storage() | Self::compute_diag_dec_mask(index).storage();
		mask |= Self::compute_col_mask(index).storage() | Self::compute_row_mask(index).storage();
		mask &= !Self::from_index(index).storage();
		Self::from_storage(mask)
	}
	pub const fn generate_pass_mask_table() -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_shoot_mask(i as u8);
			i += 1;
		}
		arr
	}
	pub const fn all_lines_at_south(sq:u8) -> Self {
		let mut i=0;
		let mut mask = Self::EMPTY.storage();
		while i < sq / Self::WIDTH * Self::WIDTH {
			mask |= Self::from_index(i as usize).storage();
			i+=1;
		}
		Bitboard9x13::from_storage(mask)
	}
	pub const fn all_lines_at_north(sq:u8) -> Self {
		let mut i=Self::NB_SQUARES-1;
		let mut mask = Self::EMPTY.storage();
		while i as u8 >= (sq / Self::WIDTH + 1) * Self::WIDTH {
			mask |= Self::from_index(i).storage();
			i-=1;
		}
		Bitboard9x13::from_storage(mask)
	}
	pub const fn compute_black_tackle_position_mask(sq:u8) -> Self {
		let index = sq as usize;
		let mask = Self::NEIGHBORS_8[index].storage() & !Self::all_lines_at_south(sq).storage();
		Self::from_storage(mask)
	}
	pub const fn compute_white_tackle_position_mask(sq:u8) -> Self {
		let index = sq as usize;
		let mask = Self::NEIGHBORS_8[index].storage() & !Self::all_lines_at_north(sq).storage();
		Self::from_storage(mask)
	}
	pub const fn generate_black_tackle_position_table() -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_white_tackle_position_mask(i as u8);
			i += 1;
		}
		arr
	}
	pub const fn generate_white_tackle_position_table() -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_black_tackle_position_mask(i as u8);
			i += 1;
		}
		arr
	}

	#[inline(always)]
	pub const fn get_direction(p: u8, ball: u8) -> Option<u8> {
		let w = 9i16;
		
		let p = p as i16;
		let b = ball as i16;

		let pr = p / w; 
		let pc = p % w;
		let br = b / w; 
		let bc = b % w;

		let dr = br - pr; 
		let dc = bc - pc; 

		if dr == 0 && dc > 0 { 
			Some(0) // E
		} else if dr > 0 && dc > 0 && dr == dc { 
			Some(1) // NE
		} else if dr > 0 && dc == 0 { 
			Some(2) // N
		} else if dr > 0 && dc < 0 && dr == -dc { 
			Some(3) // SW
		} else if dr == 0 && dc < 0 { 
			Some(4) // W
		} else if dr < 0 && dc < 0 && dr == dc { 
			Some(5) // SW
		} else if dr < 0 && dc == 0 { 
			Some(6) // S
		} else if dr < 0 && dc > 0 && dr == -dc { 
			Some(7) // SE
		} else { 
			None // no alignement
		}
	}
	pub const SHOOT_MASK: [Self; Self::NB_SQUARES] = Self::generate_pass_mask_table();
	pub const PLAYER1_TACKLE_POS_MASK: [Self; Self::NB_SQUARES] = Self::generate_white_tackle_position_table();
	pub const PLAYER2_TACKLE_POS_MASK: [Self; Self::NB_SQUARES] = Self::generate_black_tackle_position_table();
	pub const BEHIND_GOALS : Self = Bitboard9x13::from_storage(Bitboard9x13::NORTH_BORDER.storage()|Bitboard9x13::SOUTH_BORDER.storage());
	pub const PLAYER1_GOAL : Self = Bitboard9x13::from_storage(0b000111000);
	pub const PLAYER2_GOAL : Self = Bitboard9x13::from_storage(0b000111000 << (9*12));
}
