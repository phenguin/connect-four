extern crate gameai;

use gameai::game::dots::*;
use gameai::runner::*;
use gameai::strategies::mcts::*;

fn main() {
    let mut human1 = HumanPlayer::new("Justin");
    let mut human2 = AIPlayer::<Dots, MCTS<Dots>>::new(
        "Robot",
        MCTSParams {
            timeout: 10000,
            c: (2.0 as f64).sqrt(),
        },
    );
    Runner::run(&mut human1, &mut human2);
}
