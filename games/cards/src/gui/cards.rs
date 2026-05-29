use egui::include_image;

use crate::playing_cards32::PlayingCard32;

impl PlayingCard32 {
	pub fn card_texture(&self) -> egui::Image {
		match self {
			PlayingCard32::SevenOfSpades =>   egui::Image::new(include_image!("../../cards/07_of_spades.svg")),
			PlayingCard32::EightOfSpades =>   egui::Image::new(include_image!("../../cards/08_of_spades.svg")),
			PlayingCard32::NineOfSpades =>    egui::Image::new(include_image!("../../cards/09_of_spades.svg")),
			PlayingCard32::TenOfSpades =>     egui::Image::new(include_image!("../../cards/10_of_spades.svg")),
			PlayingCard32::JackOfSpades =>    egui::Image::new(include_image!("../../cards/Jack_of_spades_fr.svg")),
			PlayingCard32::QueenOfSpades =>   egui::Image::new(include_image!("../../cards/Queen_of_spades_fr.svg")),
			PlayingCard32::KingOfSpades =>    egui::Image::new(include_image!("../../cards/King_of_spades_fr.svg")),
			PlayingCard32::AceOfSpades =>     egui::Image::new(include_image!("../../cards/01_of_spades_01.svg")),
			PlayingCard32::SevenOfHearts =>   egui::Image::new(include_image!("../../cards/07_of_hearts.svg")),
			PlayingCard32::EightOfHearts =>   egui::Image::new(include_image!("../../cards/08_of_hearts.svg")),
			PlayingCard32::NineOfHearts =>    egui::Image::new(include_image!("../../cards/09_of_hearts.svg")),
			PlayingCard32::TenOfHearts =>     egui::Image::new(include_image!("../../cards/10_of_hearts.svg")),
			PlayingCard32::JackOfHearts =>    egui::Image::new(include_image!("../../cards/Jack_of_hearts_fr.svg")),
			PlayingCard32::QueenOfHearts =>   egui::Image::new(include_image!("../../cards/Queen_of_hearts_fr.svg")),
			PlayingCard32::KingOfHearts =>    egui::Image::new(include_image!("../../cards/King_of_hearts_fr.svg")),
			PlayingCard32::AceOfHearts =>     egui::Image::new(include_image!("../../cards/01_of_hearts_01.svg")),
			PlayingCard32::SevenOfDiamonds => egui::Image::new(include_image!("../../cards/07_of_diamonds.svg")),
			PlayingCard32::EightOfDiamonds => egui::Image::new(include_image!("../../cards/08_of_diamonds.svg")),
			PlayingCard32::NineOfDiamonds =>  egui::Image::new(include_image!("../../cards/09_of_diamonds.svg")),
			PlayingCard32::TenOfDiamonds =>   egui::Image::new(include_image!("../../cards/10_of_diamonds.svg")),
			PlayingCard32::JackOfDiamonds =>  egui::Image::new(include_image!("../../cards/Jack_of_diamonds_fr.svg")),
			PlayingCard32::QueenOfDiamonds => egui::Image::new(include_image!("../../cards/Queen_of_diamonds_fr.svg")),
			PlayingCard32::KingOfDiamonds =>  egui::Image::new(include_image!("../../cards/King_of_diamonds_fr.svg")),
			PlayingCard32::AceOfDiamonds =>   egui::Image::new(include_image!("../../cards/01_of_diamonds_01.svg")),
			PlayingCard32::SevenOfClubs =>    egui::Image::new(include_image!("../../cards/07_of_clubs.svg")),
			PlayingCard32::EightOfClubs =>    egui::Image::new(include_image!("../../cards/08_of_clubs.svg")),
			PlayingCard32::NineOfClubs =>     egui::Image::new(include_image!("../../cards/09_of_clubs.svg")),
			PlayingCard32::TenOfClubs =>      egui::Image::new(include_image!("../../cards/10_of_clubs.svg")),
			PlayingCard32::JackOfClubs =>     egui::Image::new(include_image!("../../cards/Jack_of_clubs_fr.svg")),
			PlayingCard32::QueenOfClubs =>    egui::Image::new(include_image!("../../cards/Queen_of_clubs_fr.svg")),
			PlayingCard32::KingOfClubs =>     egui::Image::new(include_image!("../../cards/King_of_clubs_fr.svg")),
			PlayingCard32::AceOfClubs =>      egui::Image::new(include_image!("../../cards/01_of_clubs_01.svg")),
		}
	}
}