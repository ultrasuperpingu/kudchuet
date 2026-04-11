use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::{Bitboard, bitboard_table};


#[bitboard(width=8,height=8,col_major=false)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard8x8;

bitboard_table!(RAY_N, ray_n, ray_n_mask, Bitboard8x8, Bitboard8x8::generate_ray_n_table());
bitboard_table!(RAY_S, ray_s, ray_s_mask, Bitboard8x8, Bitboard8x8::generate_ray_s_table());
bitboard_table!(RAY_W, ray_w, ray_w_mask, Bitboard8x8, Bitboard8x8::generate_ray_w_table());
bitboard_table!(RAY_E, ray_e, ray_e_mask, Bitboard8x8, Bitboard8x8::generate_ray_e_table());
bitboard_table!(RAY_NE, ray_ne, ray_ne_mask, Bitboard8x8, Bitboard8x8::generate_ray_ne_table());
bitboard_table!(RAY_NW, ray_nw, ray_nw_mask, Bitboard8x8, Bitboard8x8::generate_ray_nw_table());
bitboard_table!(RAY_SE, ray_se, ray_se_mask, Bitboard8x8, Bitboard8x8::generate_ray_se_table());
bitboard_table!(RAY_SW, ray_sw, ray_sw_mask, Bitboard8x8, Bitboard8x8::generate_ray_sw_table());
bitboard_table!(DIAG_INC, diag_inc, diag_inc_mask, Bitboard8x8, Bitboard8x8::generate_diag_inc_table());
bitboard_table!(DIAG_DEC, diag_dec, diag_dec_mask, Bitboard8x8, Bitboard8x8::generate_diag_dec_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard8x8, Bitboard8x8::generate_neighbors_8_table());
impl Bitboard8x8 {
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
