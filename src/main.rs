use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::from_array([150.0, 40.0]);
const PLAYER_SPEED: f32 = 550.0;
const BLOCK_SIZE: Vec2 = Vec2::from_array([120.0, 40.0]);
const BALL_SIZE: Vec2 = Vec2::from_array([50.0, 50.0]);
const BALL_SPEED: f32 = 400.0;

fn draw_title_text(text: &str, font: Font) {
    let text_dim = measure_text(text, Some(font), 50, 1f32);
    draw_text_ex(
        text,
        screen_width() * 0.5f32 - text_dim.width * 0.5f32,
        screen_height() * 0.5f32 - text_dim.height * 0.5f32,
        TextParams {
            font: font,
            font_size: 50,
            color: WHITE,
            ..Default::default()
        },
    )
}
enum GameState {
    Menu,
    Game,
    LevelCompleted,
    Dead,
}

struct Player {
    rect: Rect,
    speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5 - PLAYER_SIZE.x * 0.5,
                screen_height() - 100.0,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ),
            speed: PLAYER_SPEED,
        }
    }

    pub fn update(&mut self, delta: f32) {
        let mut x_input = 0f32;
        if is_key_down(KeyCode::A) {
            x_input -= 1f32;
        }
        if is_key_down(KeyCode::D) {
            x_input += 1f32;
        }

        self.rect.x += x_input * delta * self.speed;

        if self.rect.x < 0f32 {
            self.rect.x = 0f32;
        }
        if self.rect.x > screen_width() - PLAYER_SIZE.x {
            self.rect.x = screen_width() - PLAYER_SIZE.x;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, WHITE);
    }
}

#[derive(PartialEq)]
enum BlockType {
    Regular,
    SpawnBallOnDeath,
}

struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType,
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives: 2,
            block_type,
        }
    }
    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => match self.lives {
                3 => GREEN,
                2 => RED,
                1 => ORANGE,
                _ => WHITE,
            },
            BlockType::SpawnBallOnDeath => match self.lives {
                _ => BLUE,
            },
        };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }
}

struct Ball {
    rect: Rect,
    vel: Vec2,
    speed: f32,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BALL_SIZE.x, BALL_SIZE.y),
            vel: vec2(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
            speed: BALL_SPEED,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.rect.x += self.vel.x * delta * self.speed;
        self.rect.y += self.vel.y * delta * self.speed;
        // touch left wall
        if self.rect.x < 0f32 {
            self.vel.x = 1f32;
        }

        // touch right wall
        if self.rect.x > screen_width() - self.rect.w {
            self.vel.x = -1f32;
        }

        // touch ceiling
        if self.rect.y < 0f32 {
            self.vel.y = 1f32;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, WHITE)
    }
}

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    // early exit
    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return false,
    };
    let a_center = a.point() + a.size() * 0.5f32;
    let b_center = b.point() + b.size() * 0.5f32;
    let to = b_center - a_center;
    let to_signum = to.signum();
    match intersection.w > intersection.h {
        true => {
            // bounce on y
            a.y -= to_signum.y * intersection.h;
            vel.y = -to_signum.y * vel.y.abs();
            vel.x += rand::gen_range(-0.2f32, 0.2f32);
        }
        false => {
            // bounce on x
            a.x -= to_signum.x * intersection.w;
            vel.x = -to_signum.x * vel.x.abs();
        }
    }
    true
}

fn default_stats() -> (i32, i32, Player, Vec<Ball>, Vec<Block>) {
    let mut balls: Vec<Ball> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();

    balls.push(Ball::new(vec2(
        screen_width() * 0.5f32 - BALL_SIZE.x * 0.5f32,
        screen_height() * 0.5f32,
    )));

    let (rows, columns) = (6, 5);
    let (padding_x, padding_y) = (5f32, 5f32);
    let total_block_size = BLOCK_SIZE + vec2(padding_x, padding_y);
    let board_start_pos = vec2(
        (screen_width() - (total_block_size.x * rows as f32)) * 0.5f32,
        50f32,
    );
    for i in 0..rows * columns {
        let block_x = (i % rows) as f32 * total_block_size.x;
        let block_y = (i / rows) as f32 * total_block_size.y;
        blocks.push(Block::new(
            board_start_pos + vec2(block_x, block_y),
            BlockType::Regular,
        ));
    }
    for i in 0..2 {
        let rand_block = rand::gen_range(0, blocks.len());
        blocks[rand_block].block_type = BlockType::SpawnBallOnDeath;
    }
    (
        0,             // score
        3,             // lives
        Player::new(), // player
        balls,         // balls vec
        blocks,        // blocks vec
    )
}

#[macroquad::main("breakout")]
async fn main() {
    let main_font = load_ttf_font("res/Roboto/Roboto-Medium.ttf").await.unwrap();
    let main_params = TextParams {
        font: main_font,
        font_size: 16,
        color: WHITE,
        ..Default::default()
    };
    let (mut score, mut player_lives, mut player, mut balls, mut blocks) = default_stats();

    let mut game_state = GameState::Menu;

    loop {
        // drawing
        clear_background(BLACK);

        match game_state {
            GameState::Menu => {
                draw_title_text("Press SPACE to start", main_font);
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game
                }
            }
            GameState::Game => {
                player.update(get_frame_time());

                for ball in balls.iter_mut() {
                    ball.update(get_frame_time());
                }

                let mut spawn = vec![];
                for ball in balls.iter_mut() {
                    resolve_collision(&mut ball.rect, &mut ball.vel, &player.rect);
                    for block in blocks.iter_mut() {
                        if resolve_collision(&mut ball.rect, &mut ball.vel, &block.rect) {
                            block.lives -= 1;
                            if block.lives <= 0 {
                                score += 10;
                                if block.block_type == BlockType::SpawnBallOnDeath {
                                    spawn.push(Ball::new(ball.rect.point()));
                                }
                            }
                        }
                    }
                }
                for ball in spawn.into_iter() {
                    balls.push(ball);
                }

                // iter through blocks, remove block if out of lives
                blocks.retain(|block| block.lives > 0);

                let balls_len = balls.len();
                let was_last_ball = balls_len == 1;
                balls.retain(|ball| ball.rect.y < screen_height());
                let removed_balls_len = balls.len();

                if removed_balls_len < balls_len && was_last_ball {
                    player_lives -= 1;
                    if player_lives <= 0 {
                        game_state = GameState::Dead;
                    }
                    balls.push(Ball::new(vec2(
                        screen_width() * 0.5f32 - BALL_SIZE.x * 0.5f32,
                        screen_height() * 0.5f32,
                    )));
                }
                player.draw();

                for block in blocks.iter() {
                    block.draw();
                }
                for ball in balls.iter() {
                    ball.draw();
                }
                let score_text = &format!("score: {}", score);
                let score_text_dim = measure_text(&score_text, Some(main_font), 16, 1f32);
                draw_text_ex(
                    &score_text,
                    screen_width() * 0.5f32 - score_text_dim.width * 0.5f32,
                    40f32,
                    main_params,
                );

                draw_text_ex(
                    &format!("lives: {}", player_lives),
                    30f32,
                    40f32,
                    main_params,
                );
            }

            GameState::LevelCompleted => {
                draw_title_text(
                    &format!("level complete! total score: {}", score),
                    main_font,
                );

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game
                }
            }

            GameState::Dead => {
                draw_title_text(&format!("you lost. total score: {}", score), main_font);

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                    (score, player_lives, player, balls, blocks) = default_stats();
                }
            }
        }

        next_frame().await;
    }
}
