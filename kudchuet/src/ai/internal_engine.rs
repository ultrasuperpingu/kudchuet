use crate::StrategyWithOptions;
use crate::ai::AIEngine;
use crate::ai::uci::UciValue;
use crate::gui::{BoardGame, BoardMove};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::ai::move_search::SearchStopSignal;
#[cfg(not(target_arch = "wasm32"))]
use futures::channel::oneshot;
use std::future::Future;

pub struct InternalEngine<G, AI>
where
	G: BoardGame,
	G::M: BoardMove<G> + Copy + 'static,
	AI: StrategyWithOptions<G> + 'static,
{
	ai: Arc<Mutex<AI>>,
	opts: HashMap<String, UciValue>,
	stop_signal: Option<SearchStopSignal>,
	phantom: std::marker::PhantomData<G>,
}

impl<G, AI> InternalEngine<G, AI>
where
	G: BoardGame,
	G::M: BoardMove<G> + Copy + 'static,
	AI: StrategyWithOptions<G> + 'static,
{
	pub fn new(ai: AI) -> Self {
		let opts = ai.get_options();
		Self {
			ai: Arc::new(Mutex::new(ai)),
			opts,
			stop_signal: None,
			phantom: std::marker::PhantomData,
		}
	}

	pub fn set_ai(&mut self, ai: AI) {
		self.opts = ai.get_options();
		self.ai = Arc::new(Mutex::new(ai));
	}
}

impl<G, AI> AIEngine<G> for InternalEngine<G, AI>
where
	G: BoardGame + Clone + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + 'static,
	AI: StrategyWithOptions<G> + Send + 'static,
{
	fn get_options(&self) -> Option<&HashMap<String, UciValue>> {
		//if let Ok(l) = self.ai.try_lock() {
			Some(&self.opts)
		//} else {
		//	None
		//}
	}
	fn get_options_mut(&mut self) -> Option<&mut HashMap<String, UciValue>> {
		//if let Ok(l) = self.ai.try_lock() {
			Some(&mut self.opts)
		//} else {
		//	None
		//}
	}

	fn set_options(&mut self, opts: HashMap<String, UciValue>) {
		//eprintln!("reset_with_options: {:?}", opts);
		self.opts = opts;
		let mut ai = self.ai.lock().unwrap();
		ai.set_options(&self.opts);
		let depth = if let UciValue::Spin(depth, _, _) = self.opts["Depth"] {
			depth as u8
		} else {
			5
		};
		let timeout = if let UciValue::Spin(timeout, _, _) = self.opts["Timeout"] {
			timeout as f32 / 1000.0
		} else {
			0.0
		};
		ai.set_depth_or_timeout(depth, Duration::from_secs_f32(timeout));
	}

	fn set_position(&self, _game: &G) {
		// Nothing to do, position is sent to choose_move
	}

	fn choose_move(&self, game: &G) -> Option<G::M> {
		let mut ai = self.ai.lock().unwrap();
		let mv = ai.choose_move(game);

		if let Some(m) = mv {
			eprintln!("bestmove: {:?} {:?}", m, ai.principal_variation().last());
		}
		mv
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
		let (tx, rx) = oneshot::channel();
		self.stop_signal = Some(self.ai.lock().unwrap().stop_signal());
		let ai = self.ai.clone(); // Arc<Mutex<AI>>
		//println!("choosing move (spawn thread)");
		std::thread::spawn(move || {
			let mv = ai.lock().unwrap().choose_move(&game);
			let _ = tx.send(mv);
		});

		Box::pin(async move { rx.await.ok().flatten() })
	}
	#[cfg(target_arch = "wasm32")]
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
		let ai = self.ai.clone();

		Box::pin(async move {
			let mut ai = ai.lock().unwrap();
			ai.choose_move(&game)
		})
	}
	fn stop_thinking(&self) {
		println!("getting stop signal");
		if let Some(ss) = &self.stop_signal {
			println!("launching stop signal");
			ss.stop_search();
		}
	}
	fn set_depth_or_timeout(&mut self, depth: u8, timeout: std::time::Duration) {
		self.ai.lock().unwrap().set_depth_or_timeout(depth, timeout);
	}
}
