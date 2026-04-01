use slint::Color;

const CYAN: Color = Color::from_rgb_u8(0, 255, 255);
const MAGENTA: Color = Color::from_rgb_u8(255, 0, 255);
const YELLOW: Color = Color::from_rgb_u8(255, 255, 0);
const AMBER: Color = Color::from_rgb_u8(255, 191, 0);
const WHITE: Color = Color::from_rgb_u8(255, 255, 255);
const WARM_WHITE: Color = Color::from_rgb_u8(255, 214, 170);
const COLD_WHITE: Color = Color::from_rgb_u8(230, 242, 255);
const UV: Color = Color::from_rgb_u8(100, 0, 255);
const LIME: Color = Color::from_rgb_u8(191, 255, 0);
const INDIGO: Color = Color::from_rgb_u8(75, 0, 130);

const fn unquantize(value: u8) -> f32 {
    (value as f32) / 255.0
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ColorInfo {
    pub dimmer: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub cyan: u8,
    pub magenta: u8,
    pub yellow: u8,
    pub amber: u8,
    pub white: u8,
    pub warm_white: u8,
    pub cold_white: u8,
    pub uv: u8,
    pub lime: u8,
    pub indigo: u8,
}

impl ColorInfo {
    /// Converts `ColorInfo` to [`slint::Color`].
    pub fn to_slint_color(&self) -> Color {
        let mut r = unquantize(self.red);
        let mut g = unquantize(self.green);
        let mut b = unquantize(self.blue);

        macro_rules! add_color {
            ($($field:ident => $color:expr),* $(,)?) => {
                $(
                    let factor = unquantize(self.$field);
                    r += unquantize($color.red()) * factor;
                    g += unquantize($color.green()) * factor;
                    b += unquantize($color.blue())* factor;
                )*
            };
        }

        add_color!(
            cyan => CYAN,
            magenta => MAGENTA,
            yellow => YELLOW,
            amber => AMBER,
            white => WHITE,
            warm_white => WARM_WHITE,
            cold_white => COLD_WHITE,
            uv => UV,
            lime => LIME,
            indigo => INDIGO
        );

        let dimmer = unquantize(self.dimmer);

        Color::from_rgb_f32(
            (dimmer * r).min(255.),
            (dimmer * g).min(255.),
            (dimmer * b).min(255.),
        )
    }
}
