#[macro_use]
extern crate glium;
use std::fs::File;
use std::io::Read;
use std::borrow::Cow;
//mod support;
mod shaders;
use self::shaders::packed_fragment::PACKED_FRAGMENT_SHADER;
use self::shaders::planar_fragment::PLANAR_FRAGMENT_SHADER;
use self::shaders::video_vertex::VIDEO_VERTEX_SHADER;
const YUV: u32 = 0;
use glium::texture::pixel_buffer::PixelBuffer;

use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{glutin, Surface};


fn main() {
    println!("pixel buffer example");
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 2],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(
            &display,
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
        .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[0u16, 1, 2, 3]).unwrap();

    // compiling shaders and linking them together
    let planar_program = program!(&display,
        330 => {
            vertex: VIDEO_VERTEX_SHADER,
            fragment: PLANAR_FRAGMENT_SHADER,
        }
    )
    .unwrap();
    /*
    let packed_program = program!(&display,
        330 => {
            vertex: VIDEO_VERTEX_SHADER,
            fragment: PACKED_FRAGMENT_SHADER,
        }
    )
    .unwrap();
    */
    let width = 1280;
    let height = 720;
    let mut y = vec![0u8; width * height];
    let mut u = vec![0u8; width * height / 4];
    let mut v = vec![0u8; width * height / 4];
    let mut f = File::open("/home/dev/orwell/lab/orwell_glium/assets/vaporwave.yuv")
        .expect("Unable to open file");

    f.read_exact(&mut y).unwrap();
    f.read_exact(&mut u).unwrap();
    f.read_exact(&mut v).unwrap();

    let mipmap = glium::texture::MipmapsOption::NoMipmap;
    let format = glium::texture::UncompressedFloatFormat::U8;

    let rect = glium::Rect{
        left: 0,
        bottom: 0,
        width: width as u32,
        height: height as u32,
    };

    let rect2 = glium::Rect{
        left: 0,
        bottom: 0,
        width: (width as u32)/2,
        height: (height as u32)/2,
    };

    //let y_raw = glium::texture::RawImage2d::from_raw_rgb(y, (width as u32, height as u32));
    /*
    let y_raw = glium::texture::RawImage2d{
        data: Cow::from(y),
        width: width as u32,
        height: height as u32,
        format: glium::texture::ClientFormat::U8
    };
    */
    let y_pixel_buffer = PixelBuffer::new_empty(&display, (width * height) as usize);
    let y_texture = glium::texture::texture2d::Texture2d::empty_with_format(&display, format, mipmap, width as u32, height as u32).unwrap();
    //y_texture.write(rect, y_raw);
    y_pixel_buffer.write(y.as_slice());
    y_texture.main_level().raw_upload_from_pixel_buffer(
        y_pixel_buffer.as_slice(),
      0..width as u32,
      0..height as u32,
      0..1);

    //let y_texture = glium::texture::Texture2d::new(&display, y_raw).unwrap();

    //let u_raw =
    //    glium::texture::RawImage2d::from_raw_rgb(u, ((width / 2) as u32, (height / 2) as u32));
    //let u_texture = glium::texture::texture2d::Texture2d::with_format(&display, u_raw, format, mipmap).unwrap();
    //let u_texture = glium::texture::Texture2d::new(&display, u_raw).unwrap();
    /*
    let u_raw = glium::texture::RawImage2d{
        data: Cow::from(u),
        width: (width as u32)/2,
        height: (height as u32)/2,
        format: glium::texture::ClientFormat::U8
    };
    */
    let u_pixel_buffer = PixelBuffer::new_empty(&display, (width/2 * height/2) as usize);
    let u_texture = glium::texture::texture2d::Texture2d::empty_with_format(&display, format, mipmap, (width as u32)/2, (height as u32)/2).unwrap();
    //u_texture.write(rect2, u_raw);
    u_pixel_buffer.write(u.as_slice());
    u_texture.main_level().raw_upload_from_pixel_buffer(
        u_pixel_buffer.as_slice(),
      0..(width/2) as u32,
      0..(height/2) as u32,
      0..1);

    //let v_raw =
    //    glium::texture::RawImage2d::from_raw_rgb(v, ((width / 2) as u32, (height / 2) as u32));
    //let v_texture = glium::texture::texture2d::Texture2d::with_format(&display, v_raw, format, mipmap).unwrap();
    //let v_texture = glium::texture::Texture2d::new(&display, v_raw).unwrap();
    /*
    let v_raw = glium::texture::RawImage2d{
        data: Cow::from(v),
        width: (width as u32)/2,
        height: (height as u32)/2,
        format: glium::texture::ClientFormat::U8
    };
    */
    let v_pixel_buffer = PixelBuffer::new_empty(&display, (width/2 * height/2) as usize);
    let v_texture = glium::texture::texture2d::Texture2d::empty_with_format(&display, format, mipmap, (width as u32)/2, (height as u32)/2).unwrap();
    //v_texture.write(rect2, v_raw);
    v_pixel_buffer.write(v.as_slice());
    v_texture.main_level().raw_upload_from_pixel_buffer(
        v_pixel_buffer.as_slice(),
      0..(width/2) as u32,
      0..(height/2) as u32,
      0..1);



    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex_y: &y_texture,
            tex_u: &u_texture,
            tex_v: &v_texture,
            tex_format: YUV as i32,
            alpha: 1.0f32
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &planar_program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    };

    draw();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                // Redraw the triangle when the window is resized.
                glutin::event::WindowEvent::Resized(..) => {
                    draw();
                    glutin::event_loop::ControlFlow::Poll
                }
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
    });
}
