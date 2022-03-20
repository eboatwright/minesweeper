use macroquad::audio::*;
use macroquad::rand::gen_range;
use macroquad::prelude::*;

use viewport::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

fn window_conf() -> Conf {
	Conf {
		window_title: "Minesweeper".to_string(),
		window_width: WINDOW_WIDTH as i32,
		window_height: WINDOW_HEIGHT as i32,
		..Default::default()
	}
}

struct Particle {
	position: Vec2,
	velocity: Vec2,
	life: f32,
}

enum State {
	Splash,
	Title,
	Game,
}

enum GameState {
	Playing,
	GameOver { won: bool },
}

#[macroquad::main(window_conf)]
async fn main() {
	let mut state = State::Splash;
	let mut game_state = GameState::Playing;

	let mut board_size = 0;

	let mut viewport = Viewport::new(WINDOW_WIDTH, WINDOW_HEIGHT);

	let mut board: Vec<Vec<i32>> = vec![];
	let mut mines: Vec<Vec<bool>> = vec![];
	let mut flags: Vec<Vec<bool>> = vec![];

	let spritesheet_tex = load_texture("res/spritesheet.png").await.unwrap();
	spritesheet_tex.set_filter(FilterMode::Nearest);
	let background_tex = load_texture("res/background.png").await.unwrap();
	background_tex.set_filter(FilterMode::Nearest);
	let title_tex = load_texture("res/title.png").await.unwrap();
	title_tex.set_filter(FilterMode::Nearest);
	let splash_tex = load_texture("res/splash.png").await.unwrap();
	splash_tex.set_filter(FilterMode::Nearest);
	let smoke_tex = load_texture("res/smoke.png").await.unwrap();
	smoke_tex.set_filter(FilterMode::Nearest);

	let flag_sfx = load_sound("res/sfx/flag.ogg").await.unwrap();
	let click_sfx = load_sound("res/sfx/click.ogg").await.unwrap();
	let lose_sfx = load_sound("res/sfx/lose.ogg").await.unwrap();
	let button_sfx = load_sound("res/sfx/button.ogg").await.unwrap();
	let win_sfx = load_sound("res/sfx/win.ogg").await.unwrap();
	let splash_sfx = load_sound("res/sfx/splash.ogg").await.unwrap();

	let mut particles: Vec<Particle> = vec![];

	let font = load_ttf_font("res/font.ttf").await.unwrap();

	let mut splash_timer = 0.0;

	let mut camera_pos = vec2(WINDOW_WIDTH * 0.5, WINDOW_HEIGHT * 0.5);
	let mut screen_shake = Vec2::ZERO;
	let mut screen_shake_timer = 2.0;

	let mut easy_button_rect = Rect {
		x: 352.0,
		y: 0.0,
		w: 96.0,
		h: 54.0,
	};

	let mut medium_button_rect = Rect {
		x: 313.0,
		y: 0.0,
		w: 173.0,
		h: 47.0,
	};
	let mut hard_button_rect = Rect {
		x: 349.0,
		y: 0.0,
		w: 102.0,
		h: 47.0,
	};

	loop {
		viewport.camera.target = camera_pos + screen_shake;
		screen_shake_timer -= get_frame_time() * 60.0;
		if screen_shake_timer <= 0.0 {
			screen_shake_timer = 2.0;
			screen_shake *= -0.5;
		}

		let tile_size = WINDOW_HEIGHT / board_size as f32 - 2.0;
		let half_board_width = tile_size * board_size as f32 * 0.5;
		let sin = (get_time() as f32 * 2.0).sin();

		match state {
			State::Splash => {
				if splash_timer == 0.0 {
					play_sound(splash_sfx, PlaySoundParams { looped: false, volume: 0.9 });
				}
				splash_timer += get_frame_time();
				if splash_timer > 3.0 {
					state = State::Title;
				}
			},
			State::Title => {
				if is_mouse_button_pressed(MouseButton::Left) {
					let mouse_pos = viewport.mouse_position();
					let mouse_rect = Rect {
						x: mouse_pos.x,
						y: mouse_pos.y,
						w: 1.0,
						h: 1.0,
					};

					easy_button_rect.y = 253.0 + sin * 20.0;
					medium_button_rect.y = 315.0 + sin * 20.0;
					hard_button_rect.y = 370.0 + sin * 20.0;

					if mouse_rect.overlaps(&easy_button_rect) {
						board_size = 8;
						generate(&mut board, &mut mines, &mut flags, board_size);
						camera_pos = Vec2::ZERO;
						play_sound(button_sfx, PlaySoundParams { looped: false, volume: 0.8 });
						state = State::Game;
					} else if mouse_rect.overlaps(&medium_button_rect) {
						board_size = 16;
						generate(&mut board, &mut mines, &mut flags, board_size);
						camera_pos = Vec2::ZERO;
						play_sound(button_sfx, PlaySoundParams { looped: false, volume: 0.8 });
						state = State::Game;
					} else if mouse_rect.overlaps(&hard_button_rect) {
						board_size = 24;
						generate(&mut board, &mut mines, &mut flags, board_size);
						camera_pos = Vec2::ZERO;
						play_sound(button_sfx, PlaySoundParams { looped: false, volume: 0.8 });
						state = State::Game;
					}
				}
			},
			State::Game => {
				match game_state {
					GameState::Playing => {
						if is_key_pressed(KeyCode::Escape) {
							camera_pos = vec2(WINDOW_WIDTH * 0.5, WINDOW_HEIGHT * 0.5);
							state = State::Title;
						}

						if is_mouse_button_pressed(MouseButton::Left) {
							let (x, y) = get_mouse_pos(&viewport, half_board_width, tile_size, board_size);
							if !flags[y][x] {
								if mines[y][x] {
									game_state = GameState::GameOver { won: false };
									play_sound(lose_sfx, PlaySoundParams { looped: false, volume: 0.6 });
									screen_shake = vec2(gen_range(-80.0, 80.0), gen_range(-80.0, 80.0));
									for y in 0..board_size {
										for x in 0..board_size {
											if mines[y][x] {
												for _ in 0..5 {
													particles.push(Particle {
														position: vec2(-half_board_width + (x as f32 + 0.2) * tile_size, -half_board_width + (y as f32 + 0.2) * tile_size),
														velocity: vec2(gen_range(-4.0, 4.0), gen_range(-10.0, -1.0)),
														life: 80.0,
													});
												}
											}
										}
									}
								} else if board[y][x] == -1 {
									play_sound(click_sfx, PlaySoundParams { looped: false, volume: 1.0 });
									calculate_tile(x as i32, y as i32, &mut board, &mut flags, &mines, &mut particles, tile_size, half_board_width);
								}
							}
						}

						if is_mouse_button_pressed(MouseButton::Right) {
							let (x, y) = get_mouse_pos(&viewport, half_board_width, tile_size, board_size);
							if board[y][x] == -1 {
								flags[y][x] = !flags[y][x];
								play_sound(flag_sfx, PlaySoundParams { looped: false, volume: 1.0 });
								if check_for_win(&mines, &flags) {
									game_state = GameState::GameOver { won: true };
									play_sound(win_sfx, PlaySoundParams { looped: false, volume: 0.9 });
								}
								for _ in 0..2 {
									particles.push(Particle {
										position: vec2(-half_board_width + (x as f32 + 0.2) * tile_size, -half_board_width + (y as f32 + 0.2) * tile_size),
										velocity: vec2(gen_range(-4.0, 4.0), gen_range(-10.0, -1.0)),
										life: 80.0,
									});
								}
							}
						}
					},
					GameState::GameOver { won } => {
						if is_mouse_button_pressed(MouseButton::Right) {
							generate(&mut board, &mut mines, &mut flags, board_size);
							if won {
								camera_pos = vec2(WINDOW_WIDTH * 0.5, WINDOW_HEIGHT * 0.5);
								state = State::Title;
							}
							game_state = GameState::Playing
						}
					},
				}

				let mut to_destroy = vec![];
				for (i, particle) in particles.iter_mut().enumerate().rev() {
					particle.velocity.y += 0.5;
					particle.velocity.x *= 0.94;
					particle.position += particle.velocity;
					particle.life -= get_frame_time() * 60.0;
					if particle.life <= 0.0 {
						to_destroy.push(i);
					}
				}
				for i in to_destroy {
					particles.remove(i);
				}
			},
		}

		clear_background(DARKGRAY);

		draw_texture(background_tex, camera_pos.x - WINDOW_WIDTH * 0.5, camera_pos.y - WINDOW_HEIGHT * 0.5, WHITE);

		match state {
			State::Splash =>
				draw_texture(splash_tex, 194.0, 220.0 + sin * 20.0, WHITE),
			State::Title =>
				draw_texture(title_tex, 196.0, 160.0 + sin * 20.0, WHITE),
			State::Game => {
				for y in 0..board_size {
					for x in 0..board_size {
						if board[y][x] == -1 {
							draw(spritesheet_tex, x, y, 0.0, half_board_width, tile_size);
						} else {
							draw(spritesheet_tex, x, y, 1.0, half_board_width, tile_size);
							if board[y][x] != 0 {
								draw_text_ex(&format!("{}", board[y][x]), -half_board_width + tile_size * 0.3 + x as f32 * tile_size, -half_board_width + tile_size * 0.75 + y as f32 * tile_size, TextParams {
									color: WHITE,
									font_size: (tile_size * 0.8) as u16,
									font: font,
									..Default::default()
								});
							}
						}

						// This is just to save another double for loop
						if let GameState::GameOver { .. } = game_state {
							if mines[y][x] {
								draw(spritesheet_tex, x, y, 2.0, half_board_width, tile_size);
							}
						}

						if flags[y][x] {
							draw(spritesheet_tex, x, y, 3.0, half_board_width, tile_size);
						}
					}
				}

				for particle in particles.iter() {
					draw_texture(smoke_tex, particle.position.x, particle.position.y, WHITE);
				}

				match game_state {
					GameState::GameOver { won } => {
						if won {
							draw_text_ex("You Win!", -82.0, 0.0, TextParams {
								color: WHITE,
								font,
								font_size: 48,
								..Default::default()
							});
						} else {
							draw_text_ex("Game Over!", -113.0, 0.0, TextParams {
								color: WHITE,
								font,
								font_size: 48,
								..Default::default()
							});
						}
					},
					_ => {},
				}
			},
		}

		viewport.render();
		next_frame().await
	}
}

fn draw(spritesheet_tex: Texture2D, x: usize, y: usize, src: f32, half_board_width: f32, tile_size: f32) {
	draw_texture_ex(
		spritesheet_tex,
		-half_board_width + x as f32 * tile_size,
		-half_board_width + y as f32 * tile_size,
		WHITE,
		DrawTextureParams {
			dest_size: Some(vec2(tile_size, tile_size)),
			source: Some(Rect {
				x: src * 64.0,
				y: 0.0,
				w: 64.0,
				h: 64.0,
			}),
			..Default::default()
		},
	);
}

fn get_mouse_pos(viewport: &Viewport, half_board_width: f32, tile_size: f32, board_size: usize) -> (usize, usize) {
	let mouse_pos = viewport.mouse_position();
	let x = ((half_board_width + mouse_pos[0]) / tile_size).floor() as usize;
	let y = ((half_board_width + mouse_pos[1]) / tile_size).floor() as usize;
	return (clamp(x, 0, board_size - 1), clamp(y, 0, board_size - 1));
}

fn generate(board: &mut Vec<Vec<i32>>, mines: &mut Vec<Vec<bool>>, flags: &mut Vec<Vec<bool>>, board_size: usize) {
	*board = vec![vec![-1; board_size]; board_size];

	*mines = vec![vec![false; board_size]; board_size];
	let mut mines_placed = 0;
	while mines_placed < (board_size as f32 * board_size as f32 * 0.182).round() as i32 {
		let x = gen_range(0usize, board_size);
		let y = gen_range(0usize, board_size);

		if !mines[y][x] {
			mines_placed += 1;
			mines[y][x] = true;
		}
	}

	*flags = vec![vec![false; board_size]; board_size];
}

fn calculate_tile(x: i32, y: i32, board: &mut Vec<Vec<i32>>, flags: &mut Vec<Vec<bool>>, mines: &Vec<Vec<bool>>, particles: &mut Vec<Particle>, tile_size: f32, half_board_width: f32) {
	if flags[y as usize][x as usize] {
		flags[y as usize][x as usize] = false;
	}
	let mut num_of_mines = 0;
	for y_off in [-1i32, 0, 1] {
		for x_off in [-1i32, 0, 1] {
			if x + x_off >= 0 && x + x_off < board.len() as i32
			&& y + y_off >= 0 && y + y_off < board.len() as i32
			&& mines[(y + y_off) as usize][(x + x_off) as usize] {
				num_of_mines += 1;
			}
		}
	}
	let particle_position = vec2(-half_board_width + (x as f32 + 0.2) * tile_size, -half_board_width + (y as f32 + 0.2) * tile_size);
	for _ in 0..5 {
		particles.push(Particle {
			position: particle_position,
			velocity: vec2(gen_range(-4.0, 4.0), gen_range(-10.0, -1.0)),
			life: 80.0,
		});
	}
	board[y as usize][x as usize] = num_of_mines;
	if board[y as usize][x as usize] == 0 {
		for y_off in [-1i32, 0, 1] {
			for x_off in [-1i32, 0, 1] {
				if x + x_off >= 0 && x + x_off < board.len() as i32
				&& y + y_off >= 0 && y + y_off < board.len() as i32
				&& board[(y + y_off) as usize][(x + x_off) as usize] == -1 {
					calculate_tile(x + x_off, y + y_off, board, flags, mines, particles, tile_size, half_board_width);
				}
			}
		}
	}
}

fn check_for_win(mines: &Vec<Vec<bool>>, flags: &Vec<Vec<bool>>) -> bool {
	for y in 0..mines.len() {
		for x in 0..mines.len() {
			if flags[y][x]
			&& !mines[y][x] {
				return false;
			}
			if mines[y][x]
			&& !flags[y][x] {
				return false;
			}
		}
	}
	return true;
}