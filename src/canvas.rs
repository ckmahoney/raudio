    use image::{ImageBuffer, Rgb};
    use rusttype::{Font, Scale, point, PositionedGlyph};

    pub struct CanvasConfig {
        pub height: u32,
        pub width: u32,
        pub title_height: u32,
        pub subtitle_height: u32,
    }

    pub struct TypographyConfig {
        pub title_font_size: f32,
        pub subtitle_font_size: f32,
        pub paragraph_font_size: f32,
        pub title_font: Font<'static>,
        pub subtitle_font: Font<'static>,
        pub paragraph_font: Font<'static>,
    }

    pub struct ColorConfig {
        pub axis_color: Rgb<u8>,
        pub background_color: Rgb<u8>,
        pub use_single_color: bool,
        pub waveform_colors: Vec<Rgb<u8>>,
    }

    pub struct AxisConfig {
        pub start: f32,
        pub end: f32, // New field to dynamically set end point
        pub step: f32,
    }
    fn draw_axes(
        imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
        canvas_config: &CanvasConfig,
        axis_color: &Rgb<u8>,
        x_axis_config: &AxisConfig,
        y_axis_config: &AxisConfig,
    ) {
        let tick_length = 10;
        let x_axis_y = canvas_config.height as f32 / 2.0; 
        let y_axis_x = canvas_config.width as f32 / 3.0; 
        
        for x in 0..canvas_config.width {
            imgbuf.put_pixel(x, x_axis_y as u32, *axis_color);
        }
    
        for y in 0..canvas_config.height {
            imgbuf.put_pixel(y_axis_x as u32, y, *axis_color);
        }
    
        let x_range = x_axis_config.end - x_axis_config.start;
        let x_tick_count = (x_range / x_axis_config.step).ceil() as i32;
        for i in 0..=x_tick_count {
            let x = y_axis_x + i as f32 * (canvas_config.width as f32 / x_tick_count as f32);
            if x >= 0.0 && x < canvas_config.width as f32 {
                for offset in -(tick_length / 2)..=(tick_length / 2) {
                    imgbuf.put_pixel(x as u32, (x_axis_y as i32 + offset) as u32, *axis_color);
                }
            }
        }
    
        let y_range = y_axis_config.end - y_axis_config.start;
        let y_tick_count = (y_range / y_axis_config.step).ceil() as i32;
        for i in 0..=y_tick_count {
            let y = x_axis_y - i as f32 * (canvas_config.height as f32 / y_tick_count as f32);
            if y < canvas_config.height as f32 {
                for offset in -(tick_length / 2)..=(tick_length / 2) {
                    imgbuf.put_pixel((y_axis_x as i32 + offset) as u32, y as u32, *axis_color);
                }
            }
        }
    }
    
    
    
    fn draw_text(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, text: &str, font: &Font, font_size: f32, x_start: u32, y_start: u32, color: &Rgb<u8>) {
        let scale = Scale { x: font_size, y: font_size };
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);
        let glyphs: Vec<_> = font.layout(text, scale, offset).collect();
        let glyphs_width: i32 = glyphs.iter().map(|g| g.pixel_bounding_box().map_or(0, |bb| bb.width())).sum();
        let width_center = ((x_start as i32) - (glyphs_width / 2)).max(0);
        let height_center = y_start + (v_metrics.ascent - v_metrics.descent) as u32 / 2;
    
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    if v > 0.0 {
                        let x = x as i32 + bounding_box.min.x + width_center;
                        let y = y as i32 + bounding_box.min.y + height_center as i32;
    
                        if x >= 0 && y >= 0 && (x as u32) < imgbuf.width() && (y as u32) < imgbuf.height() {
                            let alpha = (v * 255.0) as u8;
                            let text_color = Rgb([color[0], color[1], color[2]]);
                            let blended_pixel = blend(text_color, alpha, *imgbuf.get_pixel(x as u32, y as u32), 255);
                            imgbuf.put_pixel(x as u32, y as u32, blended_pixel);
                        }
                    }
                });
            }
        }
    }

    fn draw_title(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, title: &str, canvas_config: &CanvasConfig, typography_config: &TypographyConfig, axis_color: &Rgb<u8>) {
        draw_text(imgbuf, title, &typography_config.title_font, typography_config.title_font_size, canvas_config.width / 2, 0, axis_color);
    }

    fn draw_subtitle(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, subtitle: &str, canvas_config: &CanvasConfig, typography_config: &TypographyConfig, axis_color: &Rgb<u8>) {
        draw_text(imgbuf, subtitle, &typography_config.subtitle_font, typography_config.subtitle_font_size, canvas_config.width / 2, canvas_config.title_height + 10, axis_color);
    }


    fn blend(fg_color: Rgb<u8>, fg_alpha: u8, bg_color: Rgb<u8>, bg_alpha: u8) -> Rgb<u8> {
        let fg_alpha = fg_alpha as f32 / 255.0;
        let bg_alpha = bg_alpha as f32 / 255.0;
        let r = fg_color[0] as f32 * fg_alpha + bg_color[0] as f32 * bg_alpha * (1.0 - fg_alpha);
        let g = fg_color[1] as f32 * fg_alpha + bg_color[1] as f32 * bg_alpha * (1.0 - fg_alpha);
        let b = fg_color[2] as f32 * fg_alpha + bg_color[2] as f32 * bg_alpha * (1.0 - fg_alpha);
        Rgb([r as u8, g as u8, b as u8])
    }

    pub fn render_waveforms<F>(
        functions: Vec<(F, f32)>, 
        sample_rate: u32, 
        duration: f32, 
        title: &str, 
        subtitle: &str, 
        canvas_config: &CanvasConfig, 
        typography_config: &TypographyConfig, 
        color_config: &ColorConfig
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> where
        F: Fn(f32, f32) -> f32,
    {
        let mut imgbuf = ImageBuffer::from_pixel(
            canvas_config.width, 
            canvas_config.height + canvas_config.title_height + canvas_config.subtitle_height, 
            color_config.background_color
        );
    
        let x_axis_config = AxisConfig {
            start: -1.0,
            end: duration, // Set dynamically
            step: 1.0,
        };
    
        let y_axis_config = AxisConfig {
            start: -1.0,
            end: 1.0,
            step: 0.5,
        };
    
        draw_axes(&mut imgbuf, canvas_config, &color_config.axis_color, &x_axis_config, &y_axis_config);
        draw_title(&mut imgbuf, title, canvas_config, typography_config, &color_config.axis_color);
        draw_subtitle(&mut imgbuf, subtitle, canvas_config, typography_config, &color_config.axis_color);
    
        for (index, (function, frequency)) in functions.into_iter().enumerate() {
            let color = if color_config.use_single_color {
                color_config.waveform_colors[0]
            } else {
                color_config.waveform_colors[index % color_config.waveform_colors.len()]
            };
            let samples = generate_samples(function, sample_rate, duration, frequency);
            draw_waveform(&mut imgbuf, &samples, canvas_config, &color);
        }
    
        imgbuf
    }

    fn generate_samples<F>(function: F, sample_rate: u32, duration: f32, frequency: f32) -> Vec<f32>
where
    F: Fn(f32, f32) -> f32,
{
    let total_samples = (sample_rate as f32 * duration) as usize;
    let sample_period = 1.0 / sample_rate as f32;
    (0..total_samples-1).map(|i| function(i as f32 * sample_period, frequency)).collect()
}

fn draw_waveform(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, samples: &[f32], canvas_config: &CanvasConfig, waveform_color: &Rgb<u8>) {
    let mid_height = canvas_config.height / 2;
    let scale = mid_height as f32 - 10.0; // Padding for visualization
    let x_axis_start = canvas_config.width / 4; // Start the waveform at -1
    let plot_width = canvas_config.width as f32 - x_axis_start as f32;

    for (x, &sample) in samples.iter().enumerate() {
        let y_pos = (mid_height as f32 - sample * scale) as u32 + canvas_config.title_height;
        let x_pos = x as f32 * plot_width / samples.len() as f32 + x_axis_start as f32;

        if x_pos >= x_axis_start as f32 && x_pos < canvas_config.width as f32 {
            imgbuf.put_pixel(x_pos as u32, y_pos, *waveform_color);
            if x < samples.len() - 1 {
                let next_x_pos = (x as f32 + 1.0) * plot_width / samples.len() as f32 + x_axis_start as f32;
                let next_y_pos = (mid_height as f32 - samples[x + 1] * scale) as u32 + canvas_config.title_height;
                draw_line(imgbuf, x_pos as u32, y_pos, next_x_pos as u32, next_y_pos, waveform_color);
            }
        }
    }
}

fn draw_line(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x0: u32, y0: u32, x1: u32, y1: u32, color: &Rgb<u8>) {
    let (mut x, mut y) = (x0 as i32, y0 as i32);
    let (x1, y1) = (x1 as i32, y1 as i32);
    let dx = i32::abs(x1 - x);
    let dy = i32::abs(y1 - y);
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if x >= 0 && x < imgbuf.width() as i32 && y >= 0 && y < imgbuf.height() as i32 {
            imgbuf.put_pixel(x as u32, y as u32, *color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = err * 2;
        if e2 > -dx { err -= dy; x += sx; }
        if e2 < dy { err += dx; y += sy; }
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;
    use rusttype::Font;
    use crate::canvas::{render_waveforms, CanvasConfig, TypographyConfig, ColorConfig};
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::f32::consts::PI;
    use std::path::{Path};

    fn read_file_to_bytes(file_path: &str) -> Vec<u8> {
        let mut file = File::open(file_path).expect("Unable to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Unable to read file");
        buffer
    }

    fn sine_wave(t: f32, frequency: f32) -> f32 {
        (t * frequency * 2.0 * PI).sin()
    }

    fn triangle_wave(t: f32, frequency: f32) -> f32 {
        let t_scaled = t * frequency;
        2.0 * (t_scaled - t_scaled.floor()) - 1.0
    }

    fn square_wave(t: f32, frequency: f32) -> f32 {
        let t_scaled = t * frequency;
        if (t_scaled - t_scaled.floor()) < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    fn composite_wave(t: f32, frequency: f32) -> f32 {
        // You can create a composite waveform by summing different waveforms here
        let sine = sine_wave(t, 1.0);
        let triangle = triangle_wave(t, 2.0);
        let square = square_wave(t, 3.0);
    
        // Normalize the composite waveform to fit in the range [-1.0, 1.0]
        let sum = sine + triangle + square;
        sum / 3.0 // Divide by the number of waveforms for normalization
    }

    #[test]
    fn test_waveforms() {
        let tests = vec![
            ("Sine Wave", "A basic sine wave", sine_wave as fn(f32, f32) -> f32, 1.0, "sine_wave.png"),
            ("Triangle Wave", "A basic triangle wave", triangle_wave as fn(f32, f32) -> f32, 2.0, "triangle_wave.png"),
            ("Square Wave", "A basic square wave", square_wave as fn(f32, f32) -> f32, 3.0, "square_wave.png"),
            ("Composite Wave", "A normalized composite wave", composite_wave as fn(f32, f32) -> f32, 1.0, "composite_wave.png"), 
            // Add more tuples for other waveform tests (sawtooth, pulse, composite) with their respective frequencies...
        ];
        let output_dir = Path::new("dev-image");
        let title_font_data = read_file_to_bytes("assets/Antology.ttf");
        let subtitle_font_data = read_file_to_bytes("assets/monof56.ttf");
        let paragraph_font_data = read_file_to_bytes("assets/monof56.ttf");

        let title_font = Font::try_from_vec(title_font_data).expect("Error loading title font");
        let subtitle_font = Font::try_from_vec(subtitle_font_data).expect("Error loading subtitle font");
        let paragraph_font = Font::try_from_vec(paragraph_font_data).expect("Error loading paragraph font");

        let canvas_config = CanvasConfig { height: 720, width: 1280, title_height: 40, subtitle_height: 30 };
        let typography_config = TypographyConfig { 
            title_font_size: 64.0, 
            subtitle_font_size: 24.0, 
            paragraph_font_size: 18.0,
            title_font,
            subtitle_font,
            paragraph_font,
        };
        let color_config = ColorConfig { axis_color: Rgb([0, 0, 0]), background_color: Rgb([255, 255, 255]), use_single_color: true, waveform_colors: vec![Rgb([255, 0, 0])] };

        // Ensure the output directory exists
        std::fs::create_dir_all(&output_dir).expect("Failed to create directory");

        for (title, description, generator, frequency, file_name) in tests {
            let image = render_waveforms(
                vec![(generator, frequency)],
                44100,
                2.0,
                title,
                description,
                &canvas_config,
                &typography_config,
                &color_config,
            );

            let file_path = output_dir.join(file_name);
            image.save(file_path).unwrap();
        }
    }
}
