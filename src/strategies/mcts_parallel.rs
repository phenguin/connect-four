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
#[derive(Clone, Copy)]
pub struct MCTSParams {
    // Time limit in ms.
    pub timeout: u64,
    pub c: f64,
    pub workers: u64,
}

// struct State<G: Hash + PartialEq + Eq> {
//     stats: HashMap<G, Stats>,
//     cur: Option<G>,
// }

struct MCTSInner<G: Sync + Hash + Eq + RandGame + 'static> {
    params: MCTSParams,
    stats: Arc<Mutex<HashMap<G, Stats>>>,
    cur: Arc<RwLock<Option<G>>>,
}

pub struct MCTS<G: Sync + Hash + Eq + RandGame + 'static> {
    inner: Arc<MCTSInner<G>>,
}

impl<G: RandGame + Eq + Hash + Sync + 'static> MCTSInner<G> {
    fn simulate<R: rand::Rng>(&self, rng: &mut R, game: &G) -> Option<G::Agent> {
        let acting = game.to_act();
        let nexts = game.possible_moves();
        let winner = if nexts.is_empty() {
            game.winner()
        } else {
            let parent_visits = {
                let stats_cache = self.stats.lock().unwrap();
                stats_cache.get(game).map(|s| s.visits).unwrap_or(1)
            };
            let g = self.select(rng, nexts, parent_visits, acting);
            self.simulate(rng, &g)
        };

        let mut stats_cache = &mut self.stats.lock().unwrap();
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

    fn select<R: rand::Rng>(
        &self,
        rng: &mut R,
        choices: Vec<ValidMove<G>>,
        parent_visits: usize,
        acting: G::Agent,
    ) -> G {
        let n = choices.len();
        let mut stats = self.stats.lock().unwrap();

        if choices.is_empty() {
            panic!("Shouldn't have gotten here.")
        }

        let i = rng.gen::<usize>();
        let random_choice = choices[i % n].clone().apply();
        let games: Vec<_> = choices
            .into_iter()
            .flat_map(|m| {
                let game = m.apply();
                let stats = stats.get(&game);
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

impl<G> MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq + Sync + Send,
{
    fn start_worker(&self) {
        let mcts = self.inner.clone();
        thread::spawn(move || {
            let seed = rand::random::<[u32; 4]>();
            let mut rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
            loop {
                let game = {
                    match mcts.cur.read().unwrap().clone() {
                        None => continue,
                        Some(g) => g,
                    }
                };
                mcts.simulate(&mut rng, &game);
            }
        });
    }
}

impl<G: Sync> Strategy<G> for MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq,
{
    type Params = MCTSParams;

    fn decide(&mut self, game: &G) -> G::Move {
        {
            *(self.inner.cur.write().unwrap()) = Some(game.clone());
        }
        thread::sleep(Duration::from_millis(self.inner.params.timeout));
        let stats = &self.inner.stats.lock().unwrap();

        let nexts = game.possible_moves().into_iter().map(|m| {
            let vm = *m.valid_move();
            (vm, m.apply())
        });

        let nexts2 = game.possible_moves().into_iter().map(|m| {
            let vm = *m.valid_move();
            (vm, m.apply())
        });

        for (_, g) in nexts2 {
            let s = stats.get(&g).unwrap();
            println!("{:?}\n{}", s, g);
        }

        let (best, stats) = nexts
            .into_iter()
            .flat_map(|(m, g)| stats.get(&g).map(|s| (m, s)))
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
        let inner = Arc::new(MCTSInner {
            cur: Arc::new(RwLock::new(None)),
            stats: Arc::new(Mutex::new(HashMap::new())),
            params: params,
        });
        let new = MCTS { inner: inner };
        for _ in 0..params.workers {
            new.start_worker();
        }
        new
    }
}