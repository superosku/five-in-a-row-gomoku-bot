extern crate rand;

use rand::seq::SliceRandom;

use std::f32;

use rayon::prelude::*;
use rand::Rng;


pub struct Node {
    nodes: Vec<Node>,
    x: u32,
    y: u32,
    plays: u32,
    wins: u32,
    leaf: bool,
    root: bool
}


pub struct Board {
    shape: u32,
    play_count: u32,
    done: bool,
    data: Vec<i8>,
    last_play_position: (u32, u32),
}


impl Clone for Board {
    fn clone(&self) -> Board {
        Board{
            shape: self.shape,
            play_count: self.play_count,
            done: self.done,
            data: self.data.to_vec(),
            last_play_position: self.last_play_position,
        }
    }
}


impl Node {
    pub fn new(x: u32, y: u32) -> Node {
        Node {
            x: x,
            y: y,
            plays: 0,
            wins: 0,
            leaf: true,
            nodes: Vec::new(),
            root: false
        }
    }

    pub fn new_root() -> Node {
        Node {
            x: 0,
            y: 0,
            plays: 0,
            wins: 0,
            leaf: true,
            nodes: Vec::new(),
            root: true
        }
    }

    pub fn set_board_at(&mut self, board: &mut Board, x: u32, y: u32, val: i8) {
        if !self.root {
            board.set_at(x, y, val);
        }
    }

    pub fn get_sub_node_at(&self, x: u32, y: u32) -> Option<&Node> {
        for node in self.nodes.iter() {
            if node.x == x && node.y == y {
                return Some(node)
            }
        }
        None
    }

    pub fn add_other(&mut self, other: &Node) {
        for i in 0..self.nodes.len() {
            self.nodes[i].plays += other.nodes[i].plays;
            self.nodes[i].wins += other.nodes[i].wins;
        }
    }

    pub fn run_sim(&mut self, board: &mut Board, depth: u32, player: i8, orig_player: i8) -> u32 {
        let plays_per_expand = 10;

        self.plays += plays_per_expand;
        let next_player = if self.root {player} else {-player};

        self.set_board_at(board, self.x, self.y, player);

        // Check if this move is winning move, Don't expand if so.
        if !self.root {
            if board.define_win(player, self.x as i32, self.y as i32) {
                if (player == orig_player) {
                    self.wins += plays_per_expand;
                    self.set_board_at(board, self.x, self.y, 0);
                    return plays_per_expand
                } else {
                    self.wins += plays_per_expand;
                    self.set_board_at(board, self.x, self.y, 0);
                    return 0
                }
            }
        }

        // If we are leaf, lets expand
        if self.leaf {
            self.leaf = false;

            for y in 0..board.shape {
                for x in 0..board.shape {
                    if board.get_at(x, y) == 0 {
                        if board.play_count == 0 || board.would_play_here(x as i32, y as i32) {
                            self.nodes.push(Node::new(x, y))
                        }
                    }
                }
            }

            if (!self.root) {
                let mut wins = 0;
                for _ in 0..(plays_per_expand as usize) {
                    let (result, _) = board.rec_random_play(player, depth); // TODO: Vaiko -player?
                    if result == orig_player {
                        wins += 1;
                    }
                }
                self.wins += wins;
                self.set_board_at(board, self.x, self.y, 0);
                return wins
//                 return if player == orig_player {wins} else {plays_per_expand - wins}
            }
            self.set_board_at(board, self.x, self.y, 0);
            return 0
        }

        // Define what node to follow
        let mut play_pos = 0;
        let mut best_score = 0.0;
        for (i, node) in self.nodes.iter().enumerate() {
            let wins = if player == orig_player {node.wins} else {node.plays - node.wins};
//             let wins = node.wins;
            let score = if node.plays == 0
                {100000.0} else
//                 {(node.plays - node.wins) as f32 / node.plays as f32 + 2.0 * ((self.plays as f32).ln() / node.plays as f32).sqrt()};
                {wins as f32 / node.plays as f32 + 0.75 * ((self.plays as f32).ln() / node.plays as f32).sqrt()};
            if score > best_score {
                best_score = score;
                play_pos = i;
            }
        }

        // Play!
        unsafe {
            let play_node = self.nodes.get_unchecked_mut(play_pos);

            let score = play_node.run_sim(
                board,
                depth + 1,
                next_player,
                orig_player
            );

            self.wins += if player == orig_player {score} else {plays_per_expand - score};
            self.set_board_at(board, self.x, self.y, 0);
            return score
        }

    }

    pub fn best(&self) -> (u32, u32) {
        for y in 0..12 {
            for x in 0..12 {
                let mut value = 0;
                for node in self.nodes.iter() {
                    if node.x == x && node.y == y {
                        value = node.plays;
                    }
                }
                print!("{:>7} ", value);
            }
            println!("");
        }

        for y in 0..12 {
            for x in 0..12 {
                let mut value = 0.0;
                for node in self.nodes.iter() {
                    if node.x == x && node.y == y {
                        value = node.wins as f32 / node.plays as f32;
                    }
                }
                print!("{:.5} ", value);
            }
            println!("");
        }

        let mut best: f32 = 0.0;
        let mut best_x = 0;
        let mut best_y = 0;
        for leaf in self.nodes.iter() {
            let score: f32 = if leaf.plays == 0 {0.0} else {leaf.wins as f32 / leaf.plays as f32};
            if score > best {
                best = score;
                best_x = leaf.x;
                best_y = leaf.y;
            }
        }
        (best_x, best_y)
    }
}

impl Board {
    pub fn new(shape: u32) -> Board {
        let mut board = Board {
            shape: shape,
            play_count: 0,
            done: false,
            data: Vec::with_capacity((shape * shape) as usize),
            last_play_position: (0, 0)
        };

        for _ in 0usize..((shape * shape) as usize) {
            board.data.push(0)
        }

        board
    }

    pub fn would_play_here(&self, x: i32, y: i32) -> bool {
        for cur_x in -1..2 {
            for cur_y in -1..2 {
//                 println!("HMM {} {}", cur_x, cur_y);
                if self.played_at(x + cur_x, y + cur_y) {
//                     println!("Returning true");
                    return true
                }
            }
        }
//         println!("Returning false");
        return false
    }

    pub fn played_at(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.shape as i32 || y >= self.shape as i32 {
            return false;
        }
//         println!("Getting at {} {} ", x, y);
        return self.get_at(x as u32, y as u32) != 0;
    }

    pub fn get_at(&self, x: u32, y: u32) -> i8 {
        return self.data[(x + y * self.shape) as usize]
    }

    pub fn set_at(&mut self, x: u32, y: u32, val: i8) {
        self.data[(x + y * self.shape as u32) as usize] = val;
    }

    pub fn play_at(&mut self, x: u32, y: u32, player: i8) {
        self.set_at(x, y, player);
        self.play_count += 1;
        if self.define_win(player, x as i32, y as i32) {
            self.done = true;
        }
        self.last_play_position = (x, y);
    }

    pub fn display(&self) {
        print!("o");
        for i in 0..(self.shape * 2 + 2) {
            print!("-");
        }
        print!("o");
        println!("");
        print!("|  |");
        for x in 0..self.shape {
            print!("{:<2}", x % 10);
        }
        println!("");
        for y in 0..self.shape {
            print!("|");
            print!("{:>2}", y);
            print!("|");
            for x in 0..self.shape {
                let value = self.get_at(x, y);
                if value == 0 {
                    print!(" ")
                }
                if self.last_play_position == (x, y) {
                    if value == -1 {
                        print!("\x1b[0;31mX\x1b[0m")
                    }
                    if value == 1 {
                        print!("\x1b[0;34mO\x1b[0m")
                    }
                } else {
                    if value == -1 {
                        print!("\x1b[0;31mx\x1b[0m")
                    }
                    if value == 1 {
                        print!("\x1b[0;34mo\x1b[0m")
                    }
                }
                print!("|");
            }
            println!("");
        }
        print!("o");
        for i in 0..(self.shape * 2 + 2) {
            print!("-");
        }
        print!("o");
        println!("");
    }

    pub fn define_win(&self, player: i8, x: i32, y: i32) -> bool {
        let mut count = 1;
        for i in 1..5 {
            if x + i < self.shape as i32 && self.data[(x + i + y * self.shape as i32) as usize] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1i32..5i32 {
            if x - i >= 0 && self.data[(x - i + y * self.shape as i32) as usize] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 5 {
            return true;
        }

        count = 1;
        for i in 1..5 {
            if y + i < self.shape as i32 && self.data[(x + (y + i) * self.shape as i32) as usize] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1i32..5i32 {
            if y - i >= 0 && self.data[(x + (y - i) * self.shape as i32) as usize] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 5 {
            return true;
        }

        count = 1;
        for i in 1..5 {
            if y + i < self.shape as i32 &&
                x + i < self.shape as i32 &&
                self.data[(x + i + (y + i) * self.shape as i32) as usize] == player
            {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1i32..5i32 {
            if y - i >= 0 &&
                x - i >= 0 &&
                self.data[(x - i + (y - i) * self.shape as i32) as usize] == player
            {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 5 {
            return true;
        }

        count = 1;
        for i in 1..5 {
            if y + i < self.shape as i32 &&
                x - i >= 0 &&
                self.data[(x - i + (y + i) * self.shape as i32) as usize] == player
            {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1i32..5i32 {
            if y - i >= 0 &&
                x + i < self.shape as i32 &&
                self.data[(x + i + (y - i) * self.shape as i32) as usize] == player
            {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 5 {
            return true;
        }

        false
    }

    pub fn rec_random_play(&mut self, player: i8, depth: u32) -> (i8, u32) {
        if depth >= (self.shape * self.shape) - 1 - self.play_count {
            return (0, depth);
        }
//         if depth > 20 {
//             return (0, depth);
//         }

        let mut rng = rand::thread_rng();

        let mut play_x = rng.gen_range(0, self.shape);
        let mut play_y = rng.gen_range(0, self.shape);
        while self.data[(play_x + play_y * self.shape) as usize] != 0 && self.would_play_here(play_x as i32, play_y as i32) {
            play_x = rng.gen_range(0, self.shape);
            play_y = rng.gen_range(0, self.shape);
        }

        self.set_at(play_x, play_y, player);
        let win = self.define_win(player, play_x as i32, play_y as i32);

        if win {
            self.set_at(play_x, play_y, 0);
            return (player, depth);
        }

        let result = self.rec_random_play(-player, depth + 1);
        self.set_at(play_x, play_y, 0);

        return result
    }

    pub fn do_monte_carlo(&mut self, player: i8) -> (u32, u32) {

        let vec: Vec<i32> = (0..10).collect();

        let nodes: Vec<Node> = vec.par_iter().map(|i| {
//         (0..10).iter().collect().par_iter().map(|i| {
            let mut node = Node::new_root();
            for _ in 0..20000 {
                node.run_sim(&mut self.clone(), 0, player, player);
            }
            return node
        }).collect();

        let mut final_node = Node::new_root();
        final_node.run_sim(self, 0, player, player);

        for node in nodes.iter() {
            final_node.add_other(node);
        }

        return final_node.best();


//         let mut node = Node::new_root();
//         for _ in 0..100000 {
//             node.run_sim(self, 0, player, player);
//         }
//         return node.best();
    }

    pub fn play_monte_carlo(&mut self, player: i8) -> (u32, u32) {
        let (x, y) = self.do_monte_carlo(player);
        self.play_at(x, y, player);
        return (x, y)
    }

    pub fn calculate_heuristic_value(&self, player: i8) -> i32 {
        let mut total_value = 0;
        let mut prev_val = 10; // Something that cant be on board
        let mut started_clean = false;
        let mut streak_length = 0;

        // Vertical
        for y in 0..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for x in 0..self.shape {
                let this_val = self.get_at(x, y);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = if this_val == 0 {true} else {false};
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }

        // Horizontal
        for x in 0..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for y in 0..self.shape {
                let this_val = self.get_at(x, y);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = this_val == 0;
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }

        // Diagonal 1
        for start_x in 0..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for i in 0..self.shape {
                if start_x + i >= self.shape {
                    break;
                }
                let this_val = self.get_at(start_x + i, i);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = this_val == 0;
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }
        // Diagonal 2
        for start_y in 1..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for i in 0..self.shape {
                if start_y + i >= self.shape {
                    break;
                }
                let this_val = self.get_at(i, i + start_y);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = this_val == 0;
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }
        // Diagonal 3
        for start_x in 0..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for i in 0..self.shape {
                if start_x < i {
                    break;
                }
                let this_val = self.get_at(start_x - i, i);
//                 println!("WTF {} {}", start_x - i, i);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = this_val == 0;
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }
        // Diagonal 4
        for start_y in 1..self.shape {
            prev_val = 10;
            started_clean = false;
            streak_length = 0;
            for i in 0..self.shape {
                if start_y + i >= self.shape {
                    break;
                }
                let this_val = self.get_at(self.shape - i - 1, start_y + i);
                if this_val == player {
                    if streak_length == 0 && prev_val == 0 {
                        started_clean = true;
                    }
                    streak_length += 1;
                } else {
                    let ended_clean = this_val == 0;
                    if streak_length > 1 && started_clean && ended_clean {
                        total_value += (10_i32.pow(streak_length) as f32 * 1.2) as i32;
                    } else if streak_length > 1 && (started_clean || ended_clean) {
                        total_value += 10_i32.pow(streak_length);
                    }
                    streak_length = 0;
                }
                prev_val = this_val;
            }
        }

        return total_value
    }

    pub fn min_max_step_recursive(
        &mut self, player: i8,
        depth: i32,
        alpha_param: f32,
        beta_param: f32
    ) -> (f32, (u32, u32)) {
//         println!("min_max_step_recursive call {} {} {} {}", player, depth, alpha_param, beta_param);

        //          No pruning | AB pruning
        // Depth 2: 0m0.020s   | 0m0.014s
        // Depth 3: 0m0.290s   | 0m0.018s
        // Depth 4: 0m12.978s  | 0m0.084s
        // Depth 5: *********  | 0m1.181s
        // Depth 6: *********  | ********

        let mut alpha = alpha_param;
        let mut beta = beta_param;

        let mut min = f32::INFINITY;
        let mut min_xys: Vec<(u32, u32)> = Vec::new();
        let mut max = f32::NEG_INFINITY;
        let mut max_xys: Vec<(u32, u32)> = Vec::new();

        for y in 0..self.shape {
            for x in 0..self.shape {
                if self.get_at(x, y) == 0 && self.would_play_here(x as i32, y as i32) {
                    self.set_at(x, y, player);

//                     let mut value = if player == 1 {f32::NEG_INFINITY} else {f32::INFINITY};
                    let mut value = 0.0;

                    if self.define_win(player, x as i32, y as i32) {
                        // Set the value at win
                        value = if player == 1 {f32::INFINITY} else {f32::NEG_INFINITY};
//                         value = if player == 1 {1000000000000.0 / (depth as f32 + 1.0)} else {-1000000000000.0 / (depth as f32 + 1.0)};
                    } else if depth < 4 {
                        // Lets do recursive stuff
                        let (new_value, _) = self.min_max_step_recursive(-player, depth + 1, alpha, beta);
                        value = new_value;
                    } else {
//                         value = 0.0;
                        value = (
                            self.calculate_heuristic_value(1) as f32 -
                            self.calculate_heuristic_value(-1) as f32 * if player == 1 {0.7} else {1.0 / 0.7}
                        )
//                         value /= (depth + 1) as f32;
                    }

                    if value < min {
                        min = value;
                        min_xys.clear();
                    }
                    if value == min {
                        min_xys.push((x, y));
                    }
                    if value > max {
                        max = value;
                        max_xys.clear();
                    }
                    if value == max {
                        max_xys.push((x, y));
                    }

                    self.set_at(x, y, 0);

                    let best_val = if player == -1 {min} else {max};
                    if player == 1 {
                        alpha = alpha.max(best_val);
                    } else {
                        beta = beta.min(best_val);
                    }
                    if alpha >= beta {
                        let mut rng = rand::thread_rng();
                        let min_xy = min_xys[rng.gen_range(0, min_xys.len())];
                        let max_xy = max_xys[rng.gen_range(0, max_xys.len())];
                        return if player == -1 {(min, min_xy)} else {(max, max_xy)};
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();
        let min_xy = min_xys[rng.gen_range(0, min_xys.len())];
        let max_xy = max_xys[rng.gen_range(0, max_xys.len())];
        return if player == -1 {(min, min_xy)} else {(max, max_xy)};
    }

    pub fn do_min_max(&mut self, player: i8) -> (u32, u32) {
        let (_value, play_pos) = self.min_max_step_recursive(player, 0, f32::NEG_INFINITY, f32::INFINITY);
        return play_pos;
    }
}

fn main() {
    println!("Hello, world!");

    let mut board = Board::new(20);

    board.play_at(10, 10, 1);
//     board.play_at(15-0, 10, 1);
//     board.play_at(13, 12, -1);

// //     board.play_at(6, 5, -1);
//     board.play_at(10, 9, 1);
//     board.play_at(10, 8, 1);
//     board.play_at(10, 7, 1);
//     board.play_at(10, 6, -1);
//     board.play_at(11, 6, -1);
// //     board.play_at(10, 6, 1);
// // //     board.play_at(6, 9, -1);
// // //     board.play_at(6, 9, 1);
// // //     board.play_at(6, 10, 1);
// // // //
//     board.play_at(7, 6, -1);
//     board.play_at(7, 7, -1);
//     board.play_at(7, 8, -1);
// //     board.play_at(7, 9, 1);
// //     board.play_at(7, 9, -1);
// //     board.play_at(7, 9, -1);
// //     board.play_at(6, 9, 1);
//     board.display();
//
//     println!(
//         "Heuristical values {} {} Total: {}",
//         board.calculate_heuristic_value(1),
//         board.calculate_heuristic_value(-1),
//         board.calculate_heuristic_value(1) - board.calculate_heuristic_value(-1),
//     );
//
//     println!(
//         "Minmax values {:?}",
//         board.do_min_max(-1)
//     );
// //
//     let mut player = -1;
//     while !board.done {
//         let (play_x, play_y) = board.do_min_max(player);
//
// //         println!("Analyzing palyer {} who played {} {}", player, play_x, play_y);
// //         let mut play_x_val = 0.0;
// //         let mut min_val = f32::INFINITY;
// //         let mut max_val = f32::NEG_INFINITY;
// //         for x in 0..board.shape {
// //             for y in 0..board.shape {
// //                 if board.get_at(x, y) == 0 && board.would_play_here(x as i32, y as i32) {
// //                     board.set_at(x, y, player);
// //                     let heuristic_value = (board.calculate_heuristic_value(1) - board.calculate_heuristic_value(-1)) as f32;
// //
// //                     if x == play_x && y == play_y {
// //                         play_x_val = heuristic_value
// //                     }
// //                     if heuristic_value < min_val {
// //                         min_val = heuristic_value;
// //                     }
// //                     if heuristic_value > max_val {
// //                         max_val = heuristic_value;
// //                     }
// //
// //                     println!("WTF {} {} {}", x, y, heuristic_value);
// //                     board.set_at(x, y, 0);
// //                 }
// //             }
// //         }
// //         if (player == -1 && play_x_val != min_val) || (player == 1 && play_x_val != max_val) {
// //             println!("ERROR");
// //         }
//
//         board.play_at(play_x, play_y, player);
//         board.display();
//
//         player *= -1;
//     }

//     println!("Asdf {} {}", play_x, play_y);

    loop {
        let mut board = Board::new(19);
        board.play_at(9, 9, 1);
        let mut player = -1;
        while !board.done {
            println!("Player {}", player);
            board.display();
            let (x, y) = board.play_monte_carlo(player);

            println!("Play at {} {}", x, y);

            player *= -1;
        }
        board.display();

        break;
    }
}
