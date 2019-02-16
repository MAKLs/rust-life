use std::error::Error;
use rand;
use rand::Rng;
use std::fmt;

type Space<T> = Vec<Vec<T>>;
type Coord = (i32, i32);

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
    space: Space<bool>,
    alive_count: u32,
    size: (usize, usize),
    generation: u32
}

impl Game {
    pub fn new(row: usize, col: usize) -> Game {
        Game{space: vec![vec![false; col]; row], alive_count: 0, size: (row, col), generation: 0}
    }

    fn init(&mut self, density: f32) -> Result<(), &'static str> {
        if 0f32 > density || 1f32 < density {
            return Err("density must be within range [0, 1]");
        }
        
        let target = (self.size.0 * self.size.1) as f32 * density;
        while target > self.alive_count as f32 {
            let coord = (rand::thread_rng().gen_range::<usize>(0, self.size.0) as i32, rand::thread_rng().gen_range::<usize>(0, self.size.1) as i32);
            if !self.cell_state(coord) {
                self.alive_count += 1;
                self.set_cell_state(coord, true);
            }
        }
        Ok(())
    }

    fn set_cell_state(&mut self, coord: Coord, state: bool) {
        let (row, col) = coord;
        self.space[row as usize][col as usize] = state;
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

    fn next_cell_state(&self, coord: Coord) -> bool {
        let neighbors = self.neighbors(coord);
        match neighbors.iter()
                .map(|c| self.cell_state(*c))
                .filter(|c| *c)
                .collect::<Vec<_>>()
                .len()
        {
            3 => true,
            4 => self.cell_state(coord),
            _ => false
        }
    }

    fn next(&mut self) {
        let mut new_space: Space<bool> = Vec::new();
        for (i, row) in self.space.iter().enumerate() {
            new_space.push(self.next_row(row, i as i32));
        }
        self.space = new_space;
        self.generation += 1;
    }

    fn next_row(&self, row: &[bool], x: i32) -> Vec<bool> {
        row.iter()
            .enumerate()
            .map(|(y,_)| self.next_cell_state((x, y as i32)))
            .collect()
    }
}


impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\rRust Life: Generation {}\n\n", self.generation)?;
        for row in &self.space {
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
    while game.generation < 1_000_000 {
        game.next();
        println!("{}\n", game);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    Ok(())
}