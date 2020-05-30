//! Shaders for pict-layer

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in   vec2    position;
        layout(location = 1) in   vec2    uv;
        layout(location = 2) in   float   text_count;

        layout(location = 0) out  vec2    tex_coords;
        layout(location = 1) out  float   text_alpha;

        layout(push_constant) uniform PushConstantData {
            vec2  offset;
            float text_cursor;
        } pc;

        void main() {
            gl_Position = vec4(pc.offset + position, 0.0, 1.0);
            tex_coords = uv;
            float tc = text_count;

            text_alpha = pc.text_cursor - tc;
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
        layout(location = 1) in   float   text_alpha;

        layout(location = 0) out  vec4    f_color;

        void main() {
            float t = texture(tex, tex_coords).r;
            f_color = vec4(t);
            
            float alpha = t;

            for (int i = -3; i <= 3; i++) {
                for (int j = -6; j <= 6; j++) {
                    alpha += texture(tex, tex_coords + vec2(i * 0.9e-3, j * 1.6e-3)).r;
                }
            }

            f_color.a = clamp(alpha, 0.0, 1.0) * clamp(text_alpha, 0.0, 1.0);
        }
        "
    }
}
