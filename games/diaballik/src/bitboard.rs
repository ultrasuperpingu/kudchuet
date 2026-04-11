use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::{Bitboard, bitboard_table};


#[bitboard(width=7,height=7)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard7x7;
bitboard_table!(RAY_N, ray_n, ray_n_mask, Bitboard7x7, Bitboard7x7::generate_ray_n_table());
bitboard_table!(RAY_S, ray_s, ray_s_mask, Bitboard7x7, Bitboard7x7::generate_ray_s_table());
bitboard_table!(RAY_W, ray_w, ray_w_mask, Bitboard7x7, Bitboard7x7::generate_ray_w_table());
bitboard_table!(RAY_E, ray_e, ray_e_mask, Bitboard7x7, Bitboard7x7::generate_ray_e_table());
bitboard_table!(RAY_NE, ray_ne, ray_ne_mask, Bitboard7x7, Bitboard7x7::generate_ray_ne_table());
bitboard_table!(RAY_NW, ray_nw, ray_nw_mask, Bitboard7x7, Bitboard7x7::generate_ray_nw_table());
bitboard_table!(RAY_SE, ray_se, ray_se_mask, Bitboard7x7, Bitboard7x7::generate_ray_se_table());
bitboard_table!(RAY_SW, ray_sw, ray_sw_mask, Bitboard7x7, Bitboard7x7::generate_ray_sw_table());
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard7x7, Bitboard7x7::generate_neighbors_ortho_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard7x7, Bitboard7x7::generate_neighbors_8_table());

impl Bitboard7x7 {
	/// Precomputed masks of all squares between any two squares on the bitboard.
	pub const RAY_BETWEEN_MASKS: [[Self; Self::NB_SQUARES]; Self::NB_SQUARES] = Self::generate_ray_between_table();
	/// Returns the precomputed bitboard mask of squares between `from` and `to`.
	#[inline(always)]
	pub fn ray_between_mask(from: usize, to: usize) -> Self {
		Self::RAY_BETWEEN_MASKS[from][to]
	}
	#[inline(always)]
	pub fn ray_between(&self, from: usize, to: usize) -> Self {
		*self & Self::RAY_BETWEEN_MASKS[from][to]
	}
}

impl Bitboard7x7 {
	const fn compute_pass_mask(sq:u8) -> Self {
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
			arr[i] = Self::compute_pass_mask(i as u8);
			i += 1;
		}
		arr
	}
	pub const PASS_MASK: [Self; Self::NB_SQUARES] = Self::generate_pass_mask_table();
}