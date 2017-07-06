extern crate gameai;

use gameai::game::dots::*;
use gameai::runner::*;

fn main() {
    let mut human1 = HumanPlayer::new("Justin");
    let mut human2 = HumanPlayer::new("Justin2");
    Runner::<Dots>::run(&mut human1, &mut human2);
}
