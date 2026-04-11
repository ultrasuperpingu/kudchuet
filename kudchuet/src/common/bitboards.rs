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

#[bitboard(width=5,height=5)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard5x5;
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_ortho_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_8_table());


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

#[bitboard(width=6,height=5)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard6x5;



#[bitboard(width=19, height=19)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Goban;
impl Goban {
	//const ODD : Self = Self::any(&self)
}
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard6x5, Bitboard6x5::generate_neighbors_ortho_table());

#[cfg(test)]
mod tests {
	use super::*;
		#[test]
	fn test_goban() {
		let g=Goban::default();
		println!("{}", g);
		//g.intersects(other)
	}
	#[test]
	fn test_neightbors_ortho() {
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(0));
		assert!(Bitboard5x5::neighbors_ortho_mask(0).count() == 2);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(1));
		assert!(Bitboard5x5::neighbors_ortho_mask(1).count() == 3);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(8));
		assert!(Bitboard5x5::neighbors_ortho_mask(8).count() == 4);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(12));
		assert!(Bitboard5x5::neighbors_ortho_mask(12).count() == 4);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(15));
		assert!(Bitboard5x5::neighbors_ortho_mask(15).count() == 3);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(19));
		assert!(Bitboard5x5::neighbors_ortho_mask(19).count() == 3);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(20));
		assert!(Bitboard5x5::neighbors_ortho_mask(20).count() == 2);
		//println!("{}", Bitboard5x5::neighbors_ortho_mask(24));
		assert!(Bitboard5x5::neighbors_ortho_mask(24).count() == 2);
	}
	#[test]
	fn test_ray_between_basic_cases() {
		// Même rang : e2 -> h2
		let e2 = 12;
		let h2 = 15;
		let mask = Bitboard8x8::compute_ray_between_mask(e2, h2);
		assert_eq!(mask, 
			Bitboard8x8::from_coords(5, 1) |
			Bitboard8x8::from_coords(6, 1)
		);

		// Même colonne : a1 -> a8
		let a1 = 0;
		let a8 = 56;
		let mask = Bitboard8x8::compute_ray_between_mask(a1, a8);
		assert_eq!(mask,
			Bitboard8x8::from_coords(0, 1) |
			Bitboard8x8::from_coords(0, 2) |
			Bitboard8x8::from_coords(0, 3) |
			Bitboard8x8::from_coords(0, 4) |
			Bitboard8x8::from_coords(0, 5) |
			Bitboard8x8::from_coords(0, 6)
		);

		// Diagonale : c1 -> h6
		let c1 = 2;
		let h6 = 47;
		let mask = Bitboard8x8::compute_ray_between_mask(c1, h6);
		assert_eq!(mask,
			Bitboard8x8::from_coords(3, 1) |
			Bitboard8x8::from_coords(4, 2) |
			Bitboard8x8::from_coords(5, 3) |
			Bitboard8x8::from_coords(6, 4)
		);

		// Anti-diagonale : f1 -> a6
		let f1 = 5;
		let a6 = 40;
		let mask = Bitboard8x8::compute_ray_between_mask(f1, a6);
		assert_eq!(mask,
			Bitboard8x8::from_coords(4, 1) |
			Bitboard8x8::from_coords(3, 2) |
			Bitboard8x8::from_coords(2, 3) |
			Bitboard8x8::from_coords(1, 4)
		);
	}

	#[test]
	fn test_ray_between_invalid_cases() {
		// Pas alignés : e2 -> f5
		let e2 = 12;
		let f5 = 37;
		let mask = Bitboard8x8::compute_ray_between_mask(e2, f5);
		assert_eq!(mask, Bitboard8x8::empty());

		// Pas alignés : a1 -> c2 (cavalier)
		let a1 = 0;
		let c2 = 17;
		let mask = Bitboard8x8::compute_ray_between_mask(a1, c2);
		assert_eq!(mask, Bitboard8x8::empty());
	}

	#[test]
	fn test_ray_between_same_square() {
		for sq in 0..64 {
			assert_eq!(
				Bitboard8x8::compute_ray_between_mask(sq, sq),
				Bitboard8x8::empty()
			);
		}
	}

	#[test]
	fn test_ray_between_table_consistency() {
		for from in 0..64 {
			for to in 0..64 {
				assert_eq!(
					Bitboard8x8::RAY_BETWEEN_MASKS[from][to],
					Bitboard8x8::compute_ray_between_mask(from, to)
				);
			}
		}
	}
}
