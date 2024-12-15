use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures::StreamExt;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawParam, Text, TextFragment};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, ContextBuilder, GameResult};
use image::{ImageBuffer, Rgba};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct AreaMap {
    height: usize,
    width: usize,
    state: Vec<Actor>,
}

#[derive(Debug, Clone)]
struct Actor {
    position: (usize, usize),
    velocity: (i32, i32),
}

struct GameState {
    map: AreaMap,
    cell_size: f32,
    tick: usize,
    auto_tick: bool,
    tick_delay: Duration,
    last_tick: Instant,
    tree_check_sender: Sender<(usize, Vec<u8>)>,
    tree_check_receiver: Receiver<(usize, String)>,
    tree_detections: Vec<(usize, String)>,
    last_check_tick: usize,
    pending_checks: HashSet<usize>,
}

impl GameState {
    fn new(map: AreaMap) -> Self {
        let (img_tx, img_rx): (Sender<(usize, Vec<u8>)>, Receiver<(usize, Vec<u8>)>) = channel();
        let (resp_tx, resp_rx): (Sender<(usize, String)>, Receiver<(usize, String)>) = channel();

        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            loop {
                if let Ok((tick, image_data)) = img_rx.recv() {
                    let resp_tx = resp_tx.clone();

                    runtime.block_on(async move {
                        let client = Client::new();
                        let base64_image = STANDARD.encode(&image_data);

                        let request = json!({
                            "model": "llama3.2-vision:11b",
                            "stream": true,
                            "messages": [{
                                "role": "user",
                                "content": "Does this image contain a Christmas Tree shape? Only answer Yes or No.",
                                "images": [base64_image]
                            }]
                        });

                        match client
                            .post("http://100.124.10.24:11434/api/chat")
                            .header("Content-Type", "application/json")
                            .body(request.to_string())
                            .send()
                            .await
                        {
                            Ok(response) => {
                                let mut full_response = String::new();
                                let mut stream = response.bytes_stream();

                                while let Some(chunk_result) = stream.next().await {
                                    if let Ok(chunk) = chunk_result {
                                        if let Ok(chunk_str) = String::from_utf8(chunk.to_vec()) {
                                            if let Ok(json) = serde_json::from_str::<Value>(&chunk_str) {
                                                if let Some(content) = json["message"]["content"].as_str() {
                                                    full_response.push_str(content);
                                                    // If this is the final message (done=true)
                                                    if json["done"].as_bool().unwrap_or(false) {
                                                        let final_result = full_response.to_lowercase();

                                                        if final_result.contains("yes") {
                                                            if let Ok(img) = image::load_from_memory(&image_data) {
                                                                let filename = format!("./out/tree_detected_tick_{:06}.png", tick);
                                                                if let Err(e) = img.save(&filename) {
                                                                    println!("Failed to save detected tree frame {}: {}", filename, e);
                                                                } else {
                                                                    println!("Saved tree detection image: {}", filename);
                                                                }
                                                            }
                                                        }
                                                        // Always send response to remove from pending
                                                        resp_tx.send((tick, final_result)).ok();
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Request failed for tick {}: {}", tick, e);
                                // Send error response to remove from pending
                                resp_tx.send((tick, "error".to_string())).ok();
                            }
                        }
                    });
                }
            }
        });

        GameState {
            map,
            cell_size: 20.0,
            tick: 0,
            auto_tick: false,
            tick_delay: Duration::from_millis(5),
            last_tick: Instant::now(),
            tree_check_sender: img_tx,
            tree_check_receiver: resp_rx,
            tree_detections: Vec::new(),
            last_check_tick: 0,
            pending_checks: HashSet::new(),
        }
    }

    fn capture_screen_for_model(&self, _ctx: &mut Context) -> Option<Vec<u8>> {
        let scale = 2.0;
        let width = (self.map.width as f32 * scale) as u32;
        let height = (self.map.height as f32 * scale) as u32;
        let mut img_buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        // Fill with black
        for pixel in img_buf.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 255]);
        }
        let mut drawn_count = 0;

        for actor in &self.map.state {
            let x = (actor.position.0 as f32 * scale) as u32;
            let y = (actor.position.1 as f32 * scale) as u32;
            if x < width && y < height {
                img_buf.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                drawn_count += 1;
            } else {
                println!("Warning: Actor position out of bounds: ({}, {})", x, y);
            }
        }

        if drawn_count != self.map.state.len() {
            println!("WARNING: Not all actors were drawn!");
        }

        // Convert to PNG
        let mut png_data = Vec::new();
        if let Ok(_) = img_buf.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        ) {
            Some(png_data)
        } else {
            println!("Failed to create PNG");
            None
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while let Ok((tick, response)) = self.tree_check_receiver.try_recv() {
            self.pending_checks.remove(&tick);
            println!("Response: {:?}", response);
            if response.to_lowercase().contains("yes") {
                self.tree_detections.push((tick, response));
                println!("Tree detected at tick {}!", tick);
            }
        }

        let should_send_frame = self.auto_tick && self.last_tick.elapsed() >= self.tick_delay;

        if should_send_frame {
            if let Some(image_data) = self.capture_screen_for_model(ctx) {
                if self.tree_check_sender.send((self.tick, image_data)).is_ok() {
                    self.last_check_tick = self.tick;
                    self.pending_checks.insert(self.tick);
                }
            }

            self.map.update();
            self.tick += 1;
            self.last_tick = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let symbol = if self.map.state.iter().any(|actor| actor.position == (x, y)) {
                    "🤖"
                } else {
                    "·"
                };

                let text = Text::new(TextFragment::new(symbol));
                let dest = Vec2::new(x as f32 * self.cell_size, y as f32 * self.cell_size);
                canvas.draw(&text, DrawParam::default().dest(dest).color(Color::WHITE));
            }
        }
        let detection_status = if !self.pending_checks.is_empty() {
            format!("Processing {} requests.", self.pending_checks.len())
        } else {
            "Idle".to_string()
        };

        let info_text = Text::new(
            TextFragment::new(format!(
                "Tick: {}\n\
             Trees Found: {}\n\
             Last Detection: {}\n\
             Detection Status: {}\n\
             Processing Lag: {} ticks\n\
             Space: Toggle auto-tick\n\
             Left/Right: Step\n\
             +/-: Speed\n\
             Esc: Exit",
                self.tick,
                self.tree_detections.len(),
                self.tree_detections
                    .last()
                    .map_or("None".to_string(), |(tick, _)| format!("Tick {}", tick)),
                detection_status,
                self.tick.saturating_sub(self.last_check_tick)
            ))
            .scale(32.0),
        );

        canvas.draw(
            &info_text,
            DrawParam::default()
                .dest(Vec2::new(10.0, 10.0))
                .color(Color::GREEN),
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::Space => {
                    self.auto_tick = !self.auto_tick;
                }
                KeyCode::Right => {
                    if !self.auto_tick {
                        if let Some(image_data) = self.capture_screen_for_model(ctx) {
                            if self.tree_check_sender.send((self.tick, image_data)).is_ok() {
                                self.last_check_tick = self.tick;
                                self.pending_checks.insert(self.tick);
                            }
                        }

                        self.map.update();
                        self.tick += 1;
                    }
                }
                KeyCode::NumpadAdd | KeyCode::Plus => {
                    self.tick_delay = self.tick_delay.saturating_sub(Duration::from_millis(50));
                }
                KeyCode::NumpadSubtract | KeyCode::Minus => {
                    self.tick_delay += Duration::from_millis(50);
                }
                _ => (),
            }
        }
        Ok(())
    }
}

impl AreaMap {
    fn new(height: usize, width: usize) -> Self {
        AreaMap {
            height,
            width,
            state: Vec::new(),
        }
    }

    fn load_actors(&mut self, input: &[String]) {
        for line in input {
            let actor = Actor::parse(line);
            self.state.push(actor);
        }
    }

    fn update(&mut self) {
        for actor in &mut self.state {
            actor.update(self.width, self.height);
        }
    }

    fn display(&self) -> GameResult {
        let (ctx, event_loop) = ContextBuilder::new("grid_display", "author")
            .window_setup(ggez::conf::WindowSetup::default().title("Day 14"))
            .window_mode(
                ggez::conf::WindowMode::default()
                    .dimensions((self.width as f32) * 20.0, (self.height as f32) * 20.0),
            )
            .build()?;
        let state = GameState::new(self.clone());
        event::run(ctx, event_loop, state)
    }

    fn find_pattern(&mut self) -> usize {
        let mut ticks = 0;
        let total_actors = self.state.len();
        let threshold = (total_actors * 9) / 10;

        loop {
            let mut adjacent_count = 0;

            for i in 0..self.state.len() {
                for j in (i + 1)..self.state.len() {
                    let (x1, y1) = self.state[i].position;
                    let (x2, y2) = self.state[j].position;

                    if (x1.abs_diff(x2) <= 1) && (y1.abs_diff(y2) <= 1) {
                        adjacent_count += 1;
                    }
                }
            }

            if adjacent_count > threshold {
                return ticks;
            }

            self.update();
            ticks += 1;

            if ticks > 1_000_000 {
                println!("Hit safety limit without finding pattern");
                return ticks;
            }
        }
    }
}

impl Actor {
    fn new(position: (usize, usize), velocity: (i32, i32)) -> Self {
        Actor { position, velocity }
    }

    fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let position_string = parts[0].trim_start_matches("p=");
        let pos: Vec<usize> = position_string
            .split(',')
            .map(|x| x.parse().unwrap())
            .collect();
        let velocity_string = parts[1].trim_start_matches("v=");
        let vel: Vec<i32> = velocity_string
            .split(',')
            .map(|x| x.parse().unwrap())
            .collect();
        Actor::new((pos[0], pos[1]), (vel[0], vel[1]))
    }

    fn update(&mut self, width: usize, height: usize) {
        let x = self.position.0 as i32;
        let y = self.position.1 as i32;
        let new_x = (x + self.velocity.0).rem_euclid(width as i32) as usize;
        let new_y = (y + self.velocity.1).rem_euclid(height as i32) as usize;
        self.position = (new_x, new_y);
    }
}

fn main() {
    let mut map = AreaMap::new(103, 101);
    let input = std::fs::read_to_string("input.txt")
        .unwrap()
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    map.load_actors(&input);
    //map.display().unwrap();
    let ticks = map.find_pattern();
    println!("Pattern found after {} ticks", ticks);
}
