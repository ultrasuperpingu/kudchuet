use bitboard_proc_macro::bitboard;
use bitboard::Bitboard;

use crate::abalone::rules::{Hex, idx};


#[bitboard(width=8, height=8)]
#[derive(Debug, Hash)]
pub struct BitboardAbalone;

impl std::fmt::Display for BitboardAbalone {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for r in -4..=4i8 {
			let indent = r.abs() as usize;
			write!(f, "{:width$}", "", width = indent)?;

			for q in -4..=4 {
				let s = -q - r;
				if s < -4 || s > 4 {
					continue;
				}

				let hex = Hex { q, r };
				
				let char_to_print = if let Some(index) = idx(hex) {
					let bit = 1u64 << index;
					if (self.storage() & bit) != 0 {
						'X'
					} else {
						'.'
					}
				} else {
					' '
				};

				write!(f, "{} ", char_to_print)?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}