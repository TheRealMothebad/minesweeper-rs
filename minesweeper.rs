//crates downloaded
extern crate rand;
extern crate regex;

//custom files
mod chars;

//all imports
use rand::thread_rng;
use rand::Rng;
use substring::Substring;
use std::io::Write;
use regex::Regex;
use terminal_size::{Width, Height, terminal_size};

struct GameState{
    game_over : bool,
    win: bool,
    trippedx : usize,
    trippedy : usize,
}

fn main() {
    let mut bsize_x: usize = 0;
    let mut bsize_y: usize = 0;
    let mut valid_inp = false;
    while !valid_inp {
        match &input("Fit board to window size? (y/n): ").to_lowercase() as &str {
            "y"|"" => {
                let size_option = terminal_size();
                if let Some((Width(w), Height(h))) = size_option {
                    bsize_x = ((w - 3) / 4) as usize;
                    bsize_y = ((h - 5) / 2) as usize;
                    valid_inp = true;
                }

            }
            "n" => {
                bsize_x = intput("Board x size: ", "Must be a number between 1 and 26 (inclusize)!\n");
                bsize_y = intput("Board y size: ", "Must be a number between 1 and infinity!");
                valid_inp = true
            }
            _ => {}
        }
    }
    if bsize_x > 26 {
        bsize_x = 26;
    }
    //print!("Making board of size ({}, {})", bsize_x, bsize_y);
    
    let bombs: usize = input("# of bombs: ").parse::<usize>().unwrap();
    let board : Vec<Vec<i8>> = mk_board(bombs, bsize_x, bsize_y);
    let mut hidden : Vec<Vec<bool>> = Vec::new();
    for x in 0 .. bsize_x {
        hidden.push(Vec::new());
        for y in 0 .. bsize_y {
            hidden[x].push(true);
        }
    }

    //print_board(&board, bsize_x, bsize_y);

    let mut gs = GameState {game_over: false, win: false, trippedx: 0, trippedy: 0};
    print!("{esc}c", esc = 27 as char);
    while !gs.game_over {
        print_title(board.len());
        
        pretty_print(&board, &hidden);
        let inp : String = input("Guess: ");
        print!("{esc}c", esc = 27 as char);
        let p_inp : Option<[usize ; 2]> = parse_inp(inp, bsize_x, bsize_y);
        match p_inp {
            Some(loc) => {
                //print!("x: {}, y: {}\n", loc[0], loc[1]);
                hidden[loc[0]][loc[1]] = false;
                if board[loc[0]][loc[1]] == 0 {
                    let mut to_change: Vec<[usize ; 2]> = Vec::new();
                    cascade(&loc, &board, &mut to_change);
                    for x in to_change {
                        hidden[x[0]][x[1]] = false; }
                }
                match check_guess(&loc, &board) {
                    Some(gst) => {gs = gst},
                    _ => {}
                }
            }
            _ => {
                print!("Bad input! Must be a number & letter within the board like \"b4\"\n");
            }
        }
        if win_condition(&board, &hidden) {
            gs.win = true;
            gs.game_over = true;
        }
    }
    print_title(board.len());
    pretty_print(&board, &vec![vec![false; bsize_y]; bsize_x]);
        if gs.win {
        print!("You won! Congrats!\n");
    }
    else {
        print!("You hit a mine at: {}{}\n", to_char(gs.trippedx), gs.trippedy + 1);
    }
}

fn intput(prompt: &str, retry: &str) -> usize {
    loop {
        match input(prompt).parse::<usize>() {
            Ok(u_inp) => {
                return u_inp;
            }
            _ => {
                print!("{}", retry);
            }
        }
    }
}

fn check_guess(loc: &[usize; 2], b: &Vec<Vec<i8>>) -> Option<GameState> {
    match b[loc[0]][loc[1]] {
        -1 => {
            Some(GameState {game_over: true, win: false, trippedx: loc[0], trippedy: loc[1]})
        }
        0 => {None}
        _ => {None}
    }
}

fn cascade(loc: &[usize; 2], b: &Vec<Vec<i8>>, ret: &mut Vec<[usize; 2]>) {
    for s_x in 0 .. 3 { 
        for s_y in 0 .. 3 {
            if !(s_x == 1 && s_y == 1) {
                let t_x: i8 = s_x as i8 + loc[0] as i8 - 1;
                let t_y: i8 = s_y as i8 + loc[1] as i8 - 1;
                if t_x >= 0 && t_x < b.len().try_into().unwrap() {
                    if t_y >= 0 && t_y < b[0].len().try_into().unwrap() {
                        if !ret.contains(&[t_x.try_into().unwrap(), t_y.try_into().unwrap()]) {
                            ret.push([t_x.try_into().unwrap(), t_y.try_into().unwrap()]);
                            if b[t_x as usize][t_y as usize]== 0 {
                                cascade(&[t_x.try_into().unwrap(), t_y.try_into().unwrap()], &b, ret);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn win_condition(b: &Vec<Vec<i8>>, h: &Vec<Vec<bool>>) -> bool {
    for x in 0..b.len() {
        for y in 0..b[x].len() {
            if b[x][y] != -1 && h[x][y] {
                return false;
            }
        }
    }
    return true;
}

fn mk_board(num_bomb: usize, bx: usize, by: usize) -> Vec<Vec<i8>> {
    let mut board : Vec<Vec<i8>> = Vec::new();
        for x in 0 .. bx {
            board.push(Vec::new());
            for y in 0 .. by {
                board[x].push(0);
            }
        }
    let bombs : [Vec<usize>; 2] = bomb_cords(num_bomb, bx, by);
    for i in 0 .. num_bomb {
        board[bombs[0][i]][bombs[1][i]] = -1;
    }
    calc_vals(&mut board, bx, by);
    return board
}

fn bomb_cords(ammount: usize, rangex: usize, rangey: usize) -> [Vec<usize>; 2] {
    let mut list : [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    let mut rng = thread_rng();
    let mut i : usize = 0;
    while i <= ammount {
        let next_x = rng.gen_range(0, rangex);
        let next_y = rng.gen_range(0, rangey);
        let mut duplicate = false;
        for x in 0 .. i {
            if list[0][x] == next_x && list[1][x] == next_y {
                duplicate = true;
                break;
            }
        }
        if !duplicate {
            list[0].push(next_x);
            list[1].push(next_y);
            i += 1;
        }
    }
    return list
}
 
fn calc_vals(b: &mut Vec<Vec<i8>>, bx: usize, by: usize) { //-> Vec<Vec<i8>> {
    for x in 0 .. bx {
        for y in 0 .. by {
            if !(b[x][y] == -1) {
               b[x][y] = adjacent_bombs(b, x, y);
            }
        }
    }
    //return b;
}

fn adjacent_bombs(b: &Vec<Vec<i8>>, x: usize, y: usize) -> i8 {
    let mut bombs = 0;
    for s_x in 0 .. 3 { 
        for s_y in 0 .. 3 {
            if !(s_x == 1 && s_y == 1) {
                let t_x: i8 = s_x as i8 + x as i8 - 1;
                let t_y: i8 = s_y as i8 + y as i8 - 1;
                if t_x >= 0 && t_x < b.len().try_into().unwrap() {
                    if t_y >= 0 && t_y < b[0].len().try_into().unwrap() {
                        if b[t_x as usize][t_y as usize] == -1 {
                            bombs += 1;
                        }
                    }
                }
            }
        }
    }
    return bombs
        //let inp : String = String::from("a2");
}

fn print_board(b: &Vec<Vec<i8>>, xsize: usize, ysize: usize) {
    for y in 0..ysize {
        for x in 0..xsize {
            if b[x][y] == -1 {
                print!("-");
            }
            else {
                print!("{}", b[x][y]);
            }
            if x != b.len() - 1 {
                print!(" ");
            }
        }
        println!("")
    }
}

fn pretty_print(b: &Vec<Vec<i8>>, h: &Vec<Vec<bool>>) {
    //not as fast as I'd like due to string vs str
    let width = b.len();
    let height = b[0].len();
    //print!("wid: {}, height: {}\n", width, height);
    print!("  ");
    for i in 0..width {
        print!("  {} ", chars::CHARS[i]);
    }
    print!("\n  ");
    for _i in 0..width {
        print!("+---");
    }
    print!("+\n");
    for y in 0..height { 
        print!("{}", y + 1);
        if y < 9 {
            print!(" ");
        }
        for x in 0..width {
            print!("| {} ", if h[x][y] { String::from(" ") } else { if b[x][y] == -1 { String::from("#") } else { b[x][y].to_string() } });
        }
        print!("|\n  +");
        for _x in 0..width {
            print!("---+");
        }
        print!("\n");
    }
}

fn print_title(board_len: usize) {
    let title_size: usize; 
    if 3+(4*board_len) >= 15 {
        title_size = 3+(4*board_len);
    }
    else {
        title_size = 16;
    }
    for _i in 0..title_size+1 {
        print!("=");
    }
    print!("\n");
    for _i in 0..(title_size-1)/2 - 7 {
        print!("|");
    }
    print!("  Minesweeper!  ");
    for _i in 0..(title_size-1)/2 - 7 {
        print!("|");
    }
    print!("\n");
    for _i in 0..title_size+1 {
        print!("=");
    }
    print!("\n");
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let mut user_in = String::new();
    std::io::stdout().flush().expect("some error message");
    let _cmdbytes = std::io::stdin().read_line(&mut user_in).unwrap();
    return user_in.trim().to_string();
}

fn to_char(int: usize) -> char {
    chars::CHARS[int]
}

fn from_char(ch: char) -> Option<usize> {
    for i in 0 .. chars::CHARS.len() {
        if chars::CHARS[i] == ch {
            return Some(i);
        }
    }
    None
}

fn parse_inp(inp: String, xsize: usize, ysize: usize) -> Option<[usize ; 2]> {
    let clean = inp.replace(" ", "").to_lowercase();
    let col_row = Regex::new(r"[a-z]\d+").unwrap();
    let row_col = Regex::new(r"\d+[a-z]").unwrap();

    let col: usize;
    let row: usize;

    if col_row.is_match(&clean) {
        col = from_char(clean.chars().nth(0).unwrap()).unwrap();
        row = clean.substring(1, clean.len()).parse::<usize>().unwrap() - 1;
    }
    else if row_col.is_match(&clean) {
        col = from_char(clean.chars().nth(clean.len() - 1).unwrap()).unwrap();
        row = clean.substring(0, clean.len() - 1).parse::<usize>().unwrap() - 1;
    }
    else {
        return None
    }
    if col < xsize && row < ysize {
        return Some([col, row]);
    }
    None
}
