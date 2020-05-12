//! Shaders for layer with overlay

pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in   vec2    position;
        layout(location = 1) in   vec2    uv;

        layout(location = 0) out  vec2    screen_pos;
        layout(location = 1) out  vec2    tex_coords;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);

            tex_coords = uv;
            screen_pos = position - vec2(0.5);
        }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
        #version 450
        
        layout(set = 0, binding = 0) uniform sampler2D tex;
        layout(set = 0, binding = 1) uniform sampler2D overlay;

        layout(push_constant) uniform OverlayDrawingOptions {
            bool  reverse_mode;
            float blend_rate;
            float opacity;
        } mode;

        layout(location = 0) in   vec2    tex_coords;
        layout(location = 1) in   vec2    position;

        layout(location = 0) out  vec4    f_color;

        void main() {
            vec4  colour = texture(tex, tex_coords);
            float blend  = texture(tex, position).r;

            f_color.rgb = colour.rgb;
            
            if (mode.reverse_mode) {
                f_color.a = colour.a
                             * mode.opacity
                             * clamp(
                                 blend + 2.0 * (mode.blend_rate - 0.5),
                                 0.0,
                                 1.0
                                );
            } else {
                f_color.a = colour.a
                             * mode.opacity
                             * clamp(
                                 (1.0 - blend) + 2.0 * (mode.blend_rate - 0.5),
                                 0.0,
                                 1.0
                                );
            }
        }
        "
    }
}
