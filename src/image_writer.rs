// image_writer.rs
// Creates a graphic frame for each simStep, then
// assembles them into a video at the end of a generation

use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use crate::basic_types::Coord;
use crate::params::Params;
use crate::grid::Grid;
use crate::peeps::Peeps;
use crate::genome_neurons::Genome;
use image::{ImageBuffer, Rgb, RgbImage};
use std::process::Command;
use minifb::{Window, WindowOptions};

pub struct ImageFrameData {
    pub sim_step: u32,
    pub generation: u32,
    pub indiv_locs: Vec<Coord>,
    pub indiv_colors: Vec<u8>,
    pub barrier_locs: Vec<Coord>,
    pub signal_layers: Vec<Vec<Vec<u8>>>,  // [layer][x][y]
}

pub struct ImageWriter {
    pub busy: Arc<Mutex<bool>>,
    dropped_frame_count: Arc<Mutex<u32>>,
    image_list: Arc<Mutex<Vec<ImageFrameData>>>,
    skipped_frames: Arc<Mutex<u32>>,
    pub window: Arc<Mutex<Option<Window>>>,
}

// Generate a color from genome for visualization
fn make_genetic_color(genome: &Genome) -> u8 {
    if genome.is_empty() {
        return 0;
    }
    ((genome.len() & 1) as u8)
        | ((genome[0].source_type) << 1)
        | ((genome[genome.len() - 1].source_type) << 2)
        | ((genome[0].sink_type) << 3)
        | ((genome[genome.len() - 1].sink_type) << 4)
        | ((genome[0].source_num & 1) << 5)
        | ((genome[0].sink_num & 1) << 6)
        | ((genome[genome.len() - 1].source_num & 1) << 7)
}

// Convert genetic color to RGB
fn genetic_color_to_rgb(color: u8) -> (u8, u8, u8) {
    let r = color;
    let g = (color & 0x1f) << 3;
    let b = (color & 7) << 5;
    
    // Prevent very bright colors (hard to see)
    const MAX_COLOR_VAL: u8 = 0xb0;
    const MAX_LUMA_VAL: u8 = 0xb0;
    
    let rgb_to_luma = |r: u8, g: u8, b: u8| -> u8 {
        ((r as u16 + r as u16 + r as u16 + b as u16 + g as u16 + g as u16 + g as u16 + g as u16) / 8) as u8
    };
    
    let mut r = r;
    let mut g = g;
    let mut b = b;
    
    if rgb_to_luma(r, g, b) > MAX_LUMA_VAL {
        if r > MAX_COLOR_VAL {
            r %= MAX_COLOR_VAL;
        }
        if g > MAX_COLOR_VAL {
            g %= MAX_COLOR_VAL;
        }
        if b > MAX_COLOR_VAL {
            b %= MAX_COLOR_VAL;
        }
    }
    
    (r, g, b)
}

// Helper function for alpha blending
fn blend_color(foreground: Rgb<u8>, background: Rgb<u8>, alpha: f32) -> Rgb<u8> {
    let alpha = alpha.max(0.0).min(1.0);
    let r = (foreground[0] as f32 * alpha + background[0] as f32 * (1.0 - alpha)) as u8;
    let g = (foreground[1] as f32 * alpha + background[1] as f32 * (1.0 - alpha)) as u8;
    let b = (foreground[2] as f32 * alpha + background[2] as f32 * (1.0 - alpha)) as u8;
    Rgb([r, g, b])
}


// Helper function to draw filled circle with gradient (for weighted challenges)
fn draw_circle_gradient(
    img: &mut RgbImage,
    params: &Params,
    width: u32,
    height: u32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    color: Rgb<u8>,
    base_opacity: f32,
    _barrier_locs: &[Coord],
) {
    // Draw filled circle with gradient opacity (stronger at center, weaker at edge)
    // Iterate over image pixels, not grid coordinates
    let scale = params.display_scale as f32;
    
    for img_y in 0..height {
        for img_x in 0..width {
            // Convert image coordinates back to grid coordinates
            let grid_x = img_x as f32 / scale;
            let grid_y = (params.size_y as f32 - 1.0) - (img_y as f32 / scale);
            
            // Calculate distance from center
            let dx = grid_x - center_x;
            let dy = grid_y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= radius {
                // Calculate opacity based on distance (1.0 at center, 0.3 at edge)
                let distance_ratio = distance / radius;
                let pixel_opacity = base_opacity * (1.0 - distance_ratio * 0.7); // Fade to 30% at edge
                
                // Draw pixel with gradient opacity
                let background = *img.get_pixel(img_x, img_y);
                let blended = blend_color(color, background, pixel_opacity);
                img.put_pixel(img_x, img_y, blended);
            }
        }
    }
    
    // Also draw the outline for clarity
    draw_circle_outline(img, params, width, height, center_x, center_y, radius, color, base_opacity, _barrier_locs);
}

// Helper function to draw circle outline
fn draw_circle_outline(
    img: &mut RgbImage, 
    params: &Params, 
    width: u32, 
    height: u32,
    center_x: f32, 
    center_y: f32, 
    radius: f32, 
    color: Rgb<u8>,
    opacity: f32,
    _barrier_locs: &[Coord],
) {
    use std::f32::consts::PI;
    let steps = (radius * 2.0 * PI).max(64.0) as usize;
    let step_size = 2.0 * PI / steps as f32;
    
    for i in 0..steps {
        let angle = i as f32 * step_size;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        
        if x >= 0.0 && x < params.size_x as f32 && y >= 0.0 && y < params.size_y as f32 {
            let img_x = (x * params.display_scale as f32) as u32;
            let img_y = ((params.size_y as f32 - 1.0 - y) * params.display_scale as f32) as u32;
            
            // Draw with alpha blending
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = (img_x as i32 + dx) as u32;
                    let py = (img_y as i32 + dy) as u32;
                    if px < width && py < height {
                        let background = *img.get_pixel(px, py);
                        let blended = blend_color(color, background, opacity);
                        img.put_pixel(px, py, blended);
                    }
                }
            }
        }
    }
}

// Helper function to draw vertical line
fn draw_vertical_line(
    img: &mut RgbImage, 
    params: &Params, 
    width: u32, 
    height: u32,
    x: i16, 
    color: Rgb<u8>,
    opacity: f32,
    _barrier_locs: &[Coord],
) {
    let img_x = (x as i32 * params.display_scale as i32) as u32;
    for y in 0..params.size_y as i16 {
        let img_y = ((params.size_y as i32 - 1 - y as i32) * params.display_scale as i32) as u32;
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                let px = (img_x as i32 + dx) as u32;
                let py = (img_y as i32 + dy) as u32;
                if px < width && py < height {
                    let background = *img.get_pixel(px, py);
                    let blended = blend_color(color, background, opacity);
                    img.put_pixel(px, py, blended);
                }
            }
        }
    }
}

// Helper function to draw border outline
fn draw_border_outline(
    img: &mut RgbImage, 
    params: &Params, 
    width: u32, 
    height: u32,
    color: Rgb<u8>,
    opacity: f32,
    _barrier_locs: &[Coord],
) {
    // Top edge (y = size_y - 1 in grid coordinates)
    for x in 0..params.size_x as i16 {
        let img_x = (x as i32 * params.display_scale as i32) as u32;
        for dx in -1..=1 {
            let px = (img_x as i32 + dx) as u32;
            if px < width {
                for py in 0..3.min(height) {
                    let background = *img.get_pixel(px, py);
                    let blended = blend_color(color, background, opacity);
                    img.put_pixel(px, py, blended);
                }
            }
        }
    }
    
    // Bottom edge (y = 0 in grid coordinates)
    for x in 0..params.size_x as i16 {
        let img_x = (x as i32 * params.display_scale as i32) as u32;
        for dx in -1..=1 {
            let px = (img_x as i32 + dx) as u32;
            if px < width {
                for py in (height.saturating_sub(3))..height {
                    let background = *img.get_pixel(px, py);
                    let blended = blend_color(color, background, opacity);
                    img.put_pixel(px, py, blended);
                }
            }
        }
    }
    
    // Left edge (x = 0)
    for y in 0..params.size_y as i16 {
        let img_y = ((params.size_y as i32 - 1 - y as i32) * params.display_scale as i32) as u32;
        for dy in -1..=1 {
            let py = (img_y as i32 + dy) as u32;
            if py < height {
                for px in 0..3.min(width) {
                    let background = *img.get_pixel(px, py);
                    let blended = blend_color(color, background, opacity);
                    img.put_pixel(px, py, blended);
                }
            }
        }
    }
    
    // Right edge (x = size_x - 1)
    for y in 0..params.size_y as i16 {
        let img_y = ((params.size_y as i32 - 1 - y as i32) * params.display_scale as i32) as u32;
        for dy in -1..=1 {
            let py = (img_y as i32 + dy) as u32;
            if py < height {
                for px in (width.saturating_sub(3))..width {
                    let background = *img.get_pixel(px, py);
                    let blended = blend_color(color, background, opacity);
                    img.put_pixel(px, py, blended);
                }
            }
        }
    }
}

// Draw challenge area based on challenge type
fn draw_challenge_area(
    img: &mut RgbImage, 
    params: &Params, 
    width: u32, 
    height: u32,
    barrier_locs: &[Coord],
) {
    use crate::simulator::*;
    
    let challenge_color = Rgb([
        params.display_challenge_area_r,
        params.display_challenge_area_g,
        params.display_challenge_area_b,
    ]);
    let opacity = params.display_challenge_area_opacity;
    
    match params.challenge {
        CHALLENGE_CIRCLE => {
            let center_x = (params.size_x / 4) as f32;
            let center_y = (params.size_y / 4) as f32;
            let radius = params.size_x as f32 / 4.0;
            draw_circle_outline(img, params, width, height, center_x, center_y, radius, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_RIGHT_HALF => {
            let x_line = (params.size_x / 2) as i16;
            draw_vertical_line(img, params, width, height, x_line, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_RIGHT_QUARTER => {
            let x_line = (params.size_x / 2 + params.size_x / 4) as i16;
            draw_vertical_line(img, params, width, height, x_line, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_LEFT_EIGHTH => {
            let x_line = (params.size_x / 8) as i16;
            draw_vertical_line(img, params, width, height, x_line, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_CENTER_WEIGHTED => {
            // Draw gradient-filled circle to show weighted nature
            let center_x = (params.size_x / 2) as f32;
            let center_y = (params.size_y / 2) as f32;
            let radius = params.size_x as f32 / 3.0;
            draw_circle_gradient(img, params, width, height, center_x, center_y, radius, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_CENTER_UNWEIGHTED => {
            // Draw just the outline for unweighted challenge
            let center_x = (params.size_x / 2) as f32;
            let center_y = (params.size_y / 2) as f32;
            let radius = params.size_x as f32 / 3.0;
            draw_circle_outline(img, params, width, height, center_x, center_y, radius, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_CENTER_SPARSE => {
            let center_x = (params.size_x / 2) as f32;
            let center_y = (params.size_y / 2) as f32;
            let outer_radius = params.size_x as f32 / 4.0;
            draw_circle_outline(img, params, width, height, center_x, center_y, outer_radius, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_CORNER => {
            let radius = params.size_x as f32 / 8.0;
            let corners = [
                (0.0, 0.0),
                (0.0, (params.size_y - 1) as f32),
                ((params.size_x - 1) as f32, 0.0),
                ((params.size_x - 1) as f32, (params.size_y - 1) as f32),
            ];
            for (cx, cy) in &corners {
                draw_circle_outline(img, params, width, height, *cx, *cy, radius, challenge_color, opacity, barrier_locs);
            }
        }
        
        CHALLENGE_CORNER_WEIGHTED => {
            let radius = params.size_x as f32 / 4.0;
            let corners = [
                (0.0, 0.0),
                (0.0, (params.size_y - 1) as f32),
                ((params.size_x - 1) as f32, 0.0),
                ((params.size_x - 1) as f32, (params.size_y - 1) as f32),
            ];
            for (cx, cy) in &corners {
                draw_circle_outline(img, params, width, height, *cx, *cy, radius, challenge_color, opacity, barrier_locs);
            }
        }
        
        CHALLENGE_RADIOACTIVE_WALLS | CHALLENGE_AGAINST_ANY_WALL | CHALLENGE_TOUCH_ANY_WALL => {
            draw_border_outline(img, params, width, height, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_EAST_WEST_EIGHTHS => {
            let left_x = (params.size_x / 8) as i16;
            let right_x = (params.size_x - params.size_x / 8) as i16;
            draw_vertical_line(img, params, width, height, left_x, challenge_color, opacity, barrier_locs);
            draw_vertical_line(img, params, width, height, right_x, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_ALTRUISM => {
            let center_x = (params.size_x / 4) as f32;
            let center_y = (params.size_y / 4) as f32;
            let radius = params.size_x as f32 / 4.0;
            draw_circle_outline(img, params, width, height, center_x, center_y, radius, challenge_color, opacity, barrier_locs);
        }
        
        CHALLENGE_ALTRUISM_SACRIFICE => {
            let center_x = (params.size_x - params.size_x / 4) as f32;
            let center_y = (params.size_y - params.size_y / 4) as f32;
            let radius = params.size_x as f32 / 4.0;
            draw_circle_outline(img, params, width, height, center_x, center_y, radius, challenge_color, opacity, barrier_locs);
        }
        
        _ => {
            // Other challenges (STRING, PAIRS, LOCATION_SEQUENCE, MIGRATE_DISTANCE, NEAR_BARRIER)
            // don't have simple geometric boundaries to draw
        }
    }
}

// Convert RGB image to buffer for minifb
// minifb expects pixels in 0xRRGGBB format (24-bit RGB)
fn image_to_buffer(img: &RgbImage) -> Vec<u32> {
    let width = img.width() as usize;
    let height = img.height() as usize;
    let mut buffer = Vec::with_capacity(width * height);
    
    // Iterate row by row (y from 0 to height-1)
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            // minifb expects 0xRRGGBB format (24-bit RGB, big-endian)
            // pixel[0] = R, pixel[1] = G, pixel[2] = B
            let rgb = ((pixel[0] as u32) << 16)
                    | ((pixel[1] as u32) << 8)
                    | (pixel[2] as u32);
            buffer.push(rgb);
        }
    }
    
    buffer
}

// Save one frame immediately and optionally display it
fn save_one_frame_immed(data: &ImageFrameData, params: &Params, window: &Arc<Mutex<Option<Window>>>) {
    let width = (params.size_x as u32 * params.display_scale) as u32;
    let height = (params.size_y as u32 * params.display_scale) as u32;
    
    // Create white background image
    let mut img: RgbImage = ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }
    
    // Draw barriers (gray)
    let barrier_color = Rgb([0x88, 0x88, 0x88]);
    for loc in &data.barrier_locs {
        let x1 = (loc.x * params.display_scale as i16 - params.display_scale as i16 / 2).max(0) as u32;
        let y1 = ((params.size_y as i16 - loc.y - 1) * params.display_scale as i16 - params.display_scale as i16 / 2).max(0) as u32;
        let x2 = ((loc.x + 1) * params.display_scale as i16).min(width as i16) as u32;
        let y2 = ((params.size_y as i16 - loc.y) * params.display_scale as i16).min(height as i16) as u32;
        
        for y in y1..y2 {
            for x in x1..x2 {
                if x < width && y < height {
                    img.put_pixel(x, y, barrier_color);
                }
            }
        }
    }
    
    // Draw challenge area highlight if enabled
    if params.display_challenge_area {
        draw_challenge_area(&mut img, params, width, height, &data.barrier_locs);
    }
    
    // Draw agents
    for (i, loc) in data.indiv_locs.iter().enumerate() {
        if i < data.indiv_colors.len() {
            let color = genetic_color_to_rgb(data.indiv_colors[i]);
            let agent_color = Rgb([color.0, color.1, color.2]);
            
            // Convert grid coordinates to image coordinates
            // Grid: (0,0) is bottom-left, x increases right, y increases up
            // Image: (0,0) is top-left, x increases right, y increases down
            // So we flip Y: image_y = (size_y - 1 - grid_y) * scale
            let center_x = (loc.x as i32 * params.display_scale as i32) as u32;
            let center_y = ((params.size_y as i32 - 1 - loc.y as i32) * params.display_scale as i32) as u32;
            let radius = params.agent_size as u32;
            
            // Draw circle for agent
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    if dx * dx + dy * dy <= (radius * radius) as i32 {
                        let x = (center_x as i32 + dx) as u32;
                        let y = (center_y as i32 + dy) as u32;
                        if x < width && y < height {
                            img.put_pixel(x, y, agent_color);
                        }
                    }
                }
            }
        }
    }
    
    // Display frame in window if enabled
    if params.display_enabled {
        if let Ok(mut window_opt) = window.lock() {
            if window_opt.is_none() {
                // Create window on first frame
                let window_title = format!("BiosimRust - Gen {} Step {}", data.generation, data.sim_step);
                eprintln!("DEBUG: Creating display window: {}x{} with title: {}", width, height, window_title);
                match Window::new(
                    &window_title,
                    width as usize,
                    height as usize,
                    WindowOptions {
                        resize: true,
                        scale: minifb::Scale::FitScreen,
                        borderless: false,
                        title: true,
                        ..WindowOptions::default()
                    },
                ) {
                    Ok(win) => {
                        eprintln!("DEBUG: Display window created successfully: {}x{}", width, height);
                        eprintln!("DEBUG: Window is_open: {}", win.is_open());
                        *window_opt = Some(win);
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to create display window: {}", e);
                        eprintln!("Window creation failed - continuing without display");
                        eprintln!("This might be a platform-specific issue. Try running with displayenabled = false");
                        return;
                    }
                }
            }
            
            if let Some(ref mut win) = *window_opt {
                let buffer = image_to_buffer(&img);
                        let title = format!("BiosimRust - Gen {} Step {}", data.generation, data.sim_step);
                win.set_title(&title);
                
                // Update window with new frame
                if !win.is_open() {
                    *window_opt = None;
                } else {
                    // Update the window buffer
                    if buffer.len() == (width as usize * height as usize) {
                        let _ = win.update_with_buffer(&buffer, width as usize, height as usize);
                    }
                }
            }
        }
    }
    
    // Save frame as PNG
    let image_dir = Path::new(&params.image_dir);
    if !image_dir.exists() {
        let _ = fs::create_dir_all(image_dir);
    }
    
    let filename = format!("{}/frame-{:06}-{:06}.png", 
        params.image_dir, data.generation, data.sim_step);
    
    if let Err(e) = img.save(&filename) {
        eprintln!("Failed to save frame {}: {}", filename, e);
    }
}

impl ImageWriter {
    pub fn new() -> Self {
        ImageWriter {
            busy: Arc::new(Mutex::new(false)),
            dropped_frame_count: Arc::new(Mutex::new(0)),
            image_list: Arc::new(Mutex::new(Vec::new())),
            skipped_frames: Arc::new(Mutex::new(0)),
            window: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_new_generation(&mut self) {
        if let Ok(mut list) = self.image_list.lock() {
            list.clear();
        }
        if let Ok(mut skipped) = self.skipped_frames.lock() {
            *skipped = 0;
        }
    }

    pub fn save_video_frame(
        &mut self,
        sim_step: u32,
        generation: u32,
        grid: &Grid,
        peeps: &Peeps,
        params: &Params,
    ) -> bool {
        if let Ok(mut busy_flag) = self.busy.lock() {
            if !*busy_flag {
                *busy_flag = true;
                
                // Cache data for frame generation
                let mut data = ImageFrameData {
                    sim_step,
                    generation,
                    indiv_locs: Vec::new(),
                    indiv_colors: Vec::new(),
                    barrier_locs: Vec::new(),
                    signal_layers: Vec::new(),
                };
                
                // Collect individual locations and colors
                for index in 1..=params.population {
                    if let Some(indiv) = peeps.get(index as u16) {
                        if indiv.alive {
                            data.indiv_locs.push(indiv.loc);
                            data.indiv_colors.push(make_genetic_color(&indiv.genome));
                        }
                    }
                }
                
                // Collect barrier locations
                for loc in grid.get_barrier_locations() {
                    data.barrier_locs.push(*loc);
                }
                
                // Save frame immediately (synchronous for now)
                save_one_frame_immed(&data, params, &self.window);
                
                // Store frame data for video generation
                if let Ok(mut list) = self.image_list.lock() {
                    list.push(data);
                }
                
                *busy_flag = false;
                return true;
            } else {
                // Image saver is busy, drop a frame
                if let Ok(mut count) = self.dropped_frame_count.lock() {
                    *count += 1;
                }
                return false;
            }
        }
        false
    }

    pub fn save_video_frame_sync(
        &mut self,
        sim_step: u32,
        generation: u32,
        grid: &Grid,
        peeps: &Peeps,
        params: &Params,
    ) -> bool {
        // Synchronous version, always returns true
        let mut data = ImageFrameData {
            sim_step,
            generation,
            indiv_locs: Vec::new(),
            indiv_colors: Vec::new(),
            barrier_locs: Vec::new(),
            signal_layers: Vec::new(),
        };
        
        // Collect individual locations and colors
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                if indiv.alive {
                    data.indiv_locs.push(indiv.loc);
                    data.indiv_colors.push(make_genetic_color(&indiv.genome));
                }
            }
        }
        
        // Collect barrier locations
        for loc in grid.get_barrier_locations() {
            data.barrier_locs.push(*loc);
        }
        
        // Save frame immediately and display
        save_one_frame_immed(&data, params, &self.window);
        
        // Store frame data for video generation
        if let Ok(mut list) = self.image_list.lock() {
            list.push(data);
        }
        
        true
    }

    pub fn save_generation_video(&mut self, generation: u32, params: &Params) {
        let image_dir = Path::new(&params.image_dir);
        if !image_dir.exists() {
            return;
        }
        
        // Count frames for this generation
        let frame_count = if let Ok(list) = self.image_list.lock() {
            list.iter()
                .filter(|frame| frame.generation == generation)
                .count()
        } else {
            return;
        };
        
        if frame_count == 0 {
            return;
        }
        
        // Try to use ffmpeg to create video from frames using pattern matching
        let video_filename = format!("{}/gen-{:06}.mp4", params.image_dir, generation);
        
        // Use ffmpeg with pattern matching to read frames
        // ffmpeg -framerate <rate> -i pattern -c:v libx264 -pix_fmt yuv420p output.mp4
        let framerate_str = params.video_framerate.to_string();
        let ffmpeg_result = Command::new("ffmpeg")
            .args(&[
                "-y",  // Overwrite output file
                "-framerate", &framerate_str,
                "-i", &format!("{}/frame-{:06}-%06d.png", params.image_dir, generation),
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-r", &framerate_str,
                &video_filename,
            ])
            .output();
        
        match ffmpeg_result {
            Ok(output) => {
                if !output.status.success() {
                    // Try alternative: use glob pattern (works on some systems)
                    let alt_result = Command::new("ffmpeg")
                        .args(&[
                            "-y",
                            "-framerate", &framerate_str,
                            "-pattern_type", "glob",
                            "-i", &format!("{}/frame-{:06}-*.png", params.image_dir, generation),
                            "-c:v", "libx264",
                            "-pix_fmt", "yuv420p",
                            "-r", &framerate_str,
                            &video_filename,
                        ])
                        .output();
                    
                    match alt_result {
                        Ok(alt_output) => {
                            if alt_output.status.success() {
                                println!("Video saved: {}", video_filename);
                            } else {
                                eprintln!("ffmpeg failed. Video frames saved as PNG files in {}", params.image_dir);
                                eprintln!("You can manually create a video using:");
                                eprintln!("  ffmpeg -framerate {} -i {}/frame-{:06}-%06d.png -c:v libx264 -pix_fmt yuv420p {}/gen-{:06}.mp4",
                                    params.video_framerate, params.image_dir, generation, params.image_dir, generation);
                            }
                        }
                        Err(_) => {
                            eprintln!("ffmpeg not found. Video frames saved as PNG files in {}", params.image_dir);
                            eprintln!("Install ffmpeg to create videos automatically.");
                        }
                    }
                } else {
                    println!("Video saved: {}", video_filename);
                }
            }
            Err(_) => {
                eprintln!("ffmpeg not found. Video frames saved as PNG files in {}", params.image_dir);
                eprintln!("Install ffmpeg and run:");
                eprintln!("  ffmpeg -framerate {} -i {}/frame-{:06}-%06d.png -c:v libx264 -pix_fmt yuv420p {}/gen-{:06}.mp4",
                    params.video_framerate, params.image_dir, generation, params.image_dir, generation);
            }
        }
        
        if let Ok(skipped) = self.skipped_frames.lock() {
            if *skipped > 0 {
                println!("Video skipped {} frames", skipped);
            }
        }
        
        self.start_new_generation();
    }

    pub fn abort(&mut self) {
        if let Ok(mut busy_flag) = self.busy.lock() {
            *busy_flag = true;
        }
    }

    pub fn dropped_frame_count(&self) -> u32 {
        if let Ok(count) = self.dropped_frame_count.lock() {
            *count
        } else {
            0
        }
    }
}

impl Default for ImageWriter {
    fn default() -> Self {
        Self::new()
    }
}
