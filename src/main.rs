use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use rand::distr::{Distribution, StandardUniform};
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufRead;
mod dialog;
use dialog::DialogLine;

#[allow(dead_code)]
enum Gender {
    Male,
    Female,
}

// === PLAYER =================================================================
const INTELLIGENCE: i32 = 5;
const GENDER: Gender = Gender::Female;
// ============================================================================

// === NPC ====================================================================
const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Erin", "Frank"];

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

fn parse_field<T: std::str::FromStr>(field: &str) -> Option<T> {
    field.trim().parse().ok()
}
fn parse_dialog_file(file: &str) -> anyhow::Result<DialogTree> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    let mut dialog_lines: DialogTree = HashMap::new();

    // TODO: Choosing zero here only works because dialogs start at 1 and the first row should
    //  always be an NPC line, hopefully...
    let mut last_npc_number: usize = 0;
    for line in reader.lines() {
        let line = line?;
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

#[derive(Debug)]
/// A non-player character to interact with
struct Npc {
    name: String,
    state: NpcState,
    dialog_tree: DialogTree,
}

impl Npc {
    #[allow(dead_code)]
    fn new(name: &str, dialog_file: &str, state: NpcState) -> Self {
        Self {
            name: name.to_string(),
            state,
            dialog_tree: parse_dialog_file(dialog_file).unwrap(),
        }
    }

    fn rand(dialog_file: &str) -> Self {
        Self {
            name: NAMES
                .iter()
                .choose_stable(&mut rand::rng())
                .unwrap()
                .to_string(),
            state: rand::random(),
            dialog_tree: parse_dialog_file(dialog_file).unwrap(),
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
            // Reached end
            return;
        }

        let Some(npc_line) = self.dialog_tree.get(&number) else {
            panic!("Invalid line number {}", number);
        };

        let Some(start) = npc_line.choices.first() else {
            // No choices for this npc line
            return;
        };

        let mut items = vec![];
        for number in &npc_line.choices {
            items.push(&self.dialog_tree.get(number).unwrap().text)
        }
        let npc_text = npc_line.get_text(&GENDER);
        let prompt = format!("{}: {}", &self.name, npc_text);
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&items)
            .interact()
            .unwrap();

        let choice_number = start + selection;

        let pc_line = self.dialog_tree.get(&choice_number).cloned().unwrap();

        if let Some(intelligence) = pc_line.intelligence {
            if INTELLIGENCE < intelligence {
                println!("Not enough intelligence to say this.");
                return self.interact_rec(choice_number);
            }
        }

        if let Some(test) = pc_line.test {
            match (test.as_str(), &self.state) {
                ("fo", NpcState::Follower) => (),
                ("fo", _) => {
                    println!(
                        "  {}",
                        style(format!(
                            "X {} must be a follower for this action.\n",
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
                        style(format!("X {} is not currently waiting.\n", self.name)).red()
                    );
                    return self.interact_rec(number);
                }
                _ => unimplemented!("Test opcode {} is not yet implemented!", test),
            }
        }

        if let Some(result) = pc_line.result {
            match result.as_str() {
                "uw" => self.set_state(NpcState::Follower),
                "so" => println!("Result opcode 'so' (spread out) is not yet implemented!"),
                "sc" => println!("Result opcode 'sc' (stay close) is not yet implemented!"),
                "wa" => self.set_state(NpcState::Waiting),
                "lv" => self.set_state(NpcState::Stranger),
                _ => unimplemented!("Result opcode {} is not yet implemented!", result),
            }
        }
        self.interact_rec(pc_line.response.unwrap())
    }
}

fn main() {
    let mut npc = Npc::rand("dlg/example.dlg");

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
