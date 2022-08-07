use egui::Pos2;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wave Function Sudoku".to_owned(),
        window_width: 900,
        window_height: 900,
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(Clone)]
enum Field {
    Collapsed {num: i8},
    Superposition {states: Vec<bool>},
}

struct Sudoku {
    pub fields: Vec<Vec<Field>>,
    pub solved: bool,
}

impl Sudoku {
    fn new() -> Sudoku {
        Sudoku {
            fields: vec![vec![Field::Superposition{ states: vec![true; 9]}; 9]; 9],
            solved: false,
        }
    }

    fn get_cell (&self, x: usize, y: usize) -> &Field {
        &self.fields[x.min(8)][y.min(8)]
    }

    fn collapse_cell(&mut self, x: usize, y: usize, num: i8) {
        self.fields[x][y] = Field::Collapsed{num};
        self.update_superpositions();
    }

    fn solve_all(&mut self) {
        while !self.solved {
            if !self.check_if_solvable() {
                break;
            }
            self.solve_next();
        }
    }

    fn check_if_solvable(&self) -> bool {
        for x in 0..9 {
            for y in 0..9 {
                match &self.fields[x][y] {
                    Field::Collapsed{num: _} => {
                    },
                    Field::Superposition{states} => {
                        if states.iter().filter(|&&x| x).count() as i8 == 0 {
                            return false;
                        }
                    }
                }
            }
        }
        
        return true;
    }

    fn solve_next(&mut self) {
        let mut counts: Vec<Vec<i8>> = vec![vec![0; 9]; 9];
        
        for x in 0..9 {
            for y in 0..9 {
                match &self.fields[x][y] {
                    Field::Collapsed{num: _} => {
                        counts[x][y] = -1;
                    },
                    Field::Superposition{states} => {
                        counts[x][y] = states.iter().filter(|&&x| x).count() as i8;
                    }
                }
            }
        }

        let mut min_count = 9;

        for x in 0..9 {
            for y in 0..9 {
                if counts[x][y] < min_count && counts[x][y] > 0 {
                    min_count = counts[x][y];
                }
            }
        }

        loop {
            let x = rand::gen_range(0, 9);
            let y = rand::gen_range(0, 9);
            match &self.fields[x][y] {
                Field::Collapsed{num: _} => {},
                Field::Superposition{states} => {
                    if states.iter().filter(|&&x| x).count() as i8 == min_count {
                        loop {
                            let num = rand::gen_range(0, 9);
                            if states[num] {
                                self.collapse_cell(x, y, num as i8);
                                self.update_superpositions();
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_superpositions(&mut self) {
        let mut all_solved = true;
        for x in 0..9 {
            for y in 0..9 {
                match &self.fields[x][y] {
                    Field::Collapsed{num: _} => {
                        //Nothing to check here
                    },
                    Field::Superposition{states} => {
                        all_solved = false;
                        let mut found_nums: Vec<bool> = vec![true; 9];
                        for i in 0..9 {
                            if i != x {
                                match &self.fields[i][y] {
                                    Field::Collapsed{num} => {
                                        found_nums[num.clone() as usize] = false;
                                    },
                                    Field::Superposition{states: _} => {
                                        //Nothing to check here
                                    },
                                }
                            }
                            if i != y {
                                match &self.fields[x][i] {
                                    Field::Collapsed{num} => {
                                        found_nums[num.clone() as usize] = false;
                                    },
                                    Field::Superposition{states: _} => {
                                        //Nothing to check here
                                    },
                                }
                            }
                        }

                        let group_index_x = x / 3;
                        let group_index_y = y / 3;

                        for mx in 0..3 {
                            for my in 0..3 {
                                if group_index_x*3 + mx == x || group_index_y*3 + my == y {
                                    continue;
                                }
                                match &self.fields[group_index_x*3 + mx][group_index_y*3 + my] {
                                    Field::Collapsed{num} => {
                                        found_nums[num.clone() as usize] = false;
                                    },
                                    Field::Superposition{states: _} => {
                                        //Nothing to check here
                                    },
                                }
                            }
                        }

                        self.fields[x][y] = Field::Superposition{states: found_nums};
                    }
                } 
            }
        }
        self.solved = all_solved;
    }
}


#[macroquad::main(window_conf)]
async fn main() {
    let mut sudoku = Sudoku::new();
    let mut hovered = false;

    loop {

        egui_macroquad::ui(|egui_ctx| {
            let window = egui::Window::new("Sudoku Solver");
            let response = window
                .show(egui_ctx, |ui| {
                    if ui.button("Reset").clicked() {
                        sudoku = Sudoku::new();
                    }
                    if ui.button("Solve Single").clicked() {
                        sudoku.solve_next();
                    }
                    if ui.button("Solve All").clicked() {
                        sudoku.solve_all();
                    }
                }).unwrap().response;
            hovered = response.rect.contains(match(egui_ctx.input().pointer.hover_pos()) {
                Some(pos) => pos,
                None => Pos2::new(0.0, 0.0),
            });
        });

        if is_mouse_button_down(MouseButton::Left) && !hovered {
            let (mouse_x, mouse_y) = ((mouse_position().0 / 100.0) as usize, (mouse_position().1 / 100.0) as usize);
            match sudoku.get_cell(mouse_x, mouse_y) {
                Field::Collapsed {num: _} => {
                    
                },
                Field::Superposition {states} => {
                    let mx = ((mouse_position().0 - mouse_x as f32 * 100.0) / (100.0 / 3.0)) as usize;
                    let my = ((mouse_position().1 - mouse_y as f32 * 100.0) / (100.0 / 3.0)) as usize;
    
                    let index = mx + my * 3;
                    if states[index] {
                        sudoku.collapse_cell(mouse_x, mouse_y, index as i8);
                    }
                }
            }
        }

        clear_background(BLACK);

        if !hovered {
            draw_selected(&sudoku);
        }

        draw_sudoku(&sudoku);
        
        egui_macroquad::draw();

        next_frame().await;
    }
}

fn draw_selected(sudoku: &Sudoku) {
    let (mouse_x, mouse_y) = ((mouse_position().0 / 100.0) as usize, (mouse_position().1 / 100.0) as usize);  
    match sudoku.get_cell(mouse_x, mouse_y) {
        Field::Collapsed {num: _} => {
            draw_rectangle(mouse_x as f32 * 100.0, mouse_y as f32 * 100.0, 100.0, 100.0, GRAY);
        },
        Field::Superposition {states} => {
            let mx = (mouse_position().0 - mouse_x as f32 * 100.0) / (100.0 / 3.0);
            let my = (mouse_position().1 - mouse_y as f32 * 100.0) / (100.0 / 3.0);

            let index = (mx as usize) + (my as usize) * 3;

            if states[index] {
                draw_rectangle(mouse_x as f32 * 100.0 + mx.floor() * (100.0 / 3.0), 
                    mouse_y as f32 * 100.0 + my.floor() * (100.0 / 3.0), 
                    100.0 / 3.0, 
                    100.0 / 3.0, 
                    GRAY);
            }
        },
    }
}

fn draw_sudoku(sudoku: &Sudoku) {
    for x in 0..9 {
        for y in 0..9 {
            match sudoku.fields.get(x).unwrap().get(y).unwrap() {
                Field::Collapsed{num} => {
                    draw_text(&(num + 1).to_string(), x as f32 * 100.0 + 30.0, y as f32 * 100.0 + 75.0, 100.0, WHITE);
                },
                Field::Superposition{states} => {
                    for sx in 0..3 {
                        for sy in 0..3 {
                            let num = sx + sy * 3;
                            draw_rectangle_lines(x as f32 * 100f32 + sx as f32 * 100f32/3f32,
                                y as f32 * 100f32 + sy as f32 * 100f32/3f32, 
                                100f32/3f32, 
                                100f32/3f32, 
                                1f32,
                            WHITE);
                            if states[num] {
                                draw_text(&(num + 1).to_string(), 
                                x as f32 * 100.0 + 15.0 + sx as f32 * 30.0, 
                                y as f32 * 100.0 + 30.0 + sy as f32 * 30.0, 
                                30.0, 
                                WHITE);
                            }
                        }
                    }
                }
            }
        }
    }

    //Draws lines
    for x in 0..9 {
        draw_rectangle_lines(x as f32 * 100f32, 0f32, 1300f32, 900f32, 3.0, WHITE);
        draw_rectangle_lines(0f32, x as f32 * 100f32, 900f32, 100f32, 3.0, WHITE);
    }
    for x in 0..3 {
        draw_rectangle_lines(x as f32 * 300f32, 0f32, 300f32, 900f32, 5.0, WHITE);
        draw_rectangle_lines(0f32, x as f32 * 300f32, 900f32, 300f32, 5.0, WHITE);
    }
}