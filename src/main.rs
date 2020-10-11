#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::texture::pixel_buffer::PixelBuffer;
#[allow(unused_imports)]
use glium::{glutin, Surface};
use std::time::Duration;
use std::time::Instant;

pub const VIDEO_VERTEX_SHADER: &'static str = "#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec2 color;

out vec2 TexCoord;

void main()
{
    gl_Position = vec4(position, 1.0);
    TexCoord = vec2(color.x, color.y);
}
";

pub const PLANAR_FRAGMENT_SHADER: &'static str = "#version 330 core

#ifdef GL_ES
// Set default precision to medium
precision mediump int;
precision mediump float;
#endif

uniform sampler2D tex_y;
uniform sampler2D tex_u;
uniform sampler2D tex_v;
uniform int tex_format;
uniform float alpha;
uniform float tex_offset;

in vec2 TexCoord;
out vec4 FragColor;

void main()
{
    vec3 yuv;
    vec4 rgba;
    yuv.r = texture(tex_y, TexCoord).r - 0.0625;
    yuv.g = texture(tex_u, TexCoord).r - 0.5;
    yuv.b = texture(tex_v, TexCoord).r - 0.5;

    rgba.r = yuv.r + 1.596 * yuv.b;
    rgba.g = yuv.r - 0.813 * yuv.b - 0.391 * yuv.g;
    rgba.b = yuv.r + 2.018 * yuv.g;
    
    rgba.a = alpha;
    FragColor = rgba;
}";

#[derive(Clone)]
pub struct FfmpegDecodedPacket {
    pub y: std::vec::Vec<u8>,
    pub u: std::vec::Vec<u8>,
    pub v: std::vec::Vec<u8>,
}

impl FfmpegDecodedPacket {
    pub fn blank() -> FfmpegDecodedPacket {
        let mut r = FfmpegDecodedPacket {
            y: std::vec::Vec::new(),
            u: std::vec::Vec::new(),
            v: std::vec::Vec::new(),
        };
        r.y.resize(1920 * 1080, 0);
        r.u.resize(1920 * 1080 / 4, 0);
        r.v.resize(1920 * 1080 / 4, 0);
        r
    }
 
    #[inline]
    pub fn get_width(&self) -> u32 {
        1920
    }
    #[inline]
    pub fn get_height(&self) -> u32 {
        1080
    }
    #[inline]
    pub fn get_line_size(&self, index: usize) -> usize {
        match index {
            0 => 1920,
            1 => 1920 / 2,
            2 => 1920 / 2,
            _ => panic!(),
        }
    }
    #[inline]
    pub fn data(&self, index: usize) -> &[u8] {
        match index {
            0 => self.y.as_slice(),
            1 => self.u.as_slice(),
            2 => self.v.as_slice(),
            _ => panic!(),
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 2],
}
implement_vertex!(Vertex, position, color);

pub struct GliumContext {
    event_loop: Option<glutin::event_loop::EventLoop<()>>,
    display: Option<glium::Display>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
    index_buffer: Option<glium::IndexBuffer<u16>>,
    planar_program: Option<glium::program::Program>,
    y_texture: Option<glium::texture::texture2d::Texture2d>,
    u_texture: Option<glium::texture::texture2d::Texture2d>,
    v_texture: Option<glium::texture::texture2d::Texture2d>,
    y_pixel_buffer: Option<PixelBuffer<u8>>,
    u_pixel_buffer: Option<PixelBuffer<u8>>,
    v_pixel_buffer: Option<PixelBuffer<u8>>,
    current_width: Option<u32>,
    current_height: Option<u32>,
}

impl GliumContext {
    pub fn new(event_loop: glutin::event_loop::EventLoop<()>) -> GliumContext {
        //let event_loop = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new().with_vsync(true);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
        let r = GliumContext {
            event_loop: Some(event_loop),
            display: Some(display),
            vertex_buffer: None,
            index_buffer: None,
            planar_program: None,
            y_texture: None,
            u_texture: None,
            v_texture: None,
            y_pixel_buffer: None,
            u_pixel_buffer: None,
            v_pixel_buffer: None,
            current_width: None,
            current_height: None,
        };
        r
    }

    fn init_vertex_stuff(&mut self) {
        self.vertex_buffer = Some(
            glium::VertexBuffer::new(
                self.display.as_ref().unwrap(),
                &[
                    Vertex {
                        position: [-1.0, -1.0, 0.0],
                        color: [0.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, -1.0, 0.0],
                        color: [1.0, 1.0],
                    },
                    Vertex {
                        position: [-1.0, 1.0, 0.0],
                        color: [0.0, 0.0],
                    },
                    Vertex {
                        position: [1.0, 1.0, 0.0],
                        color: [1.0, 0.0],
                    },
                ],
            )
            .unwrap(),
        );

        self.index_buffer = Some(
            glium::IndexBuffer::new(
                self.display.as_ref().unwrap(),
                PrimitiveType::TriangleStrip,
                &[0u16, 1, 2, 3],
            )
            .unwrap(),
        );

        self.planar_program = Some(
            program!(self.display.as_ref().unwrap(),
                330 => {
                    vertex: VIDEO_VERTEX_SHADER,
                    fragment: PLANAR_FRAGMENT_SHADER,
                }
            )
            .unwrap(),
        );
    }

    fn parse_frame(&mut self, frame: &FfmpegDecodedPacket) {
        let width: u32;
        let height: u32;
        let mut linesize: [u32; 3] = [0; 3];

        width = frame.get_width();
        height = frame.get_height();

        for i in 0..3 {
            linesize[i] = frame.get_line_size(i) as u32;
        }

        /*
            Recreate textures every time the frame_width, frame_height
            or frame_pixel_format changes, or if one of them are None.
        */
        if self.current_width != Some(width) || self.current_height != Some(height) {
            self.current_width = Some(width);
            self.current_height = Some(height);

            let mipmap = glium::texture::MipmapsOption::NoMipmap;
            let format = glium::texture::UncompressedFloatFormat::U8;

            let rect1 = glium::Rect {
                left: 0,
                bottom: 0,
                width: width as u32,
                height: height as u32,
            };

            let rect2 = glium::Rect {
                left: 0,
                bottom: 0,
                width: width / 2 as u32,
                height: height / 2 as u32,
            };

            let rect3 = glium::Rect {
                left: 0,
                bottom: 0,
                width: width / 2 as u32,
                height: height / 2 as u32,
            };

            println!(
                "Creating textures width width {} and height {}.\n
            texture1: w: {}, h: {},
            texture2: w: {}, h: {},
            texture3: w: {}, h: {},
            ",
                width,
                height,
                rect1.width,
                rect1.height,
                rect2.width,
                rect2.height,
                rect3.width,
                rect3.height
            );

            self.y_pixel_buffer = Some(PixelBuffer::new_empty(
                self.display.as_ref().unwrap(),
                (rect1.width * rect1.height) as usize,
            ));

            self.y_texture = Some(
                glium::texture::texture2d::Texture2d::empty_with_format(
                    self.display.as_ref().unwrap(),
                    format,
                    mipmap,
                    rect1.width as u32,
                    rect1.height as u32,
                )
                .unwrap(),
            );

            self.u_pixel_buffer = Some(PixelBuffer::new_empty(
                self.display.as_ref().unwrap(),
                (rect2.width * rect2.height) as usize,
            ));

            self.u_texture = Some(
                glium::texture::texture2d::Texture2d::empty_with_format(
                    self.display.as_ref().unwrap(),
                    format,
                    mipmap,
                    rect2.width as u32,
                    rect2.height as u32,
                )
                .unwrap(),
            );

            self.v_pixel_buffer = Some(PixelBuffer::new_empty(
                self.display.as_ref().unwrap(),
                (rect3.width * rect3.height) as usize,
            ));

            self.v_texture = Some(
                glium::texture::texture2d::Texture2d::empty_with_format(
                    self.display.as_ref().unwrap(),
                    format,
                    mipmap,
                    rect3.width as u32,
                    rect3.height as u32,
                )
                .unwrap(),
            );

            println!("textures and pixel buffers created");
        }
    }

    pub fn stride_respect_upload_pixel_buffer(
        pixel_buffer: &PixelBuffer<u8>,
        width: u32,
        height: u32,
        linesize: usize,
        slice: &[u8],
    ) {
        //On normal cases we'd respect the stride and upload line by line
        pixel_buffer.write(slice);
    }

    fn draw(&mut self, frame: &FfmpegDecodedPacket) {
        GliumContext::stride_respect_upload_pixel_buffer(
            self.y_pixel_buffer.as_mut().unwrap(),
            self.y_texture.as_ref().unwrap().get_width(),
            self.y_texture.as_ref().unwrap().get_height().unwrap(),
            frame.get_line_size(0),
            frame.data(0),
        );

        self.y_texture
            .as_ref()
            .unwrap()
            .main_level()
            .raw_upload_from_pixel_buffer(
                self.y_pixel_buffer.as_ref().unwrap().as_slice(),
                0..self.y_texture.as_ref().unwrap().get_width(),
                0..self.y_texture.as_ref().unwrap().get_height().unwrap(),
                0..1,
            );

        GliumContext::stride_respect_upload_pixel_buffer(
            self.u_pixel_buffer.as_mut().unwrap(),
            self.u_texture.as_ref().unwrap().get_width(),
            self.u_texture.as_ref().unwrap().get_height().unwrap(),
            frame.get_line_size(1),
            frame.data(1),
        );

        self.u_texture
            .as_ref()
            .unwrap()
            .main_level()
            .raw_upload_from_pixel_buffer(
                self.u_pixel_buffer.as_ref().unwrap().as_slice(),
                0..self.u_texture.as_ref().unwrap().get_width(),
                0..self.u_texture.as_ref().unwrap().get_height().unwrap(),
                0..1,
            );

        GliumContext::stride_respect_upload_pixel_buffer(
            self.v_pixel_buffer.as_mut().unwrap(),
            self.v_texture.as_ref().unwrap().get_width(),
            self.v_texture.as_ref().unwrap().get_height().unwrap(),
            frame.get_line_size(2),
            frame.data(2),
        );
        self.v_texture
            .as_ref()
            .unwrap()
            .main_level()
            .raw_upload_from_pixel_buffer(
                self.v_pixel_buffer.as_ref().unwrap().as_slice(),
                0..self.v_texture.as_ref().unwrap().get_width(),
                0..self.v_texture.as_ref().unwrap().get_height().unwrap(),
                0..1,
            );

        let uniforms = uniform! {
            tex_y: self.y_texture.as_ref().unwrap(),
            tex_u: self.u_texture.as_ref().unwrap(),
            tex_v: self.v_texture.as_ref().unwrap(),
            tex_format: 0 as i32,
            alpha: 1.0f32
        };

        let mut target = self.display.as_ref().unwrap().draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                self.vertex_buffer.as_ref().unwrap(),
                self.index_buffer.as_ref().unwrap(),
                self.planar_program.as_ref().unwrap(),
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    }
}

pub struct GliumRenderer {}

impl GliumRenderer {
    pub fn new() -> GliumRenderer {
        GliumRenderer {}
    }

    pub fn run(&mut self) {
        let packet = FfmpegDecodedPacket::blank();
        let event_loop = glutin::event_loop::EventLoop::new();
        let event_loop_proxy = event_loop.create_proxy();
        let _t = std::thread::spawn(move || {
            while (true) {
                event_loop_proxy.send_event(()).unwrap();
                //Should trigger the event loop window 200 times per second which is enough for rendering anything in 60fps
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let on_glium_consume =
            move || -> Option<FfmpegDecodedPacket> { 
                //Some(FfmpegDecodedPacket::blank()) 
                Some(packet.clone())
            };

        let mut glium_context = GliumContext::new(event_loop);

        glium_context.init_vertex_stuff();
        let mut start = Instant::now();
        let mut fps = 0;
        glium_context
            .event_loop
            .take()
            .unwrap()
            .run(move |event, _, control_flow| {
                *control_flow = glutin::event_loop::ControlFlow::Wait;

                match event {
                    _ => {
                        let decoded_packet = (on_glium_consume)();
                        match decoded_packet {
                            Some(decoded_packet) => {
                                glium_context.parse_frame(&decoded_packet);
                                glium_context.draw(&decoded_packet);
                                let elapsed = start.elapsed();
                                if elapsed.as_millis()>=1000 {
                                    start = Instant::now();
                                    println!("{}", fps);
                                    fps = 0;
                                }
                                fps +=1;
                            }
                            None => {}
                        }
                        ()
                    }
                }
            });
        //_t.join();
    }
}

fn main() {
    let mut r = GliumRenderer::new();
    r.run();
}
