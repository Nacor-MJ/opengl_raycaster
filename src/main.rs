use glium::{
    self,
    glutin::{
        self,
        event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    }, 
    Surface, 
    uniform,
};

const TARGET_FPS: u32 = 60;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_micros((1.0 / TARGET_FPS as f32 * 1_000_000.0) as u64);

const PI: f64 = 3.1415926535;
const TAU: f32 = 2.0 * PI as f32;

/*
const MAP_SIZE: usize = 64;
const MAP: [i32; MAP_SIZE] = [ //used for collision detection, probably should be the same as the one in the shader 
    1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 1, 0, 0, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1
];
*/

const RESOLUTION: f64 = 1.0 / 512.0;
const STRIPES_ON_SCREEN: i32 = (2.0 / RESOLUTION) as i32;  
const FOV: f64 = PI / 2.0;
const RES_TO_FOV_RATIO: f64 = FOV / STRIPES_ON_SCREEN as f64;

const PLAYER_SPEED: f32 = 2.0;
const PLAYER_ANGLE_SPEED: f32 = 2.0;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();

    let window_builder = glutin::window::WindowBuilder::new().with_title("boobies (.)(.)");
    let context_builder = glutin::ContextBuilder::new().with_depth_buffer(24);

    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

    use std::fs::read_to_string;
    let program = glium::Program::from_source(
        &display,
        &read_to_string("src/shaders/VertexShader.txt").unwrap(), 
        &read_to_string("src/shaders/FragmentShader.txt").unwrap(), 
        Some(&read_to_string("src/shaders/GeometryShader.txt").unwrap())
    ).unwrap_or_else(|err| panic!("Shader error:\n{:?}", err));

    let mut player = Player {
        x: 4.0,
        y: 4.0,
        angle: 0.0 ,
    };

    use std::path::Path;
    let checkerboard_tex = add_texture(Path::new("C:/Users/matou/MyFiles/Rust/opengl_raycaster/brick.png"), &display);
    println!("pp");
    let brick_tex = add_texture(Path::new("C:/Users/matou/MyFiles/Rust/opengl_raycaster/checkerboard.png"), &display);

    let stripes: [Angle; STRIPES_ON_SCREEN as usize] = create_stripes();

    let vertex_buffer: glium::VertexBuffer<Angle> = glium::VertexBuffer::new(&display, &stripes).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let mut previous_frame_time = std::time::Instant::now();

    let mut keys = Keys {
        w: false,
        a: false,
        s: false,
        d: false,
    };

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => {
                                match keycode {
                                    VirtualKeyCode::W => keys.w = true,
                                    VirtualKeyCode::S => keys.s = true,
                                    VirtualKeyCode::A => keys.a = true,
                                    VirtualKeyCode::D => keys.d = true,
                                    _ => (),
                                }
                            }
                            ElementState::Released => {
                                match keycode {
                                    VirtualKeyCode::W => keys.w = false,
                                    VirtualKeyCode::S => keys.s = false,
                                    VirtualKeyCode::A => keys.a = false,
                                    VirtualKeyCode::D => keys.d = false,
                                    _ => (),
                                }
                            }
                        }
                    }
                },
                _ => (),
            },
            Event::RedrawRequested(_) | Event::MainEventsCleared => {
                let current_time = std::time::Instant::now();
                let dt = current_time - previous_frame_time;
                if dt >= FRAME_DURATION {
                    previous_frame_time = current_time;

                    println!("fps: {}", 1.0 / dt.as_secs_f64());

                    keys.handle(&dt.as_secs_f32(), &mut player);

                    let mut target = display.draw();

                    let uniforms = uniform! {
                        resolution: RESOLUTION as f32 / 2.0,
                        p_x: player.x,
                        p_y: player.y,
                        p_angle: player.angle,
                        tex2: &checkerboard_tex,
                        tex1: &brick_tex,
                    };

                    use glium::draw_parameters::*;
                    let params = glium::DrawParameters{
                        polygon_mode: PolygonMode::Fill,
                        .. Default::default()
                    };

                    target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);
                    target.draw(
                        &vertex_buffer,
                        &indices,
                        &program,
                        &uniforms,
                        &params,
                    ).unwrap();
                    target.finish().unwrap();
                }
            },
            _=> (),
        }
    });
}

#[derive(Clone, Copy, Debug)]
struct  Player {
    x: f32,
    y: f32,
    angle: f32,
}

#[derive(Default, Debug, Clone, Copy)]
struct Angle {
    d_angle: f32,
    screen_x: f32,
}
glium::implement_vertex!(Angle, d_angle, screen_x);


fn create_stripes() -> [Angle; STRIPES_ON_SCREEN as usize] {
    let mut array: [Angle; STRIPES_ON_SCREEN as usize] = [Default::default(); STRIPES_ON_SCREEN as usize];
    for idx in 0..STRIPES_ON_SCREEN {
        let d_angle = (idx as f32 - STRIPES_ON_SCREEN as f32 / 2.0) * RES_TO_FOV_RATIO as f32;
        array[idx as usize] = Angle{ 
            d_angle: d_angle,
            screen_x: idx as f32 / STRIPES_ON_SCREEN as f32 * 2.0 - 1.0,
        };
    };
    array
}

#[derive(Clone, Copy)]
struct Keys {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
}

impl Keys {
    fn handle(self, dt: &f32, player: &mut Player) {
        let mut d_x: f32 = 0.0;
        let mut d_y: f32 = 0.0;
        let mut d_angle: f32 = 0.0;
        
        if self.w == true {
            d_x += player.angle.cos() * PLAYER_SPEED * *dt;
            d_y += player.angle.sin() * PLAYER_SPEED * *dt;
        };
        if self.a == true {
            d_angle -= PLAYER_ANGLE_SPEED * *dt;
        };
        if self.s == true {
            d_x -= player.angle.cos() * PLAYER_SPEED * *dt;
            d_y -= player.angle.sin() * PLAYER_SPEED * *dt;
        };
        if self.d == true {
            d_angle += PLAYER_ANGLE_SPEED * *dt;
        };

        player.x += d_x;
        player.y += d_y;

        player.angle += d_angle;

        if player.angle < 0.0 {
            player.angle += TAU;
        } else if player.angle >= TAU {
            player.angle -= TAU;
        }
    }
}

fn add_texture(path: &std::path::Path, display: &dyn glium::backend::Facade) -> glium::texture::SrgbTexture2d {
    use std::io::BufReader;
    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(file);
    let image = image::load(reader, image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions); 
    glium::texture::SrgbTexture2d::new(display, image).unwrap()
}
