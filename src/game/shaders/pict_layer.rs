//! Shaders for pict-layer

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in   vec2    position;
        layout(location = 1) in   vec2    uv;

        layout(location = 0) out  vec2    tex_coords;

        layout(push_constant) uniform PushConstantData {
            vec2  offset;
            float opacity;
            int radius_x;
            int radius_y;
        } pc;

        void main() {
            gl_Position = vec4(pc.offset + position, 0.0, 1.0);
            tex_coords = uv;
        }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
        #version 450

        #define PI 3.141592653589793
        
        layout(set = 0, binding = 0) uniform sampler2D tex;

        layout(location = 0) in   vec2    tex_coords;
        layout(location = 0) out  vec4    f_color;

        layout(push_constant) uniform PushConstantData {
            vec2  offset;
            float opacity;
            int radius_x;
            int radius_y;
        } pc;

        float blur_rate(float x, float y) {
            float r = x * x + y * y;
            return exp( - r / 2.0) / sqrt(2.0 * PI);
        }

        void main() {
            f_color = texture(tex, tex_coords);
            f_color.a *= pc.opacity;

            float sum = 1.0;

            for (int i = 1; i <= pc.radius_x; i++) {
                for (int j = 1; j <= pc.radius_y; j++) {
                    float rate = blur_rate(i / float(pc.radius_x), j / float(pc.radius_y));
                    vec2 delta = vec2(i / 3200.0, j / 1800.0);
                    vec2 delta2 = delta;

                    delta2.x = -delta2.x;

                    sum += 4.0 * rate;
                    f_color += rate * texture(tex, tex_coords + delta);
                    f_color += rate * texture(tex, tex_coords - delta);
                    f_color += rate * texture(tex, tex_coords + delta2);
                    f_color += rate * texture(tex, tex_coords - delta2);
                }
            }

            f_color /= sum;
        }
        "
    }
}
