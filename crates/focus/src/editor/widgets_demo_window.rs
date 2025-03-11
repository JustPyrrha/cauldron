use crate::editor::glam_widgets::*;
use glam::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct WidgetsDemo {
    pub vec2: Vec2,
    pub vec3: Vec3,
    pub vec3a: Vec3A,
    pub vec4: Vec4,
    pub dvec2: DVec2,
    pub dvec3: DVec3,
    pub dvec4: DVec4,

    pub mat2: Mat2,
    pub mat3: Mat3,
    pub mat3a: Mat3A,
    pub mat4: Mat4,
    pub dmat2: DMat2,
    pub dmat3: DMat3,
    pub dmat4: DMat4,

    pub quat: Quat,
    pub dquat: DQuat,
}

impl WidgetsDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Vec", |ui| {
            egui::Grid::new("vec grid").striped(true).show(ui, |ui| {
                ui.label("Vec2");
                vec2_ui(ui, &mut self.vec2);
                ui.end_row();
                ui.label("Vec2 (Read Only)");
                vec2_ui_readonly(ui, &self.vec2);
                ui.end_row();

                ui.label("Vec3");
                vec3_ui(ui, &mut self.vec3);
                ui.end_row();
                ui.label("Vec3 (Read Only)");
                vec3_ui_readonly(ui, &self.vec3);
                ui.end_row();

                ui.label("Vec3A");
                vec3a_ui(ui, &mut self.vec3a);
                ui.end_row();
                ui.label("Vec3A (Read Only)");
                vec3a_ui_readonly(ui, &self.vec3a);
                ui.end_row();

                ui.label("Vec4");
                vec4_ui(ui, &mut self.vec4);
                ui.end_row();
                ui.label("Vec4 (Read Only)");
                vec4_ui_readonly(ui, &self.vec4);
                ui.end_row();

                ui.label("DVec2");
                dvec2_ui(ui, &mut self.dvec2);
                ui.end_row();
                ui.label("DVec2 (Read Only)");
                dvec2_ui_readonly(ui, &self.dvec2);
                ui.end_row();

                ui.label("DVec3");
                dvec3_ui(ui, &mut self.dvec3);
                ui.end_row();
                ui.label("DVec3 (Read Only)");
                dvec3_ui_readonly(ui, &self.dvec3);
                ui.end_row();

                ui.label("DVec4");
                dvec4_ui(ui, &mut self.dvec4);
                ui.end_row();
                ui.label("DVec4 (Read Only)");
                dvec4_ui_readonly(ui, &self.dvec4);
                ui.end_row();
            });
        });

        ui.collapsing("Mat", |ui| {
            egui::Grid::new("mat grid").striped(true).show(ui, |ui| {
                ui.label("Mat2");
                mat2_ui(ui, &mut self.mat2);
                ui.end_row();
                ui.label("Mat2 (Read Only)");
                mat2_ui_readonly(ui, &self.mat2);
                ui.end_row();

                ui.label("Mat3");
                mat3_ui(ui, &mut self.mat3);
                ui.end_row();
                ui.label("Mat3 (Read Only)");
                mat3_ui_readonly(ui, &self.mat3);
                ui.end_row();

                ui.label("Mat3A");
                mat3a_ui(ui, &mut self.mat3a);
                ui.end_row();
                ui.label("Mat3A (Read Only)");
                mat3a_ui_readonly(ui, &self.mat3a);
                ui.end_row();

                ui.label("Mat4");
                mat4_ui(ui, &mut self.mat4);
                ui.end_row();
                ui.label("Mat4 (Read Only)");
                mat4_ui_readonly(ui, &self.mat4);
                ui.end_row();

                ui.label("DMat2");
                dmat2_ui(ui, &mut self.dmat2);
                ui.end_row();
                ui.label("DMat2 (Read Only)");
                dmat2_ui_readonly(ui, &self.dmat2);
                ui.end_row();

                ui.label("DMat3");
                dmat3_ui(ui, &mut self.dmat3);
                ui.end_row();
                ui.label("DMat3 (Read Only)");
                dmat3_ui_readonly(ui, &self.dmat3);
                ui.end_row();

                ui.label("DMat4");
                dmat4_ui(ui, &mut self.dmat4);
                ui.end_row();
                ui.label("DMat4 (Read Only)");
                dmat4_ui_readonly(ui, &self.dmat4);
                ui.end_row();
            });
        });

        ui.collapsing("Quat", |ui| {
            egui::Grid::new("quat grid").striped(true).show(ui, |ui| {
                ui.label("Quat Angles");
                let (_, q) = quat_angles_ui(ui, &mut self.quat);
                self.quat = q;
                ui.end_row();
                ui.label("Quat Angles (Read Only)");
                quat_angles_ui_readonly(ui, &self.quat);
                ui.end_row();

                ui.label("DQuat Angles");
                let (_, q) = dquat_angles_ui(ui, &mut self.dquat);
                self.dquat = q;
                ui.end_row();
                ui.label("DQuat Angles (Read Only)");
                dquat_angles_ui_readonly(ui, &self.dquat);
                ui.end_row();
            });
        });
    }
}
