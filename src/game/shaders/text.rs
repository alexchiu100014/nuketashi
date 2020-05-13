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
            bool  use_cursor;
        } pc;

        void main() {
            gl_Position = vec4(pc.offset + position, 0.0, 1.0);
            tex_coords = uv;

            float tc = text_count;

            if (pc.use_cursor)
                text_alpha = clamp(pc.text_cursor - tc, 0.0, 1.0);
            else
                text_alpha = 1.0;
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

            for (float i = -0.002; i <= 0.002; i += 5.0e-5) {
                for (float j = -0.006; j <= 0.006; j += 5.0e-5) {
                    alpha += texture(tex, tex_coords + vec2(i, j)).r;
                }
            }

            f_color.a = clamp(alpha, 0.0, 1.0) * text_alpha;
        }
        "
    }
}
