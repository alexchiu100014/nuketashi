pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in   vec2    position;
        layout(location = 0) out  vec2    tex_coords;

        void main() {
            gl_Position = vec4(position * 2.0 - vec2(1.0, 1.0), 0.0, 1.0);
            tex_coords = position;
        }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
        #version 450

        layout(set = 0, binding = 0) uniform sampler2D tex;

        layout(location = 0) in   vec2    tex_coords;
        layout(location = 0) out  vec4    f_color;

        void main() {
            f_color = texture(tex, tex_coords);
        }
        "
    }
}
