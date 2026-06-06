use crate::{
	playing_cards::{CardSet, CardSuit, PlayingCard},
	playing_cards32::PlayingCard32,
	playing_cards54::PlayingCard54,
	playing_cards78::{PlayingCard78, TarotColors},
};

pub struct CardSetIter<C> {
	bits: u128,
	_marker: std::marker::PhantomData<C>,
}
impl<C> Iterator for CardSetIter<C>
where
	C: PlayingCard,
{
	type Item = C;

	fn next(&mut self) -> Option<Self::Item> {
		if self.bits == 0 {
			return None;
		}

		let idx = self.bits.trailing_zeros() as u8;

		self.bits &= self.bits - 1;

		idx.try_into().ok()
	}
}
impl<C> CardSetIter<C> {
	pub fn new(bits: u128) -> Self {
		Self {
			bits,
			_marker: std::marker::PhantomData,
		}
	}
}

pub trait UnorderedCardSet: CardSet + Sized + Copy {
	type Storage;

	fn bits(self) -> Self::Storage;
}
macro_rules! impl_unordered_card_set {
	(
		$set:ty,
		$ident:ident,
		$iter:ty,
		$iter_ident:ident,
		$card:ty,
		$color:ty,
		$storage:ty
	) => {
		#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
		#[repr(transparent)]
		pub struct $ident(pub(crate) $storage);
		impl $set {
			pub const ALL: Self =
				Self(((1u128 << <Self as CardSet>::Card::CARD_COUNT) - 1) as $storage);
			#[inline]
			pub fn iter(self) -> $iter {
				$iter_ident(self)
			}
			#[inline]
			pub const fn contains(self, card: $card) -> bool {
				self.0 & (1 << (card as u8)) != 0
			}
			#[inline]
			pub const fn insert(&mut self, card: $card) {
				self.0 |= 1 << (card as u8);
			}

			#[inline]
			pub const fn remove(&mut self, card: $card) {
				self.0 &= !(1 << (card as u8));
			}
			#[inline]
			pub const fn pop(&mut self) -> Option<$card> {
				if self.0 == 0 {
					return None;
				}

				let idx = (<$storage>::BITS - 1 - self.0.leading_zeros()) as u8;

				self.0 &= !((1 as $storage) << idx);

				<$card>::from_index(idx)
			}
			#[inline]
			pub const fn len(self) -> usize {
				self.0.count_ones() as usize
			}

			#[inline]
			pub const fn is_empty(self) -> bool {
				self.0 == 0
			}

			pub fn of_color(self, color: $color) -> Self {
				self & Self::by_color(color)
			}
		}
		impl CardSet for $set {
			type Card = $card;
			const EMPTY: Self = Self(0);

			#[inline]
			fn contains(&self, card: $card) -> bool {
				self.0 & (1 << (card as u8)) != 0
			}

			#[inline]
			fn insert(&mut self, card: $card) -> bool {
				self.insert(card);
				true
			}

			#[inline]
			fn remove(&mut self, card: $card) -> bool {
				self.remove(card);
				true
			}
			#[inline]
			fn len(&self) -> usize {
				self.0.count_ones() as usize
			}
			#[inline]
			fn iter(&self) -> $iter {
				$iter_ident(*self)
			}
			#[inline]
			fn is_empty(&self) -> bool {
				self.0 == 0
			}
			fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
				let mut remaining = from.len() as u32;

				if remaining < nb as u32 {
					return Err("Not enough cards in the deck".into());
				}

				let mut result = Self::EMPTY;

				for _ in 0..nb {
					let mut idx = fastrand::u32(0..remaining);
					let mut x = from.0;

					let mask = loop {
						let bit = x.trailing_zeros();
						let m = 1 << bit;

						if idx == 0 {
							break m;
						}

						x &= x - 1;
						idx -= 1;
					};

					from.0 ^= mask;
					result.0 |= mask;

					remaining -= 1;
				}

				Ok(result)
			}
			fn all() -> Self {
				Self::ALL
			}
		}
		impl UnorderedCardSet for $set {
			type Storage = $storage;

			fn bits(self) -> Self::Storage {
				self.0
			}
		}
		impl std::ops::BitOrAssign<$card> for $set {
			#[inline]
			fn bitor_assign(&mut self, rhs: $card) {
				self.0 |= 1 << rhs.index();
			}
		}
		impl std::ops::BitOr<$card> for $set {
			type Output = Self;

			fn bitor(self, rhs: $card) -> Self::Output {
				Self(self.0 | (1 << rhs.index()))
			}
		}
		impl std::ops::BitOr for $set {
			type Output = Self;

			fn bitor(self, rhs: Self) -> Self::Output {
				Self(self.0 | rhs.0)
			}
		}
		impl std::ops::BitAndAssign for $set {
			#[inline]
			fn bitand_assign(&mut self, rhs: Self) {
				self.0 &= rhs.0
			}
		}
		impl std::ops::BitAnd for $set {
			type Output = Self;

			#[inline]
			fn bitand(self, rhs: Self) -> Self::Output {
				Self(self.0 & rhs.0)
			}
		}
		impl std::ops::SubAssign for $set {
			#[inline]
			fn sub_assign(&mut self, rhs: Self) {
				self.0 &= !rhs.0
			}
		}
		impl std::ops::Sub for $set {
			type Output = Self;

			#[inline]
			fn sub(self, rhs: Self) -> Self::Output {
				Self(self.0 & !rhs.0)
			}
		}

		impl From<$card> for $set {
			#[inline]
			fn from(card: $card) -> Self {
				Self(1 << card as u8)
			}
		}
		impl Default for $set {
			fn default() -> Self {
				Self::EMPTY
			}
		}
		impl std::fmt::Display for $set {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				use std::fmt::Write;

				f.write_char('[')?;

				let mut first = true;

				for c in *self {
					if !first {
						f.write_str(", ")?;
					}

					first = false;
					write!(f, "{c}")?;
				}

				f.write_char(']')
			}
		}

		impl IntoIterator for $set {
			type Item = $card;
			type IntoIter = $iter;

			fn into_iter(self) -> Self::IntoIter {
				self.iter()
			}
		}
		impl FromIterator<$card> for $set {
			fn from_iter<T: IntoIterator<Item = $card>>(iter: T) -> Self {
				let mut set = Self::EMPTY;

				for card in iter {
					set.insert(card);
				}

				set
			}
		}

		#[derive(Clone, Copy)]
		pub struct $iter_ident($set);
		impl Iterator for $iter {
			type Item = $card;

			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				if self.0.0 != 0 {
					let res = self.0.0.trailing_zeros() as u8;
					self.0.0 &= self.0.0 - 1;
					res.try_into().ok()
				} else {
					None
				}
			}
			fn size_hint(&self) -> (usize, Option<usize>) {
				let count = self.0.len();
				(count, Some(count))
			}
		}
		impl ExactSizeIterator for $iter {}
		impl std::iter::FusedIterator for $iter {}
	};
}

impl_unordered_card_set!(
	UnorderedCardSet32,
	UnorderedCardSet32,
	CardSetIter32,
	CardSetIter32,
	PlayingCard32,
	CardSuit,
	u32
);
impl_unordered_card_set!(
	UnorderedCardSet54,
	UnorderedCardSet54,
	CardSetIter54,
	CardSetIter54,
	PlayingCard54,
	CardSuit,
	u64
);
impl_unordered_card_set!(
	UnorderedCardSet78,
	UnorderedCardSet78,
	CardSetIter78,
	CardSetIter78,
	PlayingCard78,
	TarotColors,
	u128
);

#[macro_export]
macro_rules! unordered_card_set32 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet32(
			0 $(| (1 << (PlayingCard32::$card as u8)))*
		)
	};
}

impl UnorderedCardSet32 {
	pub const fn by_color(color: CardSuit) -> Self {
		match color {
			CardSuit::Spades => unordered_card_set32![
				SevenOfSpades,
				EightOfSpades,
				NineOfSpades,
				TenOfSpades,
				JackOfSpades,
				QueenOfSpades,
				KingOfSpades,
				AceOfSpades,
			],
			CardSuit::Hearts => unordered_card_set32![
				SevenOfHearts,
				EightOfHearts,
				NineOfHearts,
				TenOfHearts,
				JackOfHearts,
				QueenOfHearts,
				KingOfHearts,
				AceOfHearts,
			],
			CardSuit::Diamonds => unordered_card_set32![
				SevenOfDiamonds,
				EightOfDiamonds,
				NineOfDiamonds,
				TenOfDiamonds,
				JackOfDiamonds,
				QueenOfDiamonds,
				KingOfDiamonds,
				AceOfDiamonds,
			],
			CardSuit::Clubs => unordered_card_set32![
				SevenOfClubs,
				EightOfClubs,
				NineOfClubs,
				TenOfClubs,
				JackOfClubs,
				QueenOfClubs,
				KingOfClubs,
				AceOfClubs,
			],
			CardSuit::Joker => unordered_card_set32![],
		}
	}
}

#[macro_export]
macro_rules! unordered_card_set54 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet54(
			0 $(| (1 << (PlayingCard54::$card as u8)))*
		)
	};
}

impl UnorderedCardSet54 {
	pub const ALL_BUT_JOKERS: Self = Self((1 << 52) - 1);
	pub const ALL32: Self = unordered_card_set54![
		AceOfSpades,
		AceOfHearts,
		AceOfDiamonds,
		AceOfClubs,
		SevenOfSpades,
		SevenOfHearts,
		SevenOfDiamonds,
		SevenOfClubs,
		EightOfSpades,
		EightOfHearts,
		EightOfDiamonds,
		EightOfClubs,
		NineOfSpades,
		NineOfHearts,
		NineOfDiamonds,
		NineOfClubs,
		TenOfSpades,
		TenOfHearts,
		TenOfDiamonds,
		TenOfClubs,
		JackOfSpades,
		JackOfHearts,
		JackOfDiamonds,
		JackOfClubs,
		QueenOfSpades,
		QueenOfHearts,
		QueenOfDiamonds,
		QueenOfClubs,
		KingOfSpades,
		KingOfHearts,
		KingOfDiamonds,
		KingOfClubs
	];
}

impl UnorderedCardSet54 {
	pub const fn by_color(color: CardSuit) -> Self {
		match color {
			CardSuit::Spades => unordered_card_set54![
				TwoOfSpades,
				ThreeOfSpades,
				FourOfSpades,
				FiveOfSpades,
				SixOfSpades,
				SevenOfSpades,
				EightOfSpades,
				NineOfSpades,
				TenOfSpades,
				JackOfSpades,
				QueenOfSpades,
				KingOfSpades,
				AceOfSpades,
			],
			CardSuit::Hearts => unordered_card_set54![
				TwoOfHearts,
				ThreeOfHearts,
				FourOfHearts,
				FiveOfHearts,
				SixOfHearts,
				SevenOfHearts,
				EightOfHearts,
				NineOfHearts,
				TenOfHearts,
				JackOfHearts,
				QueenOfHearts,
				KingOfHearts,
				AceOfHearts,
			],
			CardSuit::Diamonds => unordered_card_set54![
				TwoOfDiamonds,
				ThreeOfDiamonds,
				FourOfDiamonds,
				FiveOfDiamonds,
				SixOfDiamonds,
				SevenOfDiamonds,
				EightOfDiamonds,
				NineOfDiamonds,
				TenOfDiamonds,
				JackOfDiamonds,
				QueenOfDiamonds,
				KingOfDiamonds,
				AceOfDiamonds,
			],
			CardSuit::Clubs => unordered_card_set54![
				TwoOfClubs,
				ThreeOfClubs,
				FourOfClubs,
				FiveOfClubs,
				SixOfClubs,
				SevenOfClubs,
				EightOfClubs,
				NineOfClubs,
				TenOfClubs,
				JackOfClubs,
				QueenOfClubs,
				KingOfClubs,
				AceOfClubs,
			],
			CardSuit::Joker => unordered_card_set54![],
		}
	}
}

#[macro_export]
macro_rules! unordered_card_set78 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet78(
			0 $(| (1 << (PlayingCard78::$card as u8)))*
		)
	};
}

impl UnorderedCardSet78 {
	pub const fn by_color(color: TarotColors) -> Self {
		match color {
			TarotColors::Spades => unordered_card_set78![
				TwoOfSpades,
				ThreeOfSpades,
				FourOfSpades,
				FiveOfSpades,
				SixOfSpades,
				SevenOfSpades,
				EightOfSpades,
				NineOfSpades,
				TenOfSpades,
				JackOfSpades,
				KnightOfSpades,
				QueenOfSpades,
				KingOfSpades,
				AceOfSpades,
			],
			TarotColors::Hearts => unordered_card_set78![
				TwoOfHearts,
				ThreeOfHearts,
				FourOfHearts,
				FiveOfHearts,
				SixOfHearts,
				SevenOfHearts,
				EightOfHearts,
				NineOfHearts,
				TenOfHearts,
				JackOfHearts,
				KnightOfHearts,
				QueenOfHearts,
				KingOfHearts,
				AceOfHearts,
			],
			TarotColors::Diamonds => unordered_card_set78![
				TwoOfDiamonds,
				ThreeOfDiamonds,
				FourOfDiamonds,
				FiveOfDiamonds,
				SixOfDiamonds,
				SevenOfDiamonds,
				EightOfDiamonds,
				NineOfDiamonds,
				TenOfDiamonds,
				JackOfDiamonds,
				KnightOfDiamonds,
				QueenOfDiamonds,
				KingOfDiamonds,
				AceOfDiamonds,
			],
			TarotColors::Clubs => unordered_card_set78![
				TwoOfClubs,
				ThreeOfClubs,
				FourOfClubs,
				FiveOfClubs,
				SixOfClubs,
				SevenOfClubs,
				EightOfClubs,
				NineOfClubs,
				TenOfClubs,
				JackOfClubs,
				KnightOfClubs,
				QueenOfClubs,
				KingOfClubs,
				AceOfClubs,
			],
			TarotColors::Trump => unordered_card_set78![
				OneOfTrumps,
				TwoOfTrumps,
				ThreeOfTrumps,
				FourOfTrumps,
				FiveOfTrumps,
				SixOfTrumps,
				SevenOfTrumps,
				EightOfTrumps,
				NineOfTrumps,
				TenOfTrumps,
				ElevenOfTrumps,
				TwelveOfTrumps,
				ThirteenOfTrumps,
				FifteenOfTrumps,
				SeventeenOfTrumps,
				EighteenOfTrumps,
				NineteenOfTrumps,
				TwentyOfTrumps,
				TwentyOneOfTrumps,
			],
			TarotColors::Excuse => unordered_card_set78![Excuse],
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::playing_cards::CardSet;
	use crate::unordered_card_set::{UnorderedCardSet32, UnorderedCardSet54, UnorderedCardSet78};

	#[test]
	fn test_draw_random32() {
		let mut deck = UnorderedCardSet32::ALL;
		let player1 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1 | player2 | player3 | player4,
			UnorderedCardSet32::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
	#[test]
	fn test_draw_random54() {
		let mut deck = UnorderedCardSet54::ALL;
		let player1 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1 | player2 | player3 | player4,
			UnorderedCardSet54::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
	#[test]
	fn test_draw_random78() {
		let mut deck = UnorderedCardSet78::ALL;
		let player1 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 6);
		assert_eq!(player1.len(), 18);
		assert_eq!(player2.len(), 18);
		assert_eq!(player3.len(), 18);
		assert_eq!(player4.len(), 18);
		assert_eq!(
			player1 | player2 | player3 | player4 | deck,
			UnorderedCardSet78::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
		println!("Dog: {}", deck);
	}
}
