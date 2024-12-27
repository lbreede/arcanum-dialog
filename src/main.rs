use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::BufRead;
#[derive(Debug)]
struct DialogLine {
    text: String,
    g_field: Option<String>,
    intelligence: Option<i32>,
    test: Option<String>,
    response: Option<usize>,
    result: Option<String>,
    choices: Vec<usize>,
}

type DialogTree = HashMap<usize, DialogLine>;

#[derive(Debug)]
enum NpcState {
    Stranger,
    Waiting,
    Follower,
}

// PLAYER
const INTELLIGENCE: u32 = 5;

fn parse_dlg_file(file: &str) -> anyhow::Result<DialogTree> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);

    let re = Regex::new(
        r"^\{\s*(?<number>\d+)\s*}\{(?<text>.+)}\{(?<g>.*)}\{\s*(?<intelligence>\d*)\s*}\{\s*(?<test>\w*)\s*}\{\s*(?<response>\d*)\s*}\{\s*(?<result>\w*)\s*}",
    )?;
    let text_re = Regex::new(r"^(?<opcode>\w):(?<value>\s\d+|)$")?;

    let mut dialog_lines: DialogTree = HashMap::new();

    // TODO: Choosing zero here only works because dialogs start at 1 and the first row should
    //  always be an NPC line
    let mut last_npc_number: usize = 0;

    for line in reader.lines() {
        let line = line?;
        let Some(caps) = re.captures(&line) else {
            panic!("Failed to capture regex groups")
        };

        let number: usize = caps["number"].parse()?;

        let mut text: String = caps["text"].trim().to_string();
        if let Some(text_caps) = text_re.captures(&text) {
            let opcode = &text_caps["opcode"];
            // TODO: Implement using the value as well as the opcode
            let _value = text_caps["value"].trim();
            text = match opcode {
                "B" => "I would like to trade with you.".to_string(),
                "U" => "I need you to help me with one of your skills.".to_string(),
                "K" => "I have a question about the world.".to_string(),
                _ => unimplemented!("Text opcode '{}' is not yet implemented!", opcode),
            };
        }

        let _g_field: Option<String> = Some(caps["g"].trim().to_string()).filter(|s| !s.is_empty());

        // TODO: The match-statement here wraps the i32 in `Some` if parsed correctly, and `None`
        //  otherwise. This may not be the most desirable outcome. Instead, we should attempt to
        //  wrap a valid i32 in `Some`, turn an empty string into `None`, and panic, or at least
        //  print a warning, for anything else, such as "abc".
        let _intelligence = match caps["intelligence"].parse::<i32>() {
            Ok(n) => Some(n),
            Err(_) => None,
        };
        let _test: Option<String> = Some(caps["test"].to_string()).filter(|s| !s.is_empty());
        let response: Option<usize> = match caps["response"].parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => None,
        };
        let _result: Option<String> = Some(caps["result"].to_string()).filter(|s| !s.is_empty());

        let dialog_line = DialogLine {
            text,
            g_field: None,
            intelligence: _intelligence,
            test: _test,
            response,
            result: _result,
            choices: vec![],
        };
        dialog_lines.insert(number, dialog_line);

        if response.is_none() {
            last_npc_number = number;
        } else {
            dialog_lines
                .entry(last_npc_number)
                .and_modify(|line| line.choices.push(number));
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

impl Display for Npc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.name, self.state)
    }
}

impl Npc {
    fn new(name: &str, dialog_file: &str, state: NpcState) -> Self {
        Self {
            name: name.to_string(),
            state,
            dialog_tree: parse_dlg_file(dialog_file).unwrap(),
        }
    }

    fn interact(&mut self, number: usize) {
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

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(npc_line.text.clone())
            .items(&items)
            .interact()
            .unwrap();

        let choice_number = start + selection;
        let Some(pc_line) = self.dialog_tree.get(&choice_number) else {
            panic!("Invalid PC Line at {}", choice_number);
        };

        if let Some(intelligence) = &pc_line.intelligence {
            if &(INTELLIGENCE as i32) < intelligence {
                println!("Not enough intelligence to say this.");
                return self.interact(choice_number);
            }
        }

        if let Some(test) = &pc_line.test {
            match (test.as_str(), &self.state) {
                ("fo", NpcState::Follower) => (),
                ("fo", _) => {
                    println!("{} must be a follower for this action.", self.name);
                    return self.interact(number);
                }
                ("wa", NpcState::Waiting) => (),
                ("wa", _) => {
                    println!("{} is not currently waiting.", self.name);
                    return self.interact(number);
                }
                _ => unimplemented!("Test opcode {} is not yet implemented!", test),
            }
        }

        if let Some(result) = &pc_line.result {
            match result.as_str() {
                "uw" => self.state = NpcState::Follower,
                "so" => println!("Result opcode 'so' (spread out) is not yet implemented!"),
                "sc" => println!("Result opcode 'sc' (stay close) is not yet implemented!"),
                "wa" => self.state = NpcState::Waiting,
                "lv" => self.state = NpcState::Stranger,
                _ => unimplemented!("Result opcode {} is not yet implemented!", result),
            }
        }
        self.interact(pc_line.response.unwrap())
    }
}

fn main() {
    let mut alice = Npc::new("Alice", "dlg/example.dlg", NpcState::Waiting);
    println!("{}", alice);
    alice.interact(1);
    println!("{}", alice);
    alice.interact(1);
    println!("{}", alice);
}