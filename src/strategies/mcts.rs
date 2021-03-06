use super::*;
use game::*;
use rand::{Rng, XorShiftRng};
use rand;
use std::fmt;
use std::sync::*;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Stats {
    wins: usize,
    losses: usize,
    visits: usize,
}

use std::collections::HashMap;
use std::hash::Hash;
#[derive(Clone)]
pub struct MCTSParams {
    // Time limit in ms.
    pub timeout: u64,
    pub c: f64,
}

struct State<G: Hash + PartialEq + Eq> {
    stats: HashMap<G, Stats>,
    cur: Option<G>,
    rng: XorShiftRng,
}

pub struct MCTS<G: Hash + Eq + RandGame + 'static> {
    params: MCTSParams,
    state: Arc<Mutex<State<G>>>,
}

impl<G: RandGame + Eq + Hash + 'static> MCTS<G> {
    fn simulate(&self, game: &G) -> Option<G::Agent> {
        let acting = game.to_act();
        let nexts = game.possible_moves();
        let winner = if nexts.is_empty() {
            game.winner()
        } else {
            let parent_visits = {
                let stats_cache = &self.state.lock().unwrap().stats;
                stats_cache.get(game).map(|s| s.visits).unwrap_or(1)
            };
            let g = self.select(nexts, parent_visits, acting);
            self.simulate(&g)
        };

        let stats_cache = &mut self.state.lock().unwrap().stats;
        let stats = stats_cache.entry(game.clone()).or_insert(Stats {
            wins: 0,
            losses: 0,
            visits: 0,
        });

        stats.visits += 1;
        if let Some(w) = winner {
            if w == acting {
                stats.wins += 1;
            } else {
                stats.losses += 1;
            }
        }

        winner
    }

    fn key(&self, (g, s): (&G, &Stats), n: f64, acting: G::Agent) -> f64 {
        let wins = if acting == g.to_act() {
            s.wins
        } else {
            s.losses
        } as f64;
        let visits = s.visits as f64;
        wins / visits + self.params.c * (n.ln() / visits).sqrt()
    }

    fn select(&self, choices: Vec<ValidMove<G>>, parent_visits: usize, acting: G::Agent) -> G {
        let n = choices.len();
        let mut state = self.state.lock().unwrap();

        if choices.is_empty() {
            panic!("Shouldn't have gotten here.")
        }

        let i = state.rng.gen::<usize>();
        let random_choice = choices[i % n].clone().apply();
        let games: Vec<_> = choices
            .into_iter()
            .flat_map(|m| {
                let game = m.apply();
                let stats = state.stats.get(&game);
                stats.map(|s| (game, s))
            })
            .collect();

        if games.len() == n {
            let x = games.into_iter().max_by(|t1, t2| {
                let t1 = (&t1.0, t1.1);
                let t2 = (&t2.0, t2.1);
                self.key(t1, parent_visits as f64, acting)
                    .partial_cmp(&self.key(t2, parent_visits as f64, acting))
                    .unwrap()
            });
            x.unwrap().0
        } else {
            random_choice
        }
    }
}

impl<G> Strategy<G> for MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq,
{
    type Params = MCTSParams;

    fn decide(&mut self, game: &G) -> G::Move {
        {
            (*self.state.lock().unwrap()).cur = Some(game.clone());
        }
        thread::sleep(Duration::from_millis(self.params.timeout));
        let state = &self.state.lock().unwrap();

        let nexts = game.possible_moves().into_iter().map(|m| {
            let vm = *m.valid_move();
            (vm, m.apply())
        });

        // let nexts2 = game.possible_moves().into_iter().map(|m| {
        //     let vm = *m.valid_move();
        //     (vm, m.apply())
        // });

        // for (_, g) in nexts2 {
        //     let s = state.stats.get(&g).unwrap();
        //     println!("{:?}\n{}", s, g);
        // }

        let (best, stats) = nexts
            .into_iter()
            .flat_map(|(m, g)| state.stats.get(&g).map(|s| (m, s)))
            .max_by(|t1, t2| {
                let s1 = t1.1;
                let s2 = t2.1;
                s1.visits.cmp(&s2.visits)
                // (s1.losses as f64 / s1.visits as f64)
                //     .partial_cmp(&(s2.losses as f64 / s2.visits as f64))
                //     .unwrap_or(Less)
            })
            .unwrap();

        println!("Best: {:?}", stats);
        best
    }

    fn create(params: MCTSParams) -> Self {
        let seed = rand::random::<[u32; 4]>();
        let rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
        let state = Arc::new(Mutex::new(State {
            cur: None,
            stats: HashMap::new(),
            rng: rng,
        }));
        let new = Self {
            params: params.clone(),
            state: state.clone(),
        };
        let it = Self {
            params: params,
            state: state,
        };
        thread::spawn(move || loop {
            let game = {
                match it.state.lock().unwrap().cur.clone() {
                    None => continue,
                    Some(g) => g,
                }
            };

            it.simulate(&game);
        });
        new

    }
}
