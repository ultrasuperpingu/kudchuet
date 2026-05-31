use egui::include_image;

use crate::playing_cards32::PlayingCard32;

impl PlayingCard32 {
	pub fn card_texture(&self) -> egui::ImageSource {
		match self {
			PlayingCard32::SevenOfSpades =>   include_image!("../../cards/07_of_spades.svg"),
			PlayingCard32::EightOfSpades =>   include_image!("../../cards/08_of_spades.svg"),
			PlayingCard32::NineOfSpades =>    include_image!("../../cards/09_of_spades.svg"),
			PlayingCard32::TenOfSpades =>     include_image!("../../cards/10_of_spades.svg"),
			PlayingCard32::JackOfSpades =>    include_image!("../../cards/Jack_of_spades_fr.svg"),
			PlayingCard32::QueenOfSpades =>   include_image!("../../cards/Queen_of_spades_fr.svg"),
			PlayingCard32::KingOfSpades =>    include_image!("../../cards/King_of_spades_fr.svg"),
			PlayingCard32::AceOfSpades =>     include_image!("../../cards/01_of_spades_01.svg"),
			PlayingCard32::SevenOfHearts =>   include_image!("../../cards/07_of_hearts.svg"),
			PlayingCard32::EightOfHearts =>   include_image!("../../cards/08_of_hearts.svg"),
			PlayingCard32::NineOfHearts =>    include_image!("../../cards/09_of_hearts.svg"),
			PlayingCard32::TenOfHearts =>     include_image!("../../cards/10_of_hearts.svg"),
			PlayingCard32::JackOfHearts =>    include_image!("../../cards/Jack_of_hearts_fr.svg"),
			PlayingCard32::QueenOfHearts =>   include_image!("../../cards/Queen_of_hearts_fr.svg"),
			PlayingCard32::KingOfHearts =>    include_image!("../../cards/King_of_hearts_fr.svg"),
			PlayingCard32::AceOfHearts =>     include_image!("../../cards/01_of_hearts_01.svg"),
			PlayingCard32::SevenOfDiamonds => include_image!("../../cards/07_of_diamonds.svg"),
			PlayingCard32::EightOfDiamonds => include_image!("../../cards/08_of_diamonds.svg"),
			PlayingCard32::NineOfDiamonds =>  include_image!("../../cards/09_of_diamonds.svg"),
			PlayingCard32::TenOfDiamonds =>   include_image!("../../cards/10_of_diamonds.svg"),
			PlayingCard32::JackOfDiamonds =>  include_image!("../../cards/Jack_of_diamonds_fr.svg"),
			PlayingCard32::QueenOfDiamonds => include_image!("../../cards/Queen_of_diamonds_fr.svg"),
			PlayingCard32::KingOfDiamonds =>  include_image!("../../cards/King_of_diamonds_fr.svg"),
			PlayingCard32::AceOfDiamonds =>   include_image!("../../cards/01_of_diamonds_01.svg"),
			PlayingCard32::SevenOfClubs =>    include_image!("../../cards/07_of_clubs.svg"),
			PlayingCard32::EightOfClubs =>    include_image!("../../cards/08_of_clubs.svg"),
			PlayingCard32::NineOfClubs =>     include_image!("../../cards/09_of_clubs.svg"),
			PlayingCard32::TenOfClubs =>      include_image!("../../cards/10_of_clubs.svg"),
			PlayingCard32::JackOfClubs =>     include_image!("../../cards/Jack_of_clubs_fr.svg"),
			PlayingCard32::QueenOfClubs =>    include_image!("../../cards/Queen_of_clubs_fr.svg"),
			PlayingCard32::KingOfClubs =>     include_image!("../../cards/King_of_clubs_fr.svg"),
			PlayingCard32::AceOfClubs =>      include_image!("../../cards/01_of_clubs_01.svg"),
		}
	}
}