#![allow(unused)]
use super::*;
use std::ops::{Deref, DerefMut};
use futures::prelude::*;
use futures::future;
use futures_timer::Delay;
use std::hash::Hash;
use game::*;
use std::mem::size_of;
use rand::XorShiftRng;
use rand;
use std::fmt;
use std::sync::*;
use std::thread;
use std::time::{Instant, Duration};
use rayon;
use rayon::prelude::*;
use rayon_futures::{ScopeFutureExt, RayonFuture};
use std::collections::LinkedList;
use std::ops::Add;
use evmap;

impl Add for Stats {
    type Output = Stats;
    fn add(self, rhs: Self) -> Self::Output {
        Stats {
            wins: self.wins + rhs.wins,
            losses: self.losses + rhs.losses,
            visits: self.visits + rhs.visits,
        }
    }
}

use std::iter;
impl<'a> iter::Sum<&'a Stats> for Stats {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Stats::ZERO, |a, b| a + *b)
    }
}

/// Perform a generic `par_extend` by collecting to a `LinkedList<Vec<_>>` in
/// parallel, then extending the collection sequentially.
fn extend<G, I>(mut collection: &mut StatsWriter<G>, par_iter: I)
where
    I: IntoParallelIterator<Item = (G, Stats)>,
    G: Send + Eq + Hash + Clone
{
    let list = par_iter
        .into_par_iter()
        .fold(Vec::new, |mut vec, elem| {
            vec.push(elem);
            vec
        })
        .map(|vec| {
            let mut list = LinkedList::new();
            list.push_back(vec);
            list
        })
        .reduce(LinkedList::new, |mut list1, mut list2| {
            list1.append(&mut list2);
            list1
        });

    for vec in list {
        // println!("vec size: {}", vec.len());
        collection.extend(vec);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct Stats {
    wins: usize,
    losses: usize,
    visits: usize,
}

impl Stats {
    const WIN: Stats = Stats {
        wins: 1,
        losses: 0,
        visits: 1,
    };
    const LOSS: Stats = Stats {
        wins: 0,
        losses: 1,
        visits: 1,
    };
    const TIE: Stats = Stats {
        wins: 0,
        losses: 0,
        visits: 1,
    };
    const ZERO: Stats = Stats {
        wins: 0,
        losses: 0,
        visits: 0,
    };

    fn delta(maybe_won: Option<bool>) -> Self {
        match maybe_won {
            None => Self::TIE,
            Some(won) => if won { Self::WIN } else { Self::LOSS },
        }
    }
}

impl evmap::ShallowCopy for Stats {
    unsafe fn shallow_copy(&mut self) -> Self {
        *self
    }
}

#[derive(Clone)]
struct MCTSState<G: Send + Eq + Hash + Clone> {
    stats: evmap::ReadHandle<G, Stats>,
    current: Arc<Mutex<Option<G>>>,
}


impl<G: Game + Send + Eq + Hash + Clone> MCTSState<G> {
    fn stats(&self, game: &G) -> Option<Stats> {
        self.stats.get_and(game, |vs| vs[0])
    }
    fn simulate<R: rand::Rng>(
        &self,
        params: &MCTSParams,
        rng: &mut R,
        game: &G,
        stats_output: &mut Vec<(G, Stats)>,
    ) -> Option<G::Agent> {
        let acting = game.to_act();
        let nexts = game.possible_moves();
        let winner = if nexts.is_empty() {
            game.winner()
        } else {
            let parent_visits = {
                self.stats(&game).map(|s| s.visits).unwrap_or(1)
            };
            let g = self.select(params, rng, nexts, parent_visits, acting);
            self.simulate(params, rng, &g, stats_output)
        };

        let stats_delta: Stats = Stats::delta(winner.map(|w| w == acting));
        stats_output.push((game.clone(), stats_delta));

        winner
    }

    fn select<R: rand::Rng>(
        &self,
        params: &MCTSParams,
        rng: &mut R,
        choices: Vec<ValidMove<G>>,
        parent_visits: usize,
        acting: G::Agent,
    ) -> G {
        let n = choices.len();

        if choices.is_empty() {
            panic!("Shouldn't have gotten here.")
        }
        let i: usize = rng.gen();
        let random_choice = choices[i % n].clone().apply();
        let games: Vec<_> = choices
            .into_iter()
            .flat_map(|m| {
                let game = m.apply();
                let stats = self.stats(&game);
                stats.map(|s| (game, s))
            })
            .collect();

        if games.len() == n {
            let x = games.into_iter().max_by(|t1, t2| {
                let t1 = (&t1.0, t1.1);
                let t2 = (&t2.0, t2.1);
                params
                    .key(t1, parent_visits as f64, acting)
                    .partial_cmp(&params.key(t2, parent_visits as f64, acting))
                    .expect("2")
            });
            x.expect("1").0
        } else {
            random_choice
        }
    }
}

struct StatsWriter<G: Send + Eq + Hash + Clone> {
    stats: evmap::WriteHandle<G, Stats>,
}

impl<G: Send + Eq + Hash + Clone> StatsWriter<G> {
    fn compact(&mut self) {
        let read_handle: evmap::ReadHandle<G, Stats> = self.clone();
        read_handle.for_each(|k, vs: &[Stats]| if vs.len() > 1 {
            self.update(k.clone(), vs.iter().sum());
        });
        self.refresh();
        println!("Compaction complete.");
    }
}

impl<G: Send + Eq + Hash + Clone> ParallelExtend<(G, Stats)> for StatsWriter<G> {
    fn par_extend<I: IntoParallelIterator<Item = (G, Stats)>>(&mut self, par_iter: I) {
        extend(self, par_iter);
    }
}

impl<G: Send + Eq + Hash + Clone> iter::Extend<(G, Stats)> for StatsWriter<G> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (G, Stats)>,
    {
        use std::collections::HashMap;
        let mut local_map: HashMap<G, Stats> = HashMap::new();
        iter.into_iter().for_each(|(g, s)| {
            let stat = local_map.entry(g.clone())
                .or_insert_with(|| self.stats.get_and(&g, |vs| vs[0]).unwrap_or(Stats::ZERO));
            *stat = *stat + s;
            self.stats.update(g, *stat);
        });
        // println!("Refreshing.");
        self.stats.refresh();
        // println!("Refreshed.");
    }
}

impl<G: Send + Eq + Hash + Clone> Deref for StatsWriter<G> {
    type Target = evmap::WriteHandle<G, Stats>;
    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}

impl<G: Send + Eq + Hash + Clone> DerefMut for StatsWriter<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stats
    }
}



#[derive(Clone, Copy)]
pub struct MCTSParams {
    // Time limit in ms.
    pub timeout: u64,
    pub c: f64,
    pub workers: u64,
    pub worker_batch_size: u64,
    pub merger_queue_bound: usize,
    pub merger_batch_size: u64,
    pub min_flush_interval: u64,
}

impl MCTSParams {
    fn key<G: Game>(&self, (g, s): (&G, Stats), n: f64, acting: G::Agent) -> f64 {
        let wins = if acting == g.to_act() {
            s.wins
        } else {
            s.losses
        } as f64;
        let visits = s.visits as f64;
        wins / visits + self.c * (n.ln() / visits).sqrt()
    }
}

pub struct MCTS<G: Clone + Sync + Hash + Eq + RandGame> {
    params: MCTSParams,
    state: MCTSState<G>,
    notify: Condvar,
    stats_writer: StatsWriter<G>,
}

struct WrapGen<T>(T);
impl<G: Generator<Return = ()>> Iterator for WrapGen<G> {
    type Item = G::Yield;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.resume() {
            GeneratorState::Complete(_) => None,
            GeneratorState::Yielded(x) => Some(x),
        }
    }
}

use std::ops::{Generator, GeneratorState};
impl<G> MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq + Sync + Send,
{
}

impl<G: Sync> Strategy<G> for MCTS<G>
where
    G: RandGame
        + fmt::Display
        + Hash
        + Eq
        + Clone
        + fmt::Debug
        + 'static,
{
    type Params = MCTSParams;

    fn decide(&mut self, game: &G) -> G::Move {
        println!("Deciding for {}..", game);
        {
            println!("Need lock");
            *(self.state.current.lock().expect("Lock poisoned")) = Some(game.clone());
            println!("Got lock lock");
        }

        let pool = rayon::ThreadPoolBuilder::new()
            .breadth_first()
            .build()
            .unwrap();


        let duration = Duration::from_millis(self.params.timeout);
        let start_time = Instant::now();

        let sims = 120000;
        let mut sims_run = 0;
        let sims_needed = 240000;
        while sims_run < sims_needed {
            self.schedule_work(&pool, sims);
            sims_run += sims;
        }

        println!(
            "Finished {:?} sims.  {:?} has elapsed.",  
            sims_run,
            start_time.elapsed(),
        );

        let nexts = game.possible_moves().into_iter().map(|m| {
            let vm = *m.valid_move();
            (vm, m.apply())
        });

        // let nexts2 = game.possible_moves().into_iter().map(|m| {
        //     let vm = *m.valid_move();
        //     (vm, m.apply())
        // });

        // for (_, g) in nexts2 {
        //     let s = stats.get(&g).unwrap();
        //     println!("{:?}\n{}", s, g);
        // }

        let (best, stats) = nexts
            .into_iter()
            .flat_map(|(m, g)| self.state.stats(&g).map(|s| (m, s)))
            .max_by(|t1, t2| {
                let s1 = t1.1;
                let s2 = t2.1;
                s1.visits.cmp(&s2.visits)
                // (s1.losses as f64 / s1.visits as f64)
                //     .partial_cmp(&(s2.losses as f64 / s2.visits as f64))
                //     .unwrap_or(Less)
            })
            .expect("3");

        println!("Best: {:?}", stats);
        best
    }


    fn create(params: MCTSParams) -> Self {
        let seed = rand::random::<[u32; 4]>();
        let mut rng: XorShiftRng = rand::SeedableRng::from_seed(seed);

        let (read_handle, mut write_handle) = evmap::new::<G, Stats>();
        let state = MCTSState {
            stats: read_handle,
            current: Arc::new(Mutex::new(None)),
        };
        let mut stats_writer = StatsWriter { stats: write_handle };

        let seed = rand::random::<[u32; 4]>();
        let mut rng: XorShiftRng = rand::SeedableRng::from_seed(seed);

        let mcts = MCTS {
            params: params,
            state: state,
            notify: Condvar::new(),
            stats_writer: stats_writer,
        };

        mcts
    }
}


impl<G> MCTS<G>
where
    G: Clone + Sync + Hash + Eq + RandGame + fmt::Debug + 'static,
{
    fn schedule_work(&mut self, pool: &rayon::ThreadPool, i: usize) {
        let state = self.state.clone();
        let params = self.params.clone();
        pool.install(|| self.schedule(&pool, state, &params, i));
        println!("Completed {} simulations.", i);
    }

    fn schedule(
        &mut self,
        pool: &rayon::ThreadPool,
        state: MCTSState<G>,
        params: &MCTSParams,
        i: usize,
    ) {
        let current = state.current.lock().expect("Lock poisoned").clone();
        (&mut self.stats_writer).par_extend(
            rayon::iter::repeatn(current.map(|g| (g, state.clone())), i)
                .with_min_len(i / 8)
                .filter_map(|x| x)
                .flat_map(|(game, state)| {
                    let seed = rand::random::<[u32; 4]>();
                    let mut rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
                    let mut stats_output = Vec::new();
                    state.simulate(&params, &mut rng, &game, &mut stats_output);
                    stats_output.into_par_iter()
                }).collect::<Vec<_>>().into_par_iter().with_min_len(10000)
        );
    }
}
