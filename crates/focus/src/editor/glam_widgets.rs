use egui::Widget;
use glam::*;
use std::any::Any;

macro_rules! vec_ui {
    ($name:ident $name_readonly:ident $ty:ty: $($component:ident)*) => {
        pub fn $name(
            ui: &mut egui::Ui,
            value: &mut dyn Any,
        ) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();
            let mut changed = false;

            ui.horizontal(|ui| {
                $(
                    changed |= egui::DragValue::new(&mut value.$component).speed(0.5).prefix(stringify!($component)).ui(ui).changed();
                )*
            });

            changed
        }

        pub fn $name_readonly(
            ui: &mut egui::Ui,
            value: &dyn Any,
        ) {
            let mut value = *value.downcast_ref::<$ty>().unwrap();
            ui.add_enabled_ui(false, |ui| $name(ui, &mut value));
        }
    };
}

macro_rules! mat_ui {
    ($name:ident $name_readonly:ident $sub_ui:ident $ty:ty: $($component:ident)*) => {
        pub fn $name(
            ui: &mut egui::Ui,
            value: &mut dyn Any,
        ) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();

            let mut changed = false;
            ui.vertical(|ui| {
                $(
                    ui.label(stringify!($component));
                    changed |= $sub_ui(ui, &mut value.$component);
                )*
            });
            changed
        }

        pub fn $name_readonly(
            ui: &mut egui::Ui,
            value: &dyn Any,
        ) {
            let mut value = *value.downcast_ref::<$ty>().unwrap();
            ui.add_enabled_ui(false, |ui| $name(ui, &mut value));
        }
    };
}

macro_rules! quat_angles_ui {
    ($name:ident $name_readonly:ident $ty:ty) => {
        pub fn $name(ui: &mut egui::Ui, value: &mut dyn Any) -> (bool, $ty) {
            let value = value.downcast_mut::<$ty>().unwrap();

            let euler_rot = EulerRot::XYZ;
            let (roll, pitch, yaw) = value.to_euler(euler_rot);

            let changed = false;
            ui.horizontal(|ui| {
                ui.drag_angle(&mut (roll as f32));
                ui.drag_angle(&mut (pitch as f32));
                ui.drag_angle(&mut (yaw as f32));
            });

            (
                changed,
                <$ty>::from_euler(euler_rot, roll.clone(), pitch.clone(), yaw.clone()),
            )
        }

        pub fn $name_readonly(ui: &mut egui::Ui, value: &dyn Any) {
            let mut value = *value.downcast_ref::<$ty>().unwrap();
            ui.add_enabled_ui(false, |ui| $name(ui, &mut value));
        }
    };
}

vec_ui!(vec2_ui vec2_ui_readonly Vec2: x y);
vec_ui!(vec3_ui vec3_ui_readonly Vec3: x y z);
vec_ui!(vec3a_ui vec3a_ui_readonly Vec3A: x y z);
vec_ui!(vec4_ui vec4_ui_readonly Vec4: x y z w);
vec_ui!(dvec2_ui dvec2_ui_readonly DVec2: x y);
vec_ui!(dvec3_ui dvec3_ui_readonly DVec3: x y z);
vec_ui!(dvec4_ui dvec4_ui_readonly DVec4: x y z w);

mat_ui!(mat2_ui mat2_ui_readonly vec2_ui Mat2: x_axis y_axis);
mat_ui!(mat3_ui mat3_ui_readonly vec3_ui Mat3: x_axis y_axis z_axis);
mat_ui!(mat3a_ui mat3a_ui_readonly vec3a_ui Mat3A: x_axis y_axis z_axis);
mat_ui!(mat4_ui mat4_ui_readonly vec4_ui Mat4: x_axis y_axis z_axis w_axis);
mat_ui!(dmat2_ui dmat2_ui_readonly dvec2_ui DMat2: x_axis y_axis);
mat_ui!(dmat3_ui dmat3_ui_readonly dvec3_ui DMat3: x_axis y_axis z_axis);
mat_ui!(dmat4_ui dmat4_ui_readonly dvec4_ui DMat4: x_axis y_axis z_axis w_axis);

quat_angles_ui!(quat_angles_ui quat_angles_ui_readonly Quat);
quat_angles_ui!(dquat_angles_ui dquat_angles_ui_readonly DQuat);
