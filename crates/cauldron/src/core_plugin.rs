use egui::{Context, Key, Window};
use egui_extras::{Column, TableBuilder};
use semver::Version;
use focus::EguiRenderLoop;
use crate::{Plugin, PluginMeta};

pub struct CauldronCore {}


pub struct EguiToggles {
    settings: bool,
    inspection: bool,
    memory: bool,
}

impl Plugin for CauldronCore {
    fn meta(&self) -> PluginMeta {
        PluginMeta::builder("cauldron-core", Version::parse(env!("CARGO_PKG_VERSION")).unwrap())
            .name("Cauldron Core")
            .description("Core Cauldron utilities.")
            .authors(vec!["JustPyrrha"])
            .build()
    }
}

unsafe impl Sync for CauldronUI {}
unsafe impl Sync for EguiToggles {}
unsafe impl Send for CauldronUI {}
unsafe impl Send for EguiToggles {}
unsafe impl Sync for CauldronCore {}
unsafe impl Send for CauldronCore {}


pub struct CauldronUI {
    ui_open: bool,
    ui_plugins_open: bool,
    ui_egui: EguiToggles
}

impl CauldronUI {
    pub fn new() -> CauldronUI {
        CauldronUI {
            ui_open: false,
            ui_plugins_open: false,
            ui_egui: EguiToggles {
                settings: false,
                inspection: false,
                memory: false,
            }
        }
    }
}

impl EguiRenderLoop for CauldronUI {

    fn initialize<'a>(
        &'a mut self,
        ctx: &'a Context,
    ) {
        egui_extras::install_image_loaders(ctx);
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
    }

    fn before_render<'a>(
        &'a mut self,
        ctx: &'a Context,
    ) {
        if ctx.input(|f| f.key_pressed(Key::Backtick)) {
            self.ui_open = !self.ui_open;
        }
    }

    fn render(&mut self, ctx: &Context) {
        if !self.ui_open {
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!(
                    "Cauldron: v{}",
                    env!("CARGO_PKG_VERSION")
                ));
                ui.separator();
                if ui.button("Plugins").clicked() {
                    self.ui_plugins_open = !self.ui_plugins_open;
                }
                ui.menu_button("Egui", |ui| {
                    ui.checkbox(&mut self.ui_egui.settings, "🔧 Settings");
                    ui.checkbox(&mut self.ui_egui.inspection, "🔍 Inspection");
                    ui.checkbox(&mut self.ui_egui.memory, "📝 Memory");
                });
            });
        });

        Window::new("Plugins")
            .open(&mut self.ui_plugins_open)
            .show(ctx, |ui| {
                ui.label("todo");

                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto());

                table.header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Order");
                    });
                    header.col(|ui| {
                        ui.strong("Id");
                    });
                    header.col(|ui| {
                        ui.strong("Version");
                    });
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Authors");
                    });
                    header.col(|ui| {
                        ui.strong("Description");
                    });
                }).body(|mut body| {

                    // plugins.iter().enumerate().for_each(|(index, plugin)| {
                    //     body.row(18.0, |mut row| {
                    //         let meta = plugin.meta();
                    //         let name = &meta.name.unwrap_or(String::new());
                    //         let authors = &meta.authors.unwrap_or(Vec::new()).join(", ");
                    //         let description = &meta.description.unwrap_or(String::new());
                    //
                    //         row.col(|ui| {
                    //             ui.label(index.to_string());
                    //         });
                    //         row.col(|ui| {
                    //             ui.label(meta.id);
                    //         });
                    //         row.col(|ui| {
                    //             ui.label(meta.version.to_string());
                    //         });
                    //         row.col(|ui| {
                    //             ui.label(name);
                    //         });
                    //         row.col(|ui| {
                    //             ui.label(authors);
                    //         });
                    //         row.col(|ui| {
                    //             ui.label(description);
                    //         });
                    //     });
                    // });
                });
            });

        Window::new("🔧 Settings")
            .open(&mut self.ui_egui.settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        Window::new("🔍 Inspection")
            .open(&mut self.ui_egui.inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        Window::new("📝 Memory")
            .open(&mut self.ui_egui.memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });
    }
}