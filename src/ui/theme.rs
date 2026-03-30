use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CatppuccinFlavor {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl CatppuccinFlavor {
    pub const ALL: [Self; 4] = [Self::Latte, Self::Frappe, Self::Macchiato, Self::Mocha];

    pub fn name(self) -> &'static str {
        match self {
            Self::Latte => "Latte",
            Self::Frappe => "Frappé",
            Self::Macchiato => "Macchiato",
            Self::Mocha => "Mocha",
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        ctx.set_visuals(self.visuals());
    }

    fn palette(self) -> Palette {
        match self {
            Self::Latte => LATTE,
            Self::Frappe => FRAPPE,
            Self::Macchiato => MACCHIATO,
            Self::Mocha => MOCHA,
        }
    }

    fn visuals(self) -> egui::Visuals {
        let p = self.palette();
        let is_light = matches!(self, Self::Latte);

        let make_widget = |bg: egui::Color32, bg_weak: egui::Color32, fg: egui::Color32| {
            egui::style::WidgetVisuals {
                bg_fill: bg,
                weak_bg_fill: bg_weak,
                bg_stroke: egui::Stroke::new(1.0, p.overlay0),
                fg_stroke: egui::Stroke::new(1.0, fg),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
            }
        };

        egui::Visuals {
            dark_mode: !is_light,
            override_text_color: Some(p.text),
            widgets: egui::style::Widgets {
                noninteractive: make_widget(p.base, p.mantle, p.subtext1),
                inactive: make_widget(p.surface0, p.surface0, p.text),
                hovered: make_widget(p.surface1, p.surface1, p.text),
                active: make_widget(p.surface2, p.surface2, p.text),
                open: make_widget(p.surface0, p.surface0, p.text),
            },
            selection: egui::style::Selection {
                bg_fill: egui::Color32::from_rgba_unmultiplied(
                    p.blue.r(),
                    p.blue.g(),
                    p.blue.b(),
                    80,
                ),
                stroke: egui::Stroke::new(1.0, p.blue),
            },
            hyperlink_color: p.blue,
            faint_bg_color: p.surface0,
            extreme_bg_color: p.crust,
            code_bg_color: p.mantle,
            warn_fg_color: p.peach,
            error_fg_color: p.red,
            window_fill: p.base,
            panel_fill: p.mantle,
            window_stroke: egui::Stroke::new(1.0, p.overlay0),
            window_shadow: egui::Shadow {
                offset: [2, 4],
                blur: 16,
                spread: 0,
                color: egui::Color32::from_black_alpha(60),
            },
            popup_shadow: egui::Shadow {
                offset: [1, 2],
                blur: 8,
                spread: 0,
                color: egui::Color32::from_black_alpha(40),
            },
            ..if is_light {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            }
        }
    }
}

struct Palette {
    // surfaces
    crust: egui::Color32,
    mantle: egui::Color32,
    base: egui::Color32,
    surface0: egui::Color32,
    surface1: egui::Color32,
    surface2: egui::Color32,
    overlay0: egui::Color32,
    // text
    text: egui::Color32,
    subtext1: egui::Color32,
    // accent
    blue: egui::Color32,
    red: egui::Color32,
    peach: egui::Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

const LATTE: Palette = Palette {
    crust:    rgb(220, 224, 232),
    mantle:   rgb(230, 233, 239),
    base:     rgb(239, 241, 245),
    surface0: rgb(204, 208, 218),
    surface1: rgb(188, 192, 204),
    surface2: rgb(172, 176, 190),
    overlay0: rgb(156, 160, 176),
    text:     rgb(76,  79,  105),
    subtext1: rgb(92,  95,  119),
    blue:     rgb(30,  102, 245),
    red:      rgb(210, 15,  57),
    peach:    rgb(254, 100, 11),
};

const FRAPPE: Palette = Palette {
    crust:    rgb(35,  38,  52),
    mantle:   rgb(41,  44,  60),
    base:     rgb(48,  52,  70),
    surface0: rgb(65,  69,  89),
    surface1: rgb(81,  87,  109),
    surface2: rgb(98,  104, 128),
    overlay0: rgb(115, 121, 148),
    text:     rgb(198, 208, 245),
    subtext1: rgb(181, 191, 226),
    blue:     rgb(140, 170, 238),
    red:      rgb(231, 130, 132),
    peach:    rgb(239, 159, 118),
};

const MACCHIATO: Palette = Palette {
    crust:    rgb(24,  25,  38),
    mantle:   rgb(30,  32,  48),
    base:     rgb(36,  39,  58),
    surface0: rgb(54,  58,  79),
    surface1: rgb(73,  77,  100),
    surface2: rgb(91,  96,  120),
    overlay0: rgb(110, 115, 141),
    text:     rgb(202, 211, 245),
    subtext1: rgb(184, 192, 224),
    blue:     rgb(138, 173, 244),
    red:      rgb(237, 135, 150),
    peach:    rgb(245, 169, 127),
};

const MOCHA: Palette = Palette {
    crust:    rgb(17,  17,  27),
    mantle:   rgb(24,  24,  37),
    base:     rgb(30,  30,  46),
    surface0: rgb(49,  50,  68),
    surface1: rgb(69,  71,  90),
    surface2: rgb(88,  91,  112),
    overlay0: rgb(108, 112, 134),
    text:     rgb(205, 214, 244),
    subtext1: rgb(186, 194, 222),
    blue:     rgb(137, 180, 250),
    red:      rgb(243, 139, 168),
    peach:    rgb(250, 179, 135),
};
