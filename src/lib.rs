use std::error::Error;
use std::collections::HashSet;
use std::fmt;
use rand;
use rand::Rng;

type Space<T> = Vec<Vec<T>>;
type Coord = (usize, usize);
type Cache = HashSet<Coord>;

#[derive(PartialEq, Debug, Clone, Copy)]
enum Cell {
    Alive,
    Dead
}

impl Cell {
    fn new() -> Cell {
        Cell::Dead
    }

    fn kill(&mut self) -> Result<(), &'static str> {
        if let Cell::Dead = *self {
            return Err("cell already dead");
        }
        *self = Cell::Dead;
        Ok(())
    }

    fn revive(&mut self) -> Result<(), &'static str> {
        if let Cell::Alive = *self {
            return Err("cell already alive");
        }
        *self = Cell::Alive;
        Ok(())
    }
}

pub struct Game {
    width: usize,
    height: usize,
    space: Space<Cell>,
    cache: Cache,
    max_iter: u32,
    alive_count: u32,
}

impl Game {
    pub fn new(width: usize, height: usize, max_iter: u32) -> Game {
        let space = vec![vec![Cell::new(); width]; height];
        let cache: Cache = HashSet::new();

        Game{width, height, space, cache, max_iter, alive_count: 0u32}
    }

    pub fn init_space(&mut self, density: f32) -> Result<(), &'static str> {
        //Density must be percent of space that should be alive
        if 0f32 > density || 1f32 < density {
            return Err("density must be within range [0, 1]");
        }

        let mut rng = rand::thread_rng();
        let target_pop = (self.width * self.height) as f32 * density;
        let mut coord: Coord;

        //Revive cells randomly in space until we meet the desired density
        while self.alive_count < target_pop as u32 {
            coord = (rng.gen_range(0, self.width), rng.gen_range(0, self.height));
            if let Ok(()) = self.space[coord.0][coord.1].revive() {
                self.alive_count += 1u32;
                self.cache.insert(coord);
            };
        }

        Ok(())
    }

    fn update_space(&mut self) {

    }

    fn get_neighbor(&self, coord: &Coord) -> [Coord; 9] {
        let (x, y) = coord;
        let (x_0, y_0) = (self.width, self.height);

        [*coord;9]
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.space {
            let mut line = String::new();
            for cell in row {
                line.push_str(match cell {
                    Cell::Alive => " * ",
                    Cell::Dead => "   "
                });
            }
            line.push_str("\n");
            write!(f, "{}", line)?;
        }

        Ok(())
    }
}

pub trait ModuloSignedExt {
    fn modulo(&self, n: Self) -> Self;
}
macro_rules! modulo_signed_ext_impl {
    ($($t:ty)*) => {
        
    };
}

pub fn run() -> Result<(), Box<dyn Error>>{
    let mut game = Game::new(10, 10, 10);

    game.init_space(0.5)?;
    println!("{}", game);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_cell() {
        let cell = Cell::new();
        
        assert_eq!(cell, Cell::Dead);
    }

    #[test]
    fn revive_cell() {
        let mut cell = Cell::new();
        assert_eq!(cell, Cell::Dead);

        if let Err(e) = cell.revive() {
            panic!("{}", e);
        };
        assert_eq!(cell, Cell::Alive);
    }

    #[test]
    fn kill_cell() {
        let mut cell = Cell::new();
        assert_eq!(cell, Cell::Dead);

        if let Err(e) = cell.revive() {
            panic!("{}", e);
        };
        assert_eq!(cell, Cell::Alive);

        if let Err(e) = cell.kill() {
            panic!("{}", e);
        };
        assert_eq!(cell, Cell::Dead);
    }
}