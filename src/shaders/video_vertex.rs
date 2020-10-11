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