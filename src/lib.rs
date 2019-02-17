use std::error::Error;
use rand;
use rand::Rng;
use std::fmt;
use std::collections::HashSet;

type Space<T> = Vec<Vec<T>>;
type Coord = (i32, i32);
type Cache = HashSet<(i32, i32)>;

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
    space: Box<Space<bool>>,
    next_space: Box<Space<bool>>,
    cache: Cache,
    alive_count: u32,
    size: (usize, usize),
    generation: u32
}

impl Game {
    pub fn new(row: usize, col: usize) -> Game {
        let space = Box::new(vec![vec![false; col]; row]);
        let next_space = Box::new(vec![vec![false; col]; row]);
        let cache: Cache = Cache::new();

        Game{space, next_space, cache, alive_count: 0, size: (row, col), generation: 0}
    }

    fn init(&mut self, density: f32) -> Result<(), &'static str> {
        if 0f32 > density || 1f32 < density {
            return Err("density must be within range [0, 1]");
        }
        
        let target = (self.size.0 * self.size.1) as f32 * density;
        while self.alive_count < target as u32 {
            let coord = (rand::thread_rng().gen_range::<usize>(0, self.size.0) as i32, rand::thread_rng().gen_range::<usize>(0, self.size.1) as i32);
            if !self.cell_state(coord) {
                //If cell is not alive yet: 
                // 1. Increment alive_count
                // 2. Set cell state in both spaces
                // 3. Insert cell and neighbors into cache
                self.alive_count += 1u32;
                self.set_cell_state(coord, true, true);
                self.insert_cache(self.neighbors(coord));
            }
        }
        Ok(())
    }

    fn set_cell_state(&mut self, coord: Coord, state: bool, both: bool) {
        //Set the cell state in the next space
        let (row, col) = coord;
        self.next_space[row as usize][col as usize] = state;

        if both {
            self.space[row as usize][col as usize] = state;
        }
    }

    fn insert_cache(&mut self, coords: [Coord; 9]) {
        for c in &coords {
            if c.0 >= 0 && c.1 >= 0 && self.size.0 > c.0 as usize && self.size.1 > c.1 as usize {
                self.cache.insert(*c);
            }
        }
    }

    fn cell_state(&self, coord: Coord) -> bool {
        let (row, col) = coord;
        match self.space.iter().nth(row as usize) {
            Some(row) => match row.iter().nth(col as usize) {
                Some(cell) => *cell,
                None => false
            },
            None => false
        }
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

    fn next_cell_state(&mut self, coord: Coord) -> Option<[Coord; 9]> {
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
                    self.set_cell_state(coord, true, false);
                    self.alive_count += 1u32;
                }
                Some(neighbors)
            },
            4 => None,
            _ => {
                if self.cell_state(coord) {
                    //Cell died
                    self.set_cell_state(coord, false, false);
                    self.alive_count -= 1u32;
                }
                Some(neighbors)
            }
        }
    }

    fn next(&mut self) {
        let cache = self.cache.clone();
        self.cache = Cache::new();
        //Iterate through cache and update next space
        for coord in cache.iter() {
            if let Some(neighbors) = self.next_cell_state(*coord) {
                self.insert_cache(neighbors);
            };
        }
        self.space = self.next_space.clone();
        self.next_space = Box::new(*self.space.clone());
        self.generation += 1;
    }
}


impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\rRust Life:\n\tGeneration {}\n\tAlive {}\n\n", self.generation, self.alive_count)?;
        for row in self.space.iter() {
            let mut line = String::new();
            for cell in row {
                line.push_str(match cell {
                    true => "*",
                    false => " "
                });
            }
            write!(f, "\r|{}|\n", line)?;
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

    println!("{}", game);
    while game.generation < 1_000_000 && game.alive_count > 0 {
        game.next();
        
        println!("{}\n", game.generation);
        //std::thread::sleep(std::time::Duration::from_millis(30));
        //std::process::Command::new("cmd").args(&["/c","cls"]).status().unwrap();
    }

    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn set_cell_in_current_space() {
        let mut game = Game::new(10, 10);
        //Cell state is false initially
        assert!(!game.cell_state((0, 0)));

        game.set_cell_state((0, 0), true, true);
        assert!(game.cell_state((0, 0)));
    }
    
    #[test]
    fn initialize_space_with_density() {
        let row = 10;
        let col = 10;
        let density = 0.711;

        let mut game = Game::new(row, col);
        if let Err(e) = game.init(density)
        {
            panic!("Test failed: {}", e);
        };

        let expected_count = (10f32 * 10f32 * density).ceil();
        let alive_count: usize = game.space.iter()
                                        .flat_map(|row| row.iter().filter(|cell| **cell))
                                        .collect::<Vec<&bool>>()
                                        .len();
        assert_eq!(alive_count, expected_count as usize,
            "expected {} living cells, observed {}", expected_count as usize, alive_count);
        assert_eq!(game.cache.len(), expected_count as usize,
            "expected {} items in cache", expected_count);
    }
}