use crate::metadata::{ContributorsList, PluginMetadataV0};
use egui::{Context, Key, Window};
use egui_extras::{Column, TableBuilder};
use focus::EguiRenderLoop;

#[derive(Debug)]
pub struct CauldronUI {
    ui_open: bool,
    ui_plugins_open: bool,
    ui_egui: EguiToggles,
    open_key: Key,
    plugins: Vec<PluginMetadataV0>,
}

impl CauldronUI {
    pub fn new(plugins: Vec<PluginMetadataV0>, open_key: Key) -> CauldronUI {
        CauldronUI {
            ui_open: false,
            ui_plugins_open: false,
            ui_egui: EguiToggles {
                settings: false,
                inspection: false,
                memory: false,
            },
            open_key,
            plugins,
        }
    }
}

impl EguiRenderLoop for CauldronUI {
    fn initialize<'a>(&'a mut self, ctx: &'a Context) {
        egui_extras::install_image_loaders(ctx);
        // catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
    }

    fn before_render<'a>(&'a mut self, ctx: &'a Context) {
        if ctx.input(|f| f.key_pressed(self.open_key)) {
            self.ui_open = !self.ui_open;
        }
    }

    fn render(&mut self, ctx: &Context) {
        if !self.ui_open {
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("Cauldron: v{}", env!("CARGO_PKG_VERSION")));
                ui.separator();
                if ui.button("Plugins").clicked() {
                    self.ui_plugins_open = !self.ui_plugins_open;
                }
                ui.menu_button("Egui", |ui| {
                    ui.checkbox(&mut self.ui_egui.settings, "Settings");
                    ui.checkbox(&mut self.ui_egui.inspection, "Inspection");
                    ui.checkbox(&mut self.ui_egui.memory, "Memory");
                });
            });
        });

        Window::new("Plugins")
            .open(&mut self.ui_plugins_open)
            .resizable(true)
            .show(ctx, |ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto());

                table
                    .header(20.0, |mut header| {
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
                            ui.strong("Description");
                        });
                        header.col(|ui| {
                            ui.strong("Authors");
                        });
                    })
                    .body(|mut body| {
                        self.plugins.iter().enumerate().for_each(|(index, plugin)| {
                            body.row(18.0, |mut row| {
                                let mut name = String::new();
                                let mut description = String::new();
                                let mut authors = String::new();

                                if plugin.cauldron.metadata.is_some() {
                                    let meta = plugin.cauldron.metadata.as_ref().unwrap();
                                    name = meta.name.clone().unwrap_or(String::new());
                                    description = meta.description.clone().unwrap_or(String::new());
                                    if meta.contributors.is_some() {
                                        let contributors = meta.contributors.as_ref().unwrap();
                                        match contributors {
                                            ContributorsList::Plain(contributors) => {
                                                authors = contributors.join(", ");
                                            }
                                            ContributorsList::WithRoles(contributors) => {
                                                authors = contributors
                                                    .keys()
                                                    .map(|a| a.to_string())
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                            }
                                        }
                                    }
                                }

                                row.col(|ui| {
                                    ui.label(index.to_string());
                                });
                                row.col(|ui| {
                                    ui.label(plugin.cauldron.id.clone());
                                });
                                row.col(|ui| {
                                    ui.label(plugin.cauldron.version.clone().to_string());
                                });
                                row.col(|ui| {
                                    ui.label(name);
                                });
                                row.col(|ui| {
                                    ui.label(description);
                                });
                                row.col(|ui| {
                                    ui.label(authors);
                                });
                            });
                        });
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

#[derive(Debug)]
pub struct EguiToggles {
    settings: bool,
    inspection: bool,
    memory: bool,
}

unsafe impl Sync for CauldronUI {}
unsafe impl Sync for EguiToggles {}
unsafe impl Send for CauldronUI {}
unsafe impl Send for EguiToggles {}
