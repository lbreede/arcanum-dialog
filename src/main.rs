use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use rand::distr::{Distribution, StandardUniform};
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::string::ToString;
use std::{env, fs, io};

mod dialog;
use dialog::DialogLine;

#[allow(dead_code)]
enum Gender {
    Male,
    Female,
}

// === PLAYER =================================================================
const NAME: &str = "Lennart";
const INTELLIGENCE: i32 = 5;
const GENDER: Gender = Gender::Female;
// ============================================================================

// === NPC ====================================================================
const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Erin", "Frank"];
// ============================================================================

// NOTE: The term dialog "tree" is somewhat of a misnomer, yet here we are...
type DialogTree = HashMap<usize, DialogLine>;

impl DialogLine {
    pub fn get_text(&self, gender: &Gender) -> &str {
        match gender {
            Gender::Male => &self.text,
            Gender::Female => self.female_text.as_deref().unwrap_or(&self.text),
        }
    }
}
#[derive(Debug)]
enum NpcState {
    Stranger,
    Waiting,
    Follower,
}

impl Distribution<NpcState> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NpcState {
        match rng.random_range(0..=2) {
            0 => NpcState::Stranger,
            1 => NpcState::Waiting,
            2 => NpcState::Follower,
            _ => unreachable!(),
        }
    }
}

fn parse_dialog_file(file: &str) -> anyhow::Result<DialogTree> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    let mut dialog_lines: DialogTree = HashMap::new();

    // TODO: Choosing zero here only works because dialogs start at 1 and the first row should
    //  always be an NPC line, hopefully...
    let mut last_npc_number: usize = 0;
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if !line.starts_with("{") {
            continue;
        }
        match DialogLine::try_from(line) {
            Ok(dialog_line) => {
                if dialog_line.response.is_none() {
                    last_npc_number = dialog_line.number;
                } else {
                    dialog_lines
                        .entry(last_npc_number)
                        .and_modify(|line| line.choices.push(dialog_line.number));
                }
                dialog_lines.insert(dialog_line.number, dialog_line);
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to parse dialog line: {}", e)),
        }
    }
    Ok(dialog_lines)
}

//region NPC
#[derive(Debug)]
/// A non-player character to interact with
struct Npc {
    name: String,
    state: NpcState,
    dialog_tree: DialogTree,
}

impl Npc {
    fn new(name: &str, dialog_file: &str, state: NpcState) -> Self {
        Self {
            name: name.to_string(),
            state,
            dialog_tree: parse_dialog_file(dialog_file).unwrap(),
        }
    }

    fn rand() -> Self {
        let dir_path = Path::new("dlg/");
        let entries = fs::read_dir(dir_path).ok().unwrap();
        let dlg_files: Vec<String> = entries
            .filter_map(Result::ok) // Filter out errors
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "dlg"))
            .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
            .collect();
        let dlg_file = dlg_files.iter().choose_stable(&mut rand::rng()).unwrap();
        println!("DEBUG: Loaded file {:?}\n", dlg_file);
        Self {
            name: NAMES
                .iter()
                .choose_stable(&mut rand::rng())
                .unwrap()
                .to_string(),
            state: rand::random(),
            dialog_tree: parse_dialog_file(dlg_file).unwrap(),
        }
    }

    fn set_state(&mut self, state: NpcState) {
        println!(
            "  {}",
            style(match state {
                NpcState::Stranger => format!("✔ {} is now a stranger.\n", self.name),
                NpcState::Waiting =>
                    format!("✔ {} is now waiting where you left them.\n", self.name),
                NpcState::Follower => format!("✔ {} is now a follower.\n", self.name),
            })
            .green()
        );
        self.state = state;
    }

    fn interact(&mut self) {
        match self.state {
            NpcState::Stranger => println!(
                "You approach the stranger. They introduce themselves as {}\n",
                self.name
            ),
            NpcState::Waiting => println!(
                "You approach {}. They have been waiting for you.\n",
                self.name
            ),
            NpcState::Follower => println!("You turn to {}.\n", self.name),
        }
        self.interact_rec(1);
    }

    fn interact_rec(&mut self, number: usize) {
        if number == 0 {
            return; // Reached end
        }
        let Some(npc_line) = self.dialog_tree.get(&number) else {
            panic!("Invalid line number {}", number);
        };
        if npc_line.choices.is_empty() {
            return;
        }
        let mut items = vec![];
        for number in &npc_line.choices {
            items.push(&self.dialog_tree.get(number).unwrap().text)
        }
        let npc_text = npc_line.get_text(&GENDER);
        let prompt = format!("{}: {}", &self.name, npc_text.replace("@pcname@", NAME));
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&items)
            .interact()
            .unwrap();

        let pc_line = self
            .dialog_tree
            .get(&npc_line.choices[selection])
            .unwrap()
            .clone();
        if let Some(intelligence) = pc_line.intelligence {
            if INTELLIGENCE < intelligence {
                println!("{}", style("Not enough intelligence to say this.").red());
                return self.interact_rec(pc_line.number);
            }
        }
        if let Some(test) = pc_line.test {
            match (test.as_str(), &self.state) {
                ("fo", NpcState::Follower) => (),
                ("fo", _) => {
                    println!(
                        "  {}",
                        style(format!(
                            "{} must be a follower for this action.\n",
                            self.name
                        ))
                        .red()
                    );
                    return self.interact_rec(number);
                }
                ("wa", NpcState::Waiting) => (),
                ("wa", _) => {
                    println!(
                        "  {}",
                        style(format!("{} is not currently waiting.\n", self.name)).red()
                    );
                    return self.interact_rec(number);
                }
                _ => {
                    println!(
                        "  {}",
                        style(format!("Test opcode [{}] is not yet implemented!", test)).red()
                    )
                }
            }
        }
        if let Some(result) = pc_line.result {
            match result.as_str() {
                "uw" => self.set_state(NpcState::Follower),
                "so" => println!("Result opcode [so] (spread out) is not yet implemented!"),
                "sc" => println!("Result opcode [sc] (stay close) is not yet implemented!"),
                "wa" => self.set_state(NpcState::Waiting),
                "lv" => self.set_state(NpcState::Stranger),
                _ => println!(
                    "  {}",
                    style(format!(
                        "Result opcode [{}] is not yet implemented!",
                        result
                    ))
                    .red()
                ),
            }
        }
        self.interact_rec(pc_line.response.unwrap())
    }
}
//endregion

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut npc = match &args[..] {
        [_, path] => Npc::new("Tim", path, NpcState::Follower),
        [_] => Npc::rand(),
        _ => panic!("Not sure how to handle {:?}", &args),
    };
    let items = vec!["Talk", "Leave"];
    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What do you want to do?")
            .items(&items)
            .interact()
            .unwrap();
        match selection {
            0 => npc.interact(),
            1 => break,
            _ => unreachable!(),
        }
    }
}
