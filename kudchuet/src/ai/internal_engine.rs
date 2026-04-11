use std::pin::Pin;
use std::sync::{Arc, Mutex};
use crate::gui::{BoardGame, BoardMove};
use crate::ai::{AIEngine, AIOptions};
use crate::{ConcreteStrategy};

#[cfg(not(target_arch = "wasm32"))]
use futures::channel::oneshot;
use minimax::strategies::iterative::SearchStopSignal;
use std::future::Future;

pub struct InternalEngine<G, AI>
where
	G: BoardGame,
	G::M: BoardMove<G> + Copy + 'static,
	AI: ConcreteStrategy<G> + 'static,
{
	ai: Arc<Mutex<AI>>,
	stop_signal: Option<SearchStopSignal>,
	phantom: std::marker::PhantomData<G>,
}

impl<G, AI> InternalEngine<G, AI>
where
	G: BoardGame,
	G::M: BoardMove<G> + Copy + 'static,
	AI: ConcreteStrategy<G> + 'static,
{
	pub fn new(ai: AI) -> Self {
		Self {
			ai: Arc::new(Mutex::new(ai)),
			stop_signal: None,
			phantom: std::marker::PhantomData,
		}
	}

	pub fn set_ai(&mut self, ai: AI) {
		self.ai = Arc::new(Mutex::new(ai));
	}
}

impl<G, AI> AIEngine<G> for InternalEngine<G, AI>
where
	G: BoardGame + Clone + Send + Sync+ 'static,
	G::M: BoardMove<G> + Copy + Send + 'static,
	AI: ConcreteStrategy<G> + Send + 'static,
{
	fn get_options(&self) -> Option<AIOptions> {
		if let Ok(l) = self.ai.try_lock() {
			Some(l.get_options())
		} else {
			None
		}
	}

	fn reset_with_options(&mut self, opts: AIOptions) {
		//eprintln!("reset_with_options: {:?}", opts);
		self.ai.lock().unwrap().reset_with_options(opts);
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
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> 
	{
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
	fn choose_move_async(
		&mut self,
		game: G,
	) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
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
	fn set_depth_or_timeout(&mut self, depth:u8, timeout: std::time::Duration) {
		self.ai.lock().unwrap().set_depth_or_timeout(depth, timeout);
	}
}
