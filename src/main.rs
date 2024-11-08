mod handleSessionDB;
mod util;

use handleSessionDB::{add_skill, get_list_of_skills, JSON_DB_FILE_PATH};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{palette::material::YELLOW, Color, Modifier, Style, Styled, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{block::Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use std::{char, io};
use util::searchInVector;

//TODO : Create seperate mods for different Tabs
enum Tab {
    Home,
    Timer,
}

enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    input_skill: String,
    character_index: usize,
    input_mode: InputMode,
    total_skills: Vec<String>,
    skills: Vec<String>,
    exit: bool,
}

impl App {
    const fn new() -> Self {
        Self {
            input_skill: String::new(),
            input_mode: InputMode::Normal,
            total_skills: Vec::new(),
            skills: Vec::new(),
            character_index: 0,
            exit: false,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_skill.insert(index, new_char);
        self.move_cursor_right();
        //TODO : Search and update the result vector
        self.skills = searchInVector(&self.skills, &self.input_skill);
    }

    fn byte_index(&self) -> usize {
        self.input_skill
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input_skill.len())
    }
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // do this cause - delete can happen at any index
            // getting all the character before the character deleted.
            let before_char_to_delete = self.input_skill.chars().take(from_left_to_current_index);
            //getting all the character after the character deleted
            let after_char_to_delete = self.input_skill.chars().skip(current_index);

            //put the above 2 sets together
            self.input_skill = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
        //TODO: Search in the total skills set and update the result vector
        self.skills = searchInVector(&self.total_skills, &self.input_skill);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_skill.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_messages(&mut self) {
        //TODO : Check if the given skill already exists in json database
        //if the search result skills is empty, then it means there is no match
        if self.skills.len() == 0 {
            let result = add_skill(JSON_DB_FILE_PATH, &self.input_skill.clone());
            if result.is_ok() {
                self.total_skills.push(self.input_skill.clone());
                self.skills.push(self.input_skill.clone());
            }
        }
        // self.input_skill.clear();
        // self.reset_cursor();
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        //Add skills before starting the run loop
        self.total_skills
            .append(&mut get_list_of_skills(JSON_DB_FILE_PATH));
        self.skills = self.total_skills.clone();
        //TODO : If the Tab is Timer, run timer's draw method
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);

        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit ".into(),
                    "e".bold(),
                    " to start editing ".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),

            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input_skill.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));

        frame.render_widget(input, input_area);

        match self.input_mode {
            InputMode::Normal => {}

            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            )),
        }

        let messages: Vec<ListItem> = self
            .skills
            .iter()
            .enumerate()
            .map(|(i, m)| {
                // println!("skill is : {}", m);
                let mut content = Line::from(Span::raw(format!("{i}: {m}")));
                if i == 0 {
                    content = content.bg(Color::LightRed);
                }
                ListItem::new(content)
            })
            .collect();

        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => return Ok(()),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                _ => {}
            },

            InputMode::Editing if key_event.kind == KeyEventKind::Press => match key_event.code {
                KeyCode::Enter => self.submit_messages(),
                KeyCode::Char(char_to_insert) => self.enter_char(char_to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },

            InputMode::Editing => {}
        }
        return Ok(());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

#[cfg(test)]
mod tests {
    const TEST_DB_FILE_NAME: &str = "resources/sessionDBTest.json";
    use std::{ascii::AsciiExt, fs::File};

    use super::*;
    use handleSessionDB::{add_skill, SkillEntry};
    use io::BufReader;

    #[test]
    fn check_read_json_file() {
        let list_of_test_names: Vec<String> = get_list_of_skills(TEST_DB_FILE_NAME);
        assert!(list_of_test_names.contains(&"json".to_string()));
    }

    #[test]
    fn check_add_skill_function() {
        let skill_to_add = "bevy";
        let res = add_skill(TEST_DB_FILE_NAME, skill_to_add);

        res.unwrap_or_else(|e| {
            eprintln!("Error : {}", e);
        });

        let file = File::open(TEST_DB_FILE_NAME).unwrap();
        let reader = BufReader::new(file);

        let data: Vec<SkillEntry> = serde_json::from_reader(reader).unwrap();

        assert!(data
            .into_iter()
            .any(|x| x.name.eq_ignore_ascii_case(skill_to_add)));
    }

    #[test]
    fn render() {}

    #[test]
    fn handle_key_event() -> io::Result<()> {
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
