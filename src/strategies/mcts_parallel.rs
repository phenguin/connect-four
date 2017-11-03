use super::*;
use game::*;
use rand::XorShiftRng;
use rand;
use std::fmt;
use std::sync::*;
use std::thread;
use std::time::{Instant, Duration};

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
    pub worker_batch_size: u64,
    pub merger_queue_bound: usize,
    pub merger_batch_size: u64,
    pub min_flush_interval: u64,
}

// struct State<G: Hash + PartialEq + Eq> {
//     stats: HashMap<G, Stats>,
//     cur: Option<G>,
// }

enum WorkerMessage<G: Send> {
    UpdateStats(HashMap<G, Stats>),
    UpdateCur(Option<G>),
}

struct MCTSWorker<G: Sync + Hash + Eq + RandGame + 'static> {
    input: mpsc::Receiver<WorkerMessage<G>>,
    last_flush: Instant,
    merger: mpsc::SyncSender<MergerMessage<G>>,
    params: MCTSParams,
    updates: HashMap<G, Stats>,
    stats_cache: HashMap<G, Stats>,
    cur: Option<G>,
}

enum MergerMessage<G> {
    GetStats(mpsc::Sender<HashMap<G, Stats>>),
    Merge(HashMap<G, Stats>),
}

struct MCTSMerger<G: Send> {
    input: mpsc::Receiver<MergerMessage<G>>,
    priority_input: mpsc::Receiver<MergerMessage<G>>,
    worker_outputs: Vec<mpsc::Sender<WorkerMessage<G>>>,
    params: MCTSParams,
    stats: HashMap<G, Stats>,
}

impl<G: 'static + Send + Clone + Eq + Hash> MCTSMerger<G> {
    fn new(
        params: MCTSParams,
        outputs: Vec<mpsc::Sender<WorkerMessage<G>>>,
        input: mpsc::Receiver<MergerMessage<G>>,
        priority_input: mpsc::Receiver<MergerMessage<G>>,
    ) -> Self {
        let new = MCTSMerger {
            params: params,
            input: input,
            priority_input: priority_input,
            stats: HashMap::new(),
            worker_outputs: outputs,
        };
        new
    }

    fn start(mut self) {
        thread::spawn(move || loop {
            for i in 0..self.params.merger_batch_size {
                if i % 100 == 0 {
                    if let Ok(msg) = self.priority_input.try_recv() {
                        self.handle(msg);
                    }
                }
                let msg = self.input.recv().expect("Recv failed.");
                self.handle(msg);
            }

            for tx in &self.worker_outputs {
                tx.send(WorkerMessage::UpdateStats(self.stats.clone()))
                    .expect("UpdateStats failed.");
            }
        });
    }

    fn handle(&mut self, msg: MergerMessage<G>) {
        use self::MergerMessage::*;
        match msg {
            GetStats(tx) => tx.send(self.stats.clone()).expect("GetStats failed."),
            Merge(updates) => {
                for (k, v) in updates.into_iter() {
                    let mut stats = self.stats.entry(k).or_insert(Stats {
                        visits: 0,
                        wins: 0,
                        losses: 0,
                    });

                    stats.visits += v.visits;
                    stats.wins += v.wins;
                    stats.losses += v.losses;
                }
            }
        }
    }
}

pub struct MCTS<G: Sync + Hash + Eq + RandGame + 'static> {
    params: MCTSParams,
    workers: Vec<mpsc::Sender<WorkerMessage<G>>>,
    merger: mpsc::Sender<MergerMessage<G>>,
    // merger: mpsc::Sender
}

impl<G: RandGame + Eq + Hash + Sync + 'static> MCTSWorker<G> {
    fn simulate<R: rand::Rng>(&mut self, rng: &mut R, game: &G) -> Option<G::Agent> {
        let acting = game.to_act();
        let nexts = game.possible_moves();
        let winner = if nexts.is_empty() {
            game.winner()
        } else {
            let parent_visits = {
                self.stats_cache.get(game).map(|s| s.visits).unwrap_or(1)
            };
            let g = self.select(rng, nexts, parent_visits, acting);
            self.simulate(rng, &g)
        };

        let stats = self.stats_cache.entry(game.clone()).or_insert(Stats {
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

        let stats = self.updates.entry(game.clone()).or_insert(Stats {
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

        if choices.is_empty() {
            panic!("Shouldn't have gotten here.")
        }

        let i = rng.gen::<usize>();
        let random_choice = choices[i % n].clone().apply();
        let games: Vec<_> = choices
            .into_iter()
            .flat_map(|m| {
                let game = m.apply();
                let stats = self.stats_cache.get(&game);
                stats.map(|s| (game, s))
            })
            .collect();

        if games.len() == n {
            let x = games.into_iter().max_by(|t1, t2| {
                let t1 = (&t1.0, t1.1);
                let t2 = (&t2.0, t2.1);
                self.key(t1, parent_visits as f64, acting)
                    .partial_cmp(&self.key(t2, parent_visits as f64, acting))
                    .expect("2")
            });
            x.expect("1").0
        } else {
            random_choice
        }
    }

    fn handle(&mut self, msg: WorkerMessage<G>) {
        use self::WorkerMessage::*;
        match msg {
            UpdateStats(stats) => {
                // self.flush_updates();
                self.stats_cache = stats;
            }
            UpdateCur(cur) => self.cur = cur,
        }
    }

    fn maybe_flush_updates(&mut self) {
        use std::mem;
        let now = Instant::now();
        if now.duration_since(self.last_flush) >
            Duration::from_millis(self.params.min_flush_interval)
        {
            self.merger
                .send(MergerMessage::Merge(
                    mem::replace(&mut self.updates, HashMap::new()),
                ))
                .expect("Couldn't flush updates.");
            self.last_flush = now;
        } else {
            println!("Not flushing too soon.");
        }
    }

    fn start(mut self) {
        thread::spawn(move || {
            let seed = rand::random::<[u32; 4]>();
            let mut rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
            loop {
                for _ in 0..self.params.worker_batch_size {
                    if let Ok(msg) = self.input.try_recv() {
                        self.handle(msg);
                    }

                    let game = {
                        match self.cur {
                            None => continue,
                            Some(ref g) => g.clone(),
                        }
                    };
                    self.simulate(&mut rng, &game);
                }

                self.maybe_flush_updates();
            }
        });
    }
}

impl<G> MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq + Sync + Send,
{
}

impl<G: Sync> Strategy<G> for MCTS<G>
where
    G: RandGame + fmt::Display + Hash + Eq,
{
    type Params = MCTSParams;

    fn decide(&mut self, game: &G) -> G::Move {
        for tx in &self.workers {
            tx.send(WorkerMessage::UpdateCur(Some(game.clone())))
                .expect("UpdateCur failed.");
        }
        thread::sleep(Duration::from_millis(self.params.timeout));
        let (tx, rx) = mpsc::channel();
        self.merger.send(MergerMessage::GetStats(tx)).expect(
            "GetStats request didn't send",
        );
        let stats = rx.recv().expect("Couldn't get stats.");

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
            .flat_map(|(m, g)| stats.get(&g).map(|s| (m, s)))
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

        let mut workers = Vec::new();

        let (merger_tx, merger_rx) = mpsc::sync_channel(params.merger_queue_bound);
        for _ in 0..params.workers {
            let (tx, rx) = mpsc::channel();
            let worker = MCTSWorker {
                input: rx,
                last_flush: Instant::now(),
                merger: merger_tx.clone(),
                cur: None,
                stats_cache: HashMap::new(),
                updates: HashMap::new(),
                params: params.clone(),
            };
            workers.push(tx);
            worker.start();
        }

        let (priority_merger_tx, priority_merger_rx) = mpsc::channel();
        let merger = MCTSMerger::new(
            params.clone(),
            workers.clone(),
            merger_rx,
            priority_merger_rx,
        );
        merger.start();
        let new = MCTS {
            params: params,
            workers: workers,
            merger: priority_merger_tx,
        };

        new
    }
}
