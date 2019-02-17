use std::error::Error;
use rand;
use rand::Rng;
use std::fmt;
use std::collections::HashSet;


type Coord = (i32, i32);
type Space = HashSet<Coord>;

struct Config {
    row: usize,
    col: usize,
    density: f32
}

impl Config {
    pub fn new<T>(mut args: T) -> Result<Config, &'static str>
        where T: Iterator<Item = String>
    {
        //First argument is program name
        args.next();

        let row = match args.next() {
            Some(arg) => match arg.parse::<usize>() {
                Ok(val) => val,
                Err(_) => return Err("row must be positive integer")
            },
            None => return Err("not enough arguments")
        };
        let col = match args.next() {
            Some(arg) => match arg.parse::<usize>() {
                Ok(val) => val,
                Err(_) => return Err("col must be positive integer")
            },
            None => return Err("not enough arguments")
        };
        let density = match args.next() {
            Some(arg) => match arg.parse::<f32>() {
                Ok(val) => val,
                Err(_) => return Err("density must be float")
            },
            None => return Err("not enough arguments")
        };

        Ok(Config{row, col, density})
    }
}

struct Game {
    space: Space,
    cache: Box<Space>,
    alive_count: usize,
    size: (usize, usize),
    generation: usize
}

impl Game {
    pub fn new(row: usize, col: usize) -> Game {

        Game{space: Space::new(), cache: Box::new(Space::new()), alive_count: 0, size: (row, col), generation: 0}
    }

    fn init(&mut self, density: f32) -> Result<(), &'static str> {
        if 0f32 > density || 1f32 < density {
            return Err("density must be within range [0, 1]");
        }
        
        let target = (self.size.0 * self.size.1) as f32 * density;
        while self.alive_count < target as usize {
            let coord = (rand::thread_rng().gen_range::<usize>(0, self.size.0) as i32, rand::thread_rng().gen_range::<usize>(0, self.size.1) as i32);
            if !self.cell_state(coord) {
                self.set_cell_state(coord);
                self.insert_cache(self.neighbors(coord));
            }
        }
        Ok(())
    }

    pub fn alive(&self) -> usize {
        self.alive_count
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    fn set_cell_state(&mut self, coord: Coord) {
        self.space.insert(coord);
        self.alive_count += 1;
    }

    fn insert_cache(&mut self, coords: [Coord; 9]) {
        for c in &coords {
            if c.0 >= 0 && c.1 >= 0 && self.size.0 > c.0 as usize && self.size.1 > c.1 as usize {
                self.cache.insert(*c);
            }
        }
    }

    fn cell_state(&self, coord: Coord) -> bool {
        self.space.contains(&coord)
    }

    fn neighbors(&self, coord: Coord) -> [Coord; 9] {
        let (x, y) = coord;
        [(x, y),
         (x + 1, y),
         (x - 1, y),
         (x, y + 1),
         (x, y - 1),
         (x + 1, y + 1),
         (x + 1, y - 1),
         (x - 1, y + 1),
         (x - 1, y - 1)
        ]
    }

    fn next_cell_state(&mut self, coord: Coord) -> (bool, Option<[Coord; 9]>) {
        let neighbors = self.neighbors(coord);
        match neighbors.iter()
                .map(|c| self.cell_state(*c))
                .filter(|c| *c)
                .collect::<Vec<_>>()
                .len()
        {
            3 => {
                if !self.cell_state(coord) {
                    //Cell revived
                    self.alive_count += 1usize;
                }
                (true, Some(neighbors))
            },
            4 => (self.cell_state(coord), None),
            _ => {
                if self.cell_state(coord) {
                    //Cell died
                    self.alive_count -= 1usize;
                }
                (false, Some(neighbors))
            }
        }
    }

    fn next(&mut self) {
        let cache = self.cache.clone();
        let mut next_space = Space::new();
        self.cache = Box::new(Space::new());

        for coord in cache.iter() {
            let next_state = self.next_cell_state(*coord);
            if next_state.0 {
                next_space.insert(*coord);
            }
            if let Some(neighbors) = next_state.1 {
                self.insert_cache(neighbors);
            };
        }

        self.space = next_space;
        self.generation += 1;
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rust Life:\n\tGeneration {}\n\tAlive {}\n\n", self.generation, self.alive_count)?;
        for row in 0..self.size.0 {
            let mut row_str = String::new();
            for col in 0..self.size.1 {
                if self.cell_state((row as i32, col as i32)) {
                    row_str.push_str("*");
                } else {
                    row_str.push_str(" ")
                }
            }
            write!(f, "|{}|\n", row_str)?;
        }

        Ok(())
    }
}

pub fn run<T>(args: T) -> Result<(), Box<dyn Error>> 
    where T: Iterator<Item = String>
{
    let config = Config::new(args)?;
    let mut game = Game::new(config.row, config.col);
    game.init(config.density)?;

    while game.generation() < 1_000_000 && game.alive() > 0 {
        println!("{}{}", "\n".repeat(config.row+4), game);
        std::thread::sleep(std::time::Duration::from_millis(50));
        //println!("{}", game.generation());
        game.next();
    }

    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn set_cell_in_current_space() {
        let mut game = new_test_game(10, 10);
        //Cell state is false initially
        assert!(!game.cell_state((0, 0)));

        game.set_cell_state((0, 0));
        assert!(game.cell_state((0, 0)));
    }
    
    #[test]
    fn initialize_space_with_density() {
        let game = init_test_game(10, 10, 0.711);
        
        let expected_count = (10f32 * 10f32 * 0.711).floor();
        let alive_count = game.alive();
        assert_eq!(alive_count, expected_count as usize,
            "expected {} living cells, observed {}", expected_count as usize, alive_count);
        assert!(game.cache.len() > expected_count as usize,
            "expected at least {} items in cache", expected_count);
    }

    #[test]
    fn should_die() {
        let mut game = new_test_game(10, 10);
        let living_cells = vec![(0,0)];
        
        prime_test_cache(&mut game, living_cells);
        game.next();
        assert_eq!(game.alive(), 0,
            "no cells should be living");
    }

    #[test]
    fn should_live() {
        let mut game = new_test_game(10, 10);
        let living_cells = vec![(0, 0), (0, 1), (1, 0), (1, 1)];

        prime_test_cache(&mut game, living_cells);
        game.next();
        assert_eq!(game.alive(), 4,
            "no cells should be living");
    }

    fn new_test_game(row: usize, col: usize) -> Game {
        Game::new(row, col)
    }

    fn init_test_game(row: usize, col: usize, density: f32) -> Game {
        let mut game = Game::new(row, col);
        if let Err(e) = game.init(density) {
            panic!("{}", e);
        };
        game
    }

    fn prime_test_cache(game: &mut Game, cells: Vec<Coord>) {
        for cell in &cells {
            let neighbors = game.neighbors(*cell);
            game.set_cell_state(*cell);
            game.insert_cache(neighbors);
        }
    }
}