use core::panic;
use std::{collections::HashMap, io::{self, stdout, Write, Stdout}};

use crossterm::{ExecutableCommand, terminal, QueueableCommand, cursor, style::{self, Stylize, Color, Attribute}};
use rand::Rng;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum ScoreType {
    Aces,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    FourOfKind,
    FullHouse,
    LittleStraight,
    BigStraight,
    Yacht,
    Chance,
}

impl ScoreType {
    fn from_u8(num: u8) -> ScoreType{
        match num {
            0 => ScoreType::Aces,
            1 => ScoreType::Twos,
            2 => ScoreType::Threes,
            3 => ScoreType::Fours,
            4 => ScoreType::Fives,
            5 => ScoreType::Sixes,
            6 => ScoreType::FourOfKind,
            7 => ScoreType::FullHouse,
            8 => ScoreType::LittleStraight,
            9 => ScoreType::BigStraight,
            10 => ScoreType::Yacht,
            11 => ScoreType::Chance,
            _ => panic!("Integer exceeds bounds of enum!")
        }
    }
}

struct ScoreTable {
    table: HashMap<ScoreType, u8>,
}

impl ScoreTable {
    fn new() -> Self {
        ScoreTable { table: HashMap::new() }
    }

    fn reset_scores(&mut self) {
        self.table.clear();
    }

    fn check_table(&self, score_type: &ScoreType) -> bool {
        self.table.contains_key(score_type)
    }

    fn table_total(&self) -> u16 {
        let mut sum = 0;
        for (_, score) in &self.table {
            sum += score;
        }

        sum as u16
    }

    fn score_on_table(&mut self, score_type: &ScoreType, roll: &Roll) -> bool {
        if self.check_table(score_type) {
            return false;
        }

        let score = evaluate_score(roll, score_type);

        self.table.insert(*score_type, score);

        true

    }

    fn get_table_value(&self, score_type: &ScoreType) -> String {
        match self.table.get(score_type) {
            Some(score) => {
                return format!(" {}", score)
            },
            None => format!(" X "),
        }
    }

    fn _print_table(&self) {
        println!("{} - {}", "Aces", self.get_table_value(&ScoreType::Aces));
        println!("{} - {}", "Twos", self.get_table_value(&ScoreType::Twos));
        println!("{} - {}", "Threes", self.get_table_value(&ScoreType::Threes));
        println!("{} - {}", "Fours", self.get_table_value(&ScoreType::Fours));
        println!("{} - {}", "Fives", self.get_table_value(&ScoreType::Fives));
        println!("{} - {}", "Sixes", self.get_table_value(&ScoreType::Sixes));

        println!("{} - {}", "Four Of A Kind", self.get_table_value(&ScoreType::FourOfKind));
        println!("{} - {}", "Full House", self.get_table_value(&ScoreType::FullHouse));
        println!("{} - {}", "Little Straight", self.get_table_value(&ScoreType::LittleStraight));
        println!("{} - {}", "Big Straight", self.get_table_value(&ScoreType::BigStraight));
        println!("{} - {}", "Yacht", self.get_table_value(&ScoreType::Yacht));
        println!("{} - {}", "Chance", self.get_table_value(&ScoreType::Chance));
    }
}

struct Roll {
    dice: [u8; 5],
    holds: [bool; 5],
}

impl Roll {
    fn new() -> Self {
        Roll {
            dice: Roll::gen_roll(),
            holds: [false; 5],
        }
    }

    fn _new_fake(roll_tuple: (u8, u8, u8, u8, u8)) -> Self {
        Roll {
            dice: [
                roll_tuple.0,
                roll_tuple.1,
                roll_tuple.2,
                roll_tuple.3,
                roll_tuple.4,
            ],
            holds: [false; 5],
        }
    }

    fn sort(&mut self) {
        self.dice.sort();
    }

    fn reset_holds(&mut self) {
        self.holds = [false; 5];
    }

    fn gen_roll() -> [u8; 5] {
        let mut rng = rand::thread_rng();

        let a = rng.gen_range(1..=6);
        let b = rng.gen_range(1..=6);
        let c = rng.gen_range(1..=6);
        let d = rng.gen_range(1..=6);
        let e = rng.gen_range(1..=6);

        [a, b, c, d, e]
    }

    fn roll_with_holds(&mut self) {
        let mut rng = rand::thread_rng();

        for i in 0..5 {
            if !self.holds[i] {
                self.dice[i] = rng.gen_range(1..=6);
            }
        }
    }

    fn hold(&mut self, num: &DiceNum) -> bool {
        if self.holds[*num as usize] {
            self.holds[*num as usize] = false;
            false
        } else {
            self.holds[*num as usize] = true;
            true
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DiceNum {
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
}

#[derive(PartialEq, Debug)]
enum GameStates {
    FirstRoll,
    SecondRoll,
    ThirdRoll,
    GameOver,
}

struct Game {
    game_state: GameStates,
    current_roll: Roll,
    score_table: ScoreTable,
    msg: String,
}

impl Game {
    fn new() -> Self {
        Game {
            game_state: GameStates::FirstRoll,
            current_roll: Roll::new(),
            score_table: ScoreTable::new(),
            msg: "".to_string(),
        }
    }

    fn advance_gamestate(&mut self) {
        match self.game_state {
            GameStates::FirstRoll => self.game_state = GameStates::SecondRoll,
            GameStates::SecondRoll => self.game_state = GameStates::ThirdRoll,
            GameStates::ThirdRoll => panic!("Cant advance from Third roll without score"),
            GameStates::GameOver => panic!("This should probably start a new game"),
        }
    }

    fn attempt_command(&mut self, command: &Command) -> Result<String, String>{
        match command {
            Command::Roll => {
                if self.game_state == GameStates::ThirdRoll {
                    return Ok("No more rolls available this round, try 'score'".to_string())
                }

                self.current_roll.roll_with_holds();

                self.advance_gamestate();

                Ok("Onto next roll".to_string())
            },
            Command::Sort => {
                self.current_roll.sort();
                self.current_roll.reset_holds();
                Ok("Dice Sorted!".to_string())
            },
            Command::Score(score_type) => {
                if self.score_table.score_on_table(&score_type, &self.current_roll) {
                    if self.score_table.table.len() == 12 {
                        self.game_state = GameStates::GameOver;
                        return Ok("Game Over! Type 'new' to start a new game!".to_string());
                    }
                    self.game_state = GameStates::FirstRoll;
                    self.current_roll = Roll::new();
                    Ok("Score submitted!".to_string())
                } else {
                    Ok("That score type was already used!".to_string())
                }
            },
            Command::Hold(hold_num) => {
                if self.current_roll.hold(hold_num) {
                    Ok(format!("Held dice number {}", *hold_num as u8 + 1))
                } else {
                    Ok(format!("Unheld dice number {}", *hold_num as u8 + 1))
                }
            },
            Command::New => {
                self.score_table.reset_scores();
                self.current_roll.roll_with_holds();
                self.game_state = GameStates::FirstRoll;
                Ok("New Game Started".to_string())
            },
            Command::NotRecognised(_) => todo!(),
            _ => panic!("Don't know how this happened, may quit wasn't handled?")
        }
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Roll,
    Sort,
    Score(ScoreType),
    Hold(DiceNum),
    New,
    Quit,
    Help(String),
    NotRecognised(String),
}


struct DrawValues {
    score_table_corner: (u16, u16),
    dice_corner: (u16, u16),
    game_status_pos: (u16, u16),
    prompt_pos: (u16, u16),
    title_pos: (u16, u16),
}

const GAME_WIDTH: u16 = 95;
const GAME_HEIGHT: u16 = 35;

fn main() {
    
    let mut game = Game::new();
    let mut stdout = stdout();

    let draw_values = DrawValues {
        score_table_corner: (3, 3),
        dice_corner: (35, 12),
        game_status_pos: (60, 3),
        prompt_pos: (3, 30),
        title_pos: (40, 0)
    };

    draw_once(&mut stdout, &draw_values);
    
    loop {

        draw_update(&game, &mut stdout, &draw_values);

        let mut command = retrieve_command();

        if command == Command::Quit {
            break;
        }

        if let GameStates::GameOver = game.game_state {
            command = Command::New;
        }

        if let Command::New = command {
            draw_once(&mut stdout, &draw_values);
        }

        if let Command::NotRecognised(msg) = command {
            game.msg = msg;
            continue;
        }

        if let Command::Help(msg) = command {
            game.msg = msg;
            continue;
        }

        let result = game.attempt_command(&command);

        game.msg = result.unwrap();
    }
}

fn draw_once(stdout: &mut Stdout, values: &DrawValues) {

    let mut score_name: Vec<String> = Vec::new();

    score_name.push("1  - Aces".to_string());
    score_name.push("2  - Twos".to_string());
    score_name.push("3  - Threes".to_string());
    score_name.push("4  - Fours".to_string());
    score_name.push("5  - Fives".to_string());
    score_name.push("6  - Sixes".to_string());
    score_name.push("7  - Four Of A Kind".to_string());
    score_name.push("8  - Full House".to_string());
    score_name.push("9  - Little Straight".to_string());
    score_name.push("10 - Big Straight".to_string());
    score_name.push("11 - Yacht".to_string());
    score_name.push("12 - Chance".to_string());

    stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();

    //border
    for y in 0..GAME_HEIGHT {
        for x in 0..GAME_WIDTH {
        if (y == 0 || y == GAME_HEIGHT - 1) || (x == 0 || x == GAME_WIDTH - 1) {
            // in this loop we are more efficient by not flushing the buffer.
            stdout
            .queue(cursor::MoveTo(x,y)).unwrap()
            .queue(style::PrintStyledContent( "▓".white())).unwrap();
        }
        }
    }

    stdout.queue(cursor::MoveTo(values.title_pos.0, values.title_pos.1)).unwrap();
    stdout.queue(style::PrintStyledContent(" YACHT DICE "
            .with(Color::Black)
            .on(Color::White)
            .attribute(Attribute::Bold)
        )).unwrap();

    //DRAW SCORE TABLE
    let top_corner = values.score_table_corner;

    stdout.queue(cursor::MoveTo(top_corner.0, top_corner.1 - 1)).unwrap();
    stdout.queue(style::Print("╔═SCORE TABLE══════════╤════╗")).unwrap();

    for i in 0..12 {
        stdout.queue(cursor::MoveTo(top_corner.0, top_corner.1 + (i*2))).unwrap();
        print!("║ {}", score_name[i as usize]);

        stdout.queue(cursor::MoveTo(top_corner.0 + 24, top_corner.1 + (i*2))).unwrap();
        stdout.queue(style::Print("    ║")).unwrap();

        stdout.queue(cursor::MoveTo(top_corner.0, top_corner.1 + (i*2) + 1)).unwrap();

        if i != 11 {
            stdout.queue(style::Print("╟━━━━━━━━━━━━━━━━━━━━━━╋━━━━╢")).unwrap();
        } else {
            stdout.queue(style::Print("╟━━━━━━━━━━━━━━━━━━━━┯━┻━━━━╢")).unwrap();
        }
        
    }

    stdout.queue(cursor::MoveTo(top_corner.0, top_corner.1 + 24)).unwrap();

    stdout.queue(style::Print("║ TOTAL              │      ║")).unwrap();

    stdout.queue(cursor::MoveTo(top_corner.0, top_corner.1 + 25)).unwrap();

    stdout.queue(style::Print("╚════════════════════╧══════╝")).unwrap();

    //DRAW DICE

    let dice_corner = values.dice_corner;
    
    stdout.queue(cursor::MoveTo(dice_corner.0, dice_corner.1)).unwrap();
    stdout.queue(style::Print("┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐")).unwrap();
    stdout.queue(cursor::MoveTo(dice_corner.0, dice_corner.1 + 1)).unwrap();
    stdout.queue(style::Print("│       │  │       │  │       │  │       │  │       │")).unwrap();
    stdout.queue(cursor::MoveTo(dice_corner.0, dice_corner.1 + 2)).unwrap();
    stdout.queue(style::Print("│       │  │       │  │       │  │       │  │       │")).unwrap();
    stdout.queue(cursor::MoveTo(dice_corner.0, dice_corner.1 + 3)).unwrap();
    stdout.queue(style::Print("│       │  │       │  │       │  │       │  │       │")).unwrap();
    stdout.queue(cursor::MoveTo(dice_corner.0, dice_corner.1 + 4)).unwrap();
    stdout.queue(style::Print("└───────┘  └───────┘  └───────┘  └───────┘  └───────┘")).unwrap();

    stdout.flush().unwrap();

}

fn draw_update(game: &Game, stdout: &mut Stdout, values: &DrawValues) {

    let mut score_status: Vec<String> = Vec::new();

    for score_type in 0..12 {
        let x = game.score_table.get_table_value(&ScoreType::from_u8(score_type));
        score_status.push(x);
    }

    //Draw Table Values

    let top_corner = values.score_table_corner;

    for i in 0..12 {

        stdout.queue(cursor::MoveTo(top_corner.0 + 23, top_corner.1 + (i*2))).unwrap();
        print!("┃{}", score_status[i as usize]);
    }

    stdout.queue(cursor::MoveTo(top_corner.0 + 23, top_corner.1 + 24)).unwrap();
    let total = format!("{}  ", game.score_table.table_total());
    stdout.queue(style::Print(total)).unwrap();

    //DRAW GAME STATE
    stdout.queue(cursor::MoveTo(values.game_status_pos.0, values.game_status_pos.1)).unwrap();
    stdout.queue(style::Print("Game Status:               ")).unwrap();
    stdout.queue(cursor::MoveTo(values.game_status_pos.0 + 12, values.game_status_pos.1)).unwrap();
    match game.game_state {
        GameStates::FirstRoll => stdout.queue(style::PrintStyledContent(" First Roll "
            .with(Color::Yellow)
            .on(Color::Green)
            .attribute(Attribute::Bold)
        )).unwrap(),
        GameStates::SecondRoll => stdout.queue(style::PrintStyledContent(" Second Roll "
            .with(Color::Black)
            .on(Color::Yellow)
            .attribute(Attribute::Bold)
        )).unwrap(),
        GameStates::ThirdRoll => stdout.queue(style::PrintStyledContent(" Final Roll "
            .with(Color::White)
            .on(Color::Red)
            .attribute(Attribute::Bold)
        )).unwrap(),
        GameStates::GameOver => stdout.queue(style::PrintStyledContent(" GAME OVER "
            .with(Color::Yellow)
            .on(Color::Blue)
            .attribute(Attribute::Bold)
        )).unwrap(),
    };
    
    let dice_corner = values.dice_corner;
    //draw faces

    draw_dice_at(stdout, (dice_corner.0 + 4, dice_corner.1 + 2), game.current_roll.dice[0]);
    draw_dice_at(stdout, (dice_corner.0 + 15, dice_corner.1 + 2), game.current_roll.dice[1]);
    draw_dice_at(stdout, (dice_corner.0 + 26, dice_corner.1 + 2), game.current_roll.dice[2]);
    draw_dice_at(stdout, (dice_corner.0 + 37, dice_corner.1 + 2), game.current_roll.dice[3]);
    draw_dice_at(stdout, (dice_corner.0 + 48, dice_corner.1 + 2), game.current_roll.dice[4]);

    //draw holds
    stdout.queue(cursor::MoveTo(dice_corner.0 + 3, dice_corner.1 + 6)).unwrap();
    if game.current_roll.holds[0] { print!("[X]") } else { print!("[ ]") }
    stdout.queue(cursor::MoveTo(dice_corner.0 + 14, dice_corner.1 + 6)).unwrap();
    if game.current_roll.holds[1] { print!("[X]") } else { print!("[ ]") }
    stdout.queue(cursor::MoveTo(dice_corner.0 + 25, dice_corner.1 + 6)).unwrap();
    if game.current_roll.holds[2] { print!("[X]") } else { print!("[ ]") }
    stdout.queue(cursor::MoveTo(dice_corner.0 + 36, dice_corner.1 + 6)).unwrap();
    if game.current_roll.holds[3] { print!("[X]") } else { print!("[ ]") }
    stdout.queue(cursor::MoveTo(dice_corner.0 + 47, dice_corner.1 + 6)).unwrap();
    if game.current_roll.holds[4] { print!("[X]") } else { print!("[ ]") }


    //cursor to input

    let prompt = values.prompt_pos;

    stdout.queue(cursor::MoveTo(prompt.0, prompt.1)).unwrap();
    stdout.queue(style::Print("                                                                                    ")).unwrap();
    let msg_line = format!("--] {}", game.msg);
    stdout.queue(cursor::MoveTo(prompt.0, prompt.1)).unwrap();
    stdout.queue(style::Print(msg_line)).unwrap();

    stdout.queue(cursor::MoveTo(prompt.0, prompt.1 + 2)).unwrap();
    stdout.queue(style::Print("-->                                              ")).unwrap();
    stdout.queue(cursor::MoveTo(prompt.0 + 4, prompt.1 + 2)).unwrap();

    stdout.flush().unwrap();
}

fn draw_dice_at(stdout: &mut io::Stdout, dice_center: (u16, u16), num: u8) {
    stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 - 1)).unwrap();
    stdout.queue(style::Print("     ")).unwrap();
    stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1)).unwrap();
    stdout.queue(style::Print("     ")).unwrap();
    stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
    stdout.queue(style::Print("     ")).unwrap();

    stdout.queue(cursor::MoveTo(dice_center.0, dice_center.1)).unwrap();

    let dot_symbol = "O";
    
    match num {
        1 => {
            stdout.queue(style::Print(dot_symbol)).unwrap();
        },
        2 => {
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
        },
        3 => {
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
        },
        4 => {
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();

        },
        5 => {
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();

        },
        6 => {
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1 - 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1 + 1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 - 2, dice_center.1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
            stdout.queue(cursor::MoveTo(dice_center.0 + 2, dice_center.1)).unwrap();
            stdout.queue(style::Print(dot_symbol)).unwrap();
        },
        _ => panic!("Tried to draw non-dice face")
    }
}

fn retrieve_command() -> Command {
    let mut raw_input = String::new();

    io::stdin().read_line(&mut raw_input).expect("failed to readline");

    let input: Vec<&str> = raw_input.split_whitespace().collect();

    parse_command_from_input(input)
}

fn parse_command_from_input(input: Vec<&str>) -> Command {
    let Some(first) = input.first() else { return Command::NotRecognised("No input found".to_string())};
    match *first {
        "r" | "roll" => Command::Roll,
        "s" | "sort" => Command::Sort,
        "h" | "hold" => {
            if let Some(num) = input.get(1) {
                if let Ok(i) = num.parse::<u8>() {
                    match i {
                        1 => Command::Hold(DiceNum::First),
                        2 => Command::Hold(DiceNum::Second),
                        3 => Command::Hold(DiceNum::Third),
                        4 => Command::Hold(DiceNum::Fourth),
                        5 => Command::Hold(DiceNum::Fifth),
                        _ => Command::NotRecognised("Invalid Dice Number, should be (1-5)".to_string()),
                    }
                } else {
                    return Command::NotRecognised("Unable to parse dice number (did you enter a number?)".to_string());
                }
                
            } else {
                return Command::NotRecognised("Couldn't find command args".to_string());
            }
        },
        "sc" | "score" => {
            if let Some(arg) = input.get(1) {
                match *arg {
                    "1" | "aces" => Command::Score(ScoreType::Aces),
                    "2" | "twos" => Command::Score(ScoreType::Twos),
                    "3" | "threes" => Command::Score(ScoreType::Threes),
                    "4" | "fours" => Command::Score(ScoreType::Fours),
                    "5" | "fives" => Command::Score(ScoreType::Fives),
                    "6" | "sixes" => Command::Score(ScoreType::Sixes),

                    "7" | "fourofakind" => Command::Score(ScoreType::FourOfKind),
                    "8" | "fullhouse" => Command::Score(ScoreType::FullHouse),
                    "9" | "littlestraight" => Command::Score(ScoreType::LittleStraight),
                    "10" | "bigstraight" => Command::Score(ScoreType::BigStraight),
                    "11" | "yacht" => Command::Score(ScoreType::Yacht),
                    "12" | "chance" => Command::Score(ScoreType::Chance),

                    _ => Command::NotRecognised("Invalid score type".to_string())
                }
            } else {
                return Command::NotRecognised("No score tpye found".to_string());
            }
            
        },
        "help" => {
            if let Some(arg) = input.get(1) {
                match *arg {
                    "roll" | "r" => Command::Help("roll: rolls the dice that aren't held. Counts as a roll!".to_string()),
                    "sort" | "s" => Command::Help("sort: sorts the dice lowest to highest. Clears held dice".to_string()),
                    "hold" | "h" => Command::Help("hold <dice>: holds dice number <dice> exluding it from next rolls".to_string()),
                    "score" | "sc" => Command::Help("score <type>: submits dice to score where <type> is the number of that score type".to_string()),
                    "new" => Command::Help("new: starts a new game, refreshing the scores".to_string()),
                    "quit" | "q" | "exit" | "e" => Command::Help("quit: quits the game".to_string()),
                    "help" => Command::Help("help <command>: shows possible commands or help for <command> (but you know that...)".to_string()),
                    _ => Command::NotRecognised("No help found for that".to_string())
                }
            } else {
                return Command::Help("commands: roll, sort, hold <dice>, score <type>, new, quit, help <command>".to_string());
            }
        }
        "new" => Command::New,
        "quit" | "q" | "exit" | "e" => Command::Quit,

        _ => Command::NotRecognised("Invalid command, try 'help' for list of commands".to_string()),

    }

    
}

fn upper(roll: &Roll, n: u8) -> u8 {
    let mut x = 0;
    for i in roll.dice {
        if i == n {
            x += n;
        }
    }
    x
}

fn evaluate_score(roll: &Roll, score_type: &ScoreType) -> u8 {

    let result = match score_type {
        ScoreType::Aces => upper(roll, 1),
        ScoreType::Twos => upper(roll, 2),
        ScoreType::Threes => upper(roll, 3),
        ScoreType::Fours => upper(roll, 4),
        ScoreType::Fives => upper(roll, 5),
        ScoreType::Sixes => upper(roll, 6),

        ScoreType::FourOfKind => {
            match roll.dice {
                [1, 1, 1, 1, _] => 4,
                [_, 2, 2, 2, 2] => 8,
                [2, 2, 2, 2, _] => 8,
                [_, 3, 3, 3, 3] => 12,
                [3, 3, 3, 3, _] => 12,
                [_, 4, 4, 4, 4] => 16,
                [4, 4, 4, 4, _] => 16,
                [_, 5, 5, 5, 5] => 20,
                [5, 5, 5, 5, _] => 20,
                [_, 6, 6, 6, 6] => 24,
                _ => 0,
            }
        }
        ScoreType::FullHouse => {
            //count the number of duplicates
            let mut first = 0;
            let mut second = 0;
            for i in 1..=6 {
                let mut amount = 0;
                for x in roll.dice {
                    if x == i {
                        amount += 1;
                    }
                }

                if amount == 3 {
                    first = i;

                    //println!("There are 3 {}s", i);
                }

                if amount == 2 {
                    second = i;
                    //println!("There are 2 {}s", i);
                }
            }

            if first == 0 || second == 0 {
                return 0;
            }

            if first != second {
                return 25;
            }

            0
        },
        ScoreType::LittleStraight => match roll.dice {
            [1, 2, 3, 4, 5] => 30,
            _ => 0,
        },
        ScoreType::BigStraight => {
            if roll.dice == [2, 3, 4, 5, 6] {
                30
            } else {
                0
            }
        }
        ScoreType::Yacht => {
            let i = roll.dice[0];
            if roll.dice.iter().all(|&x| x == i) {
                50
            } else {
                0
            }
        }
        ScoreType::Chance => {
            let sum: u8 = roll.dice.iter().sum();
            sum
        }
    };

    result
}
