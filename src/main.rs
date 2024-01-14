use std::time::Instant;

use sdl2;
use clap::Parser;
use ctrlc;
// use winit;
use msgbox;

mod windows_quirks;
mod fps_capper;

#[derive(Debug)]
struct CustomError {
    msg: String
}
impl std::fmt::Display for CustomError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg).unwrap();
        Ok(())
    }
}
impl std::error::Error for CustomError{}

#[derive(clap::Parser)]
#[command(author = "timelessnesses", about = "Nothing")]
struct Cli {
    /// Frame limiting
    #[arg(short, long)]
    fps: Option<i64>,
    /// List GPU renderers (for the SELECTED_GPU_RENDERER arg)
    #[arg(short, long)]
    list_gpu_renderers: bool,
    /// Select your own renderer if you want to
    #[arg(short, long)]
    selected_gpu_renderer: Option<usize>,
}

fn report_error(error: impl std::error::Error, title: &str) {
    msgbox::create(title, &error.to_string(), msgbox::IconType::Error).unwrap();
    panic!()
}

const SEGOE: &[u8; 509920] = include_bytes!("assets/segoe-ui.ttf");

fn main() {
    windows_quirks::windows_hide_console(); // fuck you
    let parsed = Cli::parse();
    if parsed.list_gpu_renderers {
        for (i, item) in sdl2::render::drivers().enumerate() {
            println!(
                "Renderer #{}:\n   Name: {}\n  Flags: {}",
                i + 1,
                item.name,
                item.flags
            )
        }
        return;
    }

    let fl = match parsed.fps {
        Some(f) => f,
        None => 60,
    };
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();

    let window = {
        match video
        .window("Nothing", 800, 600)
        .position_centered()
        .allow_highdpi()
        // .resizable()
        .metal_view()
        .opengl()
        .fullscreen()
        .fullscreen_desktop()
        .build() {
            Ok(w) => Some(w),
            Err(e) => {
                report_error(e, "Failed to initialize SDL2 window");
                None
            }
        }
    }.unwrap();


    let mut running = true;
    let mut canvas = match parsed.selected_gpu_renderer {
        Some(i) => match window.into_canvas().index((i - 1) as u32).build() {
            Ok(c) => Some(c),
            Err(e) => {
                report_error(e, "Failed to build Canvas");
                None
            }
        },
        None => {
            match window.into_canvas().build() {
                Ok(c) => Some(c),
                Err(e) => {
                    report_error(e, "Failed to build default Canvas");
                    None
                }
            }
        },
    }.unwrap();
    let mut event_pump = {
        match ctx.event_pump() {
            Ok(e) => Some(e),
            Err(e) => {
                report_error(CustomError {
                    msg: e
                }, "Failed to initialize EventPump");
                None
            }
        }
    }.unwrap();
    let mut capper = fps_capper::FpsLimiter::new(fl as u32);
    let _ = ctrlc::set_handler(move || {
        running = !running;
    });

    // let mut draw_intro_text = false;
    let font_ctx = match sdl2::ttf::init() {
        Ok(f) => Some(f),
        Err(e) => {
            report_error(e, "Failed to initialize TTF Context");
            None
        }
    }.unwrap();
    let segoe_font = match font_ctx.load_font_from_rwops(sdl2::rwops::RWops::from_bytes(SEGOE).unwrap(), 45) {
        Ok(f) => Some(f),
        Err(e) => {
            report_error(CustomError {
                msg: e
            }, "Failed to load Segoe UI font");
            None
        }
    }.unwrap();

    let fps_font = match font_ctx.load_font_from_rwops(sdl2::rwops::RWops::from_bytes(SEGOE).unwrap(), 15) {
        Ok(f) => Some(f),
        Err(e) => {
            report_error(CustomError {
                msg: e
            }, "Failed to load Segoe UI font");
            None
        }
    }.unwrap();

    let intro_text = match segoe_font.render("Click any keys or left/right click on your mouse to start tracking.").shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK) {
        Ok(t) => Some(t),
        Err(e) => {
            report_error(e, "Failed to render intro text");
            None
        }
    }.unwrap();

    let tc = canvas.texture_creator();

    let textured_intro = match tc.create_texture_from_surface(intro_text) {
        Ok(t) => Some(t),
        Err(e) => {
            report_error(e, "Failed to convert intro text to texture");
            None
        }
    }.unwrap();

    // states
    let mut clock_started = std::time::Instant::now();
    let mut active = false;
    let mut kb_active = false;
    let mut mouse_button_active = false;
    let mut mouse_move_active = false;
    let mut mouse_wheel_active = false;
    let mut ignore_first_keypress = true;
    let mut failed_time = std::time::Instant::now();
    let mut failed = false;

    // fps stuff
    let mut ft = std::time::Instant::now(); // frame time
    let mut fc = 0; // frame count
    let mut fps = 0.0; // frame per sec
    let mut mf = 0.0; // maximum fps
    let mut lf = 0.0; // minimum fps (shows on screen)
    let mut lpf = 0.0; // act as a cache
    let mut lft = std::time::Instant::now(); // minimum frame refresh time thingy
    'running: while running {
        // println!("Rendering");
        for event in event_pump.poll_iter() {
            if !active {
                match event {
                    sdl2::event::Event::Quit { .. } | sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        ..
                    } => {
                        println!("Quit");
                        break 'running;
                    },
                    sdl2::event::Event::KeyDown { .. } | sdl2::event::Event::MouseButtonDown { .. } => {
                        active = true;
                        kb_active = false;
                        mouse_button_active = false;
                        mouse_move_active = false;
                        mouse_wheel_active = false;
                        failed = false;
                        println!("Activated");
                        clock_started = Instant::now();
                    },
                    _ => {}
                }
            } else {
                match event {
                    sdl2::event::Event::Quit { .. } | sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        ..
                    } => {
                        println!("Quit");
                        break 'running
                    },
                    sdl2::event::Event::KeyDown { .. } | sdl2::event::Event::KeyUp { .. } => {
                        if ignore_first_keypress {
                            ignore_first_keypress = false;
                            continue;
                        }
                        kb_active = true;
                        if !failed {
                            failed_time = std::time::Instant::now();
                            failed = true;
                        }
                    },
                    sdl2::event::Event::MouseMotion { .. } => {
                        if ignore_first_keypress {
                            ignore_first_keypress = false;
                            continue;
                        }
                        mouse_move_active = true;
                        if !failed {
                            failed_time = std::time::Instant::now();
                            failed = true;
                        }
                    },
                    sdl2::event::Event::MouseButtonDown { .. } | sdl2::event::Event::MouseButtonUp { .. } => {
                        if ignore_first_keypress {
                            ignore_first_keypress = false;
                            continue;
                        }
                        mouse_button_active = true;
                        if !failed {
                            failed_time = std::time::Instant::now();
                            failed = true;
                        }
                    },
                    sdl2::event::Event::MouseWheel { .. } => {
                        if ignore_first_keypress {
                            ignore_first_keypress = false;
                            continue;
                        }
                        mouse_wheel_active = true;
                        if !failed {
                            failed_time = std::time::Instant::now();
                            failed = true;
                        }
                    }
                    _ => {},
                }
            }
        }
        if !active {
            match canvas.copy(&textured_intro, None, Some(get_middle_texture(&textured_intro, &canvas, None))) {
                Err(e) => {
                    report_error(CustomError { msg: e }, "Failed to copy intro text to canvas")
                },
                _ => {}
            };
        } else {
            if kb_active || mouse_button_active || mouse_move_active || mouse_wheel_active {
                failed = true;
                // let mut duration_time = "";
                let mut reasoning = "";
                if kb_active {
                    reasoning = "Keyboard presses detected.";
                } else if mouse_button_active {
                    reasoning = "Mouse button presses detected."
                } else if mouse_move_active {
                    reasoning = "Mouse movement detected."
                } else if mouse_wheel_active {
                    reasoning = "Mouse wheel movement detected."
                }
                let duration_time = format!("Inactive for {}", format_duration(failed_time - clock_started));
                let text = segoe_font.render(&duration_time).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
                let text2 = segoe_font.render(format!("Reason: {}", reasoning).as_str()).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
                canvas.copy(
                    &tc.create_texture_from_surface(&text).unwrap(),
                    None,
                    Some(
                        get_middle_surface(&text, &canvas, Some(
                            (canvas.output_size().unwrap().1 as f64 * 0.4) as u32
                        ))
                    )
                ).unwrap();
                canvas.copy(
                    &tc.create_texture_from_surface(&text2).unwrap(),
                    None,
                    Some(
                        get_middle_surface(&text2, &canvas, Some(
                            (canvas.output_size().unwrap().1 as f64 * 0.6) as u32
                        ))
                    )
                ).unwrap();
                if (std::time::Instant::now() - failed_time).as_secs() >= 5 {
                    active = false;

                }
            } 
        }
        let fps_text = fps_font.render(&format!("FPS: {}", fps)).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
        let mf_text = fps_font.render(&format!("Maximum FPS: {}", mf)).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
        let lfp_text = fps_font.render(&format!("Minimum FPS: {}", lf)).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
        let fc_text = fps_font.render(&format!("Capped FPS: {}", fl)).shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK).unwrap();
        canvas.copy(
            &tc.create_texture_from_surface(&fps_text).unwrap(),
            None,
            Some(
                sdl2::rect::Rect::new(
                    0,0, fps_text.width(), fps_text.height()
                )
            )
        ).unwrap();
        canvas.copy(
            &tc.create_texture_from_surface(&mf_text).unwrap(),
            None,
            Some(
                sdl2::rect::Rect::new(
                    0,20, mf_text.width(), mf_text.height()
                )
            )
        ).unwrap();
        canvas.copy(
            &tc.create_texture_from_surface(&lfp_text).unwrap(),
            None,
            Some(
                sdl2::rect::Rect::new(
                    0,40, lfp_text.width(), lfp_text.height()
                )
            )
        ).unwrap();
        canvas.copy(
            &tc.create_texture_from_surface(&fc_text).unwrap(),
            None,
            Some(
                sdl2::rect::Rect::new(
                    0,60, fc_text.width(), fc_text.height()
                )
            )
        ).unwrap();
        canvas.present();
        fc += 1;
        let elapsed_time = ft.elapsed();
        if elapsed_time.as_secs() >= 1 {
            fps = fc as f64 / elapsed_time.as_secs_f64();
            fc = 0;
            ft = std::time::Instant::now();
            if fps > mf {
                mf = fps
            } else if fps < lpf {
                lpf = fps
            }
        }
        let elapsed_time = lft.elapsed();
        if elapsed_time.as_secs() >= 3 {
            lf = lpf;
            lpf = fps;
            lft = std::time::Instant::now();
        }
        canvas.clear();
        capper.limit_fps();
    }

}

fn get_middle_surface(
    surface: &sdl2::surface::Surface,
    window: &sdl2::render::Canvas<sdl2::video::Window>,
    y: Option<u32>,
) -> sdl2::rect::Rect {
    let (w, h) = window.output_size().unwrap();
    let r: sdl2::rect::Rect;

    match y {
        Some(pos) => {
            r = sdl2::rect::Rect::new(
                ((w - surface.width()) / 2) as i32,
                ((pos as u32 - surface.height()) / 2) as i32,
                surface.width(),
                surface.height(),
            );
        }
        None => {
            r = sdl2::rect::Rect::new(
                ((w - surface.width()) / 2) as i32,
                ((h - surface.height()) / 2) as i32,
                surface.width(),
                surface.height(),
            );
        }
    }

    return r;
}

fn get_middle_texture(
    surface: &sdl2::render::Texture,
    window: &sdl2::render::Canvas<sdl2::video::Window>,
    y: Option<u32>,
) -> sdl2::rect::Rect {
    let (w, h) = window.output_size().unwrap();
    let r: sdl2::rect::Rect;

    match y {
        Some(pos) => {
            r = sdl2::rect::Rect::new(
                ((w - surface.query().width) / 2) as i32,
                ((pos as u32 - surface.query().height) / 2) as i32,
                surface.query().width,
                surface.query().height,
            );
        }
        None => {
            r = sdl2::rect::Rect::new(
                ((w - surface.query().width) / 2) as i32,
                ((h - surface.query().height) / 2) as i32,
                surface.query().width,
                surface.query().height,
            );
        }
    }

    return r;
}

fn format_duration(dur: std::time::Duration) -> String {
    let seconds = dur.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30; // Approximation, not considering varying month lengths
    let years = days / 365; // Approximation, not considering leap years

    let formatted_duration = if years > 0 {
        format!("{} years, {} months, {}weeks, {} days, {} hours, {} minutes, {} seconds.",
            years,
            months % 12,
            weeks % 4,
            days % 7,
            hours % 24,
            minutes % 60,
            seconds % 60,
        )
    } else if months > 0 {
        format!("{} months, {}weeks, {} days, {} hours, {} minutes, {} seconds.",
            months,
            weeks % 4,
            days % 30,
            hours % 24,
            minutes % 60,
            seconds % 60,
        )
    } else if weeks > 0 {
        format!("{}weeks, {} days, {} hours, {} minutes, {} seconds.",
            weeks,
            days % 7,
            hours % 24,
            minutes % 60,
            seconds % 60,
        )
    } else if days > 0 {
        format!("{} days, {} hours, {} minutes, {} seconds.",
            days,
            hours % 24,
            minutes % 60,
            seconds % 60,
        )
    } else if hours > 0 {
        format!("{} hours, {} minutes, {} seconds.",
            hours,
            minutes % 60,
            seconds % 60,
        )
    } else if minutes > 0 {
        format!("{} minutes, {} seconds.",
            minutes,
            seconds % 60,
        )
    } else {
        format!("{} seconds.", seconds)
    };

    formatted_duration
}