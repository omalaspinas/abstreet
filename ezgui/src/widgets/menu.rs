use crate::screen_geom::ScreenRectangle;
use crate::{text, Canvas, Event, GfxCtx, InputResult, Key, ScreenPt, Text};

// Stores some associated data with each choice
pub struct Menu<T: Clone> {
    prompt: Option<String>,
    // The bool is whether this choice is active or not
    choices: Vec<(Option<Key>, String, bool, T)>,
    current_idx: Option<usize>,
    keys_enabled: bool,
    pos: Position,

    row_height: f64,
    top_left: ScreenPt,
    first_choice_row: ScreenRectangle,
    total_height: f64,
}

#[derive(Clone)]
pub enum Position {
    ScreenCenter,
    TopLeftAt(ScreenPt),
    // Extra width needed -- if more than the natural menu length, this'll pad the menu.
    TopRightOfScreen(f64),
}

impl<T: Clone> Menu<T> {
    pub fn new(
        prompt: Option<String>,
        choices: Vec<(Option<Key>, String, T)>,
        keys_enabled: bool,
        pos: Position,
        canvas: &Canvas,
    ) -> Menu<T> {
        if choices.is_empty() {
            panic!("Can't create a menu without choices for {:?}", prompt);
        }

        // Calculate geometry.
        let mut txt = Text::new();
        if let Some(ref line) = prompt {
            txt.add_line(line.to_string());
        }
        for (hotkey, choice, _) in &choices {
            if let Some(key) = hotkey {
                txt.add_line(format!("{} - {}", key.describe(), choice));
            } else {
                txt.add_line(choice.to_string());
            }
        }
        let (total_width, total_height) = canvas.text_dims(&txt);
        let row_height = total_height / (txt.num_lines() as f64);

        let top_left = match pos {
            Position::TopLeftAt(pt) => pt,
            Position::ScreenCenter => {
                let mut pt = canvas.center_to_screen_pt();
                pt.x -= total_width / 2.0;
                pt.y -= total_height / 2.0;
                pt
            }
            Position::TopRightOfScreen(extra_width) => {
                let w = if extra_width > total_width {
                    extra_width
                } else {
                    total_width
                };
                ScreenPt::new(canvas.window_width - w, canvas.line_height)
            }
        };

        Menu {
            prompt: prompt.clone(),
            // All choices start active.
            choices: choices
                .into_iter()
                .map(|(key, choice, data)| (key, choice, true, data))
                .collect(),
            current_idx: if keys_enabled { Some(0) } else { None },
            keys_enabled,
            pos,

            row_height,
            top_left,
            first_choice_row: if prompt.is_some() {
                ScreenRectangle {
                    x1: top_left.x,
                    y1: top_left.y + row_height,
                    x2: top_left.x + total_width,
                    y2: top_left.y + (2.0 * row_height),
                }
            } else {
                ScreenRectangle {
                    x1: top_left.x,
                    y1: top_left.y,
                    x2: top_left.x + total_width,
                    y2: top_left.y + row_height,
                }
            },
            total_height,
        }
    }

    pub fn event(&mut self, ev: Event, canvas: &Canvas) -> InputResult<T> {
        // Handle the mouse
        if ev == Event::LeftMouseButtonDown {
            if let Some(i) = self.current_idx {
                let (_, choice, active, data) = self.choices[i].clone();
                if active {
                    return InputResult::Done(choice, data);
                } else {
                    return InputResult::StillActive;
                }
            } else {
                return InputResult::Canceled;
            }
        } else if ev == Event::RightMouseButtonDown {
            return InputResult::Canceled;
        } else if let Event::MouseMovedTo(pt) = ev {
            let mut matched = false;
            for i in 0..self.choices.len() {
                if self.choices[i].2
                    && self
                        .first_choice_row
                        .translate(0.0, (i as f64) * self.row_height)
                        .contains(pt)
                {
                    self.current_idx = Some(i);
                    matched = true;
                    break;
                }
            }
            if !matched && !self.keys_enabled {
                self.current_idx = None;
            }
            return InputResult::StillActive;
        }

        // Handle keys
        if ev == Event::KeyPress(Key::Escape) {
            return InputResult::Canceled;
        }

        if self.keys_enabled {
            let idx = self.current_idx.unwrap();
            if ev == Event::KeyPress(Key::Enter) {
                let (_, name, active, data) = self.choices[idx].clone();
                if active {
                    return InputResult::Done(name, data);
                } else {
                    return InputResult::StillActive;
                }
            } else if ev == Event::KeyPress(Key::UpArrow) {
                if idx > 0 {
                    self.current_idx = Some(idx - 1);
                }
            } else if ev == Event::KeyPress(Key::DownArrow) {
                if idx < self.choices.len() - 1 {
                    self.current_idx = Some(idx + 1);
                }
            }
        }

        if let Event::KeyPress(key) = ev {
            for (maybe_key, choice, active, data) in &self.choices {
                if *active && Some(key) == *maybe_key {
                    return InputResult::Done(choice.to_string(), data.clone());
                }
            }
        }

        if let Event::WindowResized(_, _) = ev {
            // Recreate the menu, then steal the geometry from it.
            let new = Menu::new(
                self.prompt.clone(),
                self.choices
                    .iter()
                    .map(|(key, choice, _, data)| (*key, choice.to_string(), data.clone()))
                    .collect(),
                self.keys_enabled,
                self.pos.clone(),
                canvas,
            );
            self.top_left = new.top_left;
            self.first_choice_row = new.first_choice_row;
            return InputResult::StillActive;
        }

        InputResult::StillActive
    }

    pub fn draw(&self, g: &mut GfxCtx) {
        let mut txt = Text::new();
        if let Some(ref line) = self.prompt {
            txt.add_styled_line(line.to_string(), None, Some(text::PROMPT_COLOR));
        }
        for (idx, (hotkey, choice, active, _)) in self.choices.iter().enumerate() {
            let bg = if Some(idx) == self.current_idx {
                Some(text::SELECTED_COLOR)
            } else {
                None
            };
            if *active {
                if let Some(key) = hotkey {
                    txt.add_styled_line(key.describe(), Some(text::HOTKEY_COLOR), bg);
                    txt.append(format!(" - {}", choice), None);
                } else {
                    txt.add_styled_line(choice.to_string(), None, bg);
                }
            } else {
                if let Some(key) = hotkey {
                    txt.add_styled_line(
                        format!("{} - {}", key.describe(), choice),
                        Some(text::INACTIVE_CHOICE_COLOR),
                        bg,
                    );
                } else {
                    txt.add_styled_line(choice.to_string(), Some(text::INACTIVE_CHOICE_COLOR), bg);
                }
            }
        }
        g.canvas.mark_covered_area(ScreenRectangle {
            x1: self.top_left.x,
            y1: self.top_left.y,
            x2: self.first_choice_row.x2,
            y2: self.top_left.y + (self.row_height * (txt.num_lines() as f64)),
        });
        g.draw_text_at_screenspace_topleft(txt, self.top_left);
    }

    pub fn current_choice(&self) -> Option<&T> {
        let idx = self.current_idx?;
        Some(&self.choices[idx].3)
    }

    // If there's no matching choice, be silent. The two callers don't care.
    pub fn mark_active(&mut self, choice: &str) {
        for (_, action, ref mut active, _) in self.choices.iter_mut() {
            if choice == action {
                if *active {
                    panic!("Menu choice for {} was already active", choice);
                }
                *active = true;
                return;
            }
        }
    }

    pub fn mark_all_inactive(&mut self) {
        for (_, _, ref mut active, _) in self.choices.iter_mut() {
            *active = false;
        }
    }

    // Assume that this doesn't vastly affect width.
    pub fn change_prompt(&mut self, prompt: String) {
        assert!(self.prompt.is_some());
        self.prompt = Some(prompt);
    }

    pub fn get_bottom_left(&self) -> ScreenPt {
        ScreenPt::new(self.top_left.x, self.top_left.y + self.total_height)
    }
}