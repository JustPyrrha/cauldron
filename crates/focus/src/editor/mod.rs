pub mod glam_widgets;
pub mod widgets_demo_window;

use egui::{Ui, Widget};
use libdecima::log;
use libdecima::types::decima::core::enums::EPhysicsCollisionLayerGame;
use libdecima::types::decima::core::rot_matrix::DecimaCoordinateDirection;
use libdecima::types::decima::core::vec::Vec3;
use libdecima::types::decima::core::world_position::WorldPosition;
use libdecima::types::decima::core::world_transform::GlamTransform;
use libdecima::types::decima::core::{g_core::GCore, mover::Mover, player::Player};

#[derive(Debug)]
pub struct FocusEditor {
    /// enabled when selecting a world object to inspect
    pub _inspection_active: bool,

    pub collision_layer: EPhysicsCollisionLayerGame,
    pub player_as_entity: bool,
    pub flag_unk_a: bool,
    pub flag_unk_b: bool,
    pub line_distance: f64,
    pub move_distance: f32,
}

impl Default for FocusEditor {
    fn default() -> Self {
        FocusEditor {
            _inspection_active: false,
            collision_layer: EPhysicsCollisionLayerGame::Ray_vs_Static,
            player_as_entity: false,
            flag_unk_a: false,
            flag_unk_b: false,
            line_distance: 200.0,
            move_distance: 10.0,
        }
    }
}

impl FocusEditor {
    pub fn editor_ui(&mut self, ui: &mut Ui) {
        if ui.button("IntersectLine").clicked() {
            self.handle_inspect_click();
        }
        ui.checkbox(&mut self.player_as_entity, "use player as intersect entity");
        ui.checkbox(&mut self.flag_unk_a, "flag_unk_a");
        ui.checkbox(&mut self.flag_unk_b, "flag_unk_b");
        egui::ComboBox::from_label("Collision Layer")
            .selected_text(format!("{:?}", self.collision_layer))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static,
                    "Static",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ,
                    "Dynamic_HQ",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic,
                    "Dynamic",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Water_Affected,
                    "Water_Affected",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Sound_occlusion,
                    "Sound_occlusion",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Ragdoll,
                    "Ragdoll",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Gravity_pockets,
                    "Gravity_pockets",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static_shoot_through,
                    "Static_shoot_through",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_shoot_through,
                    "Dynamic_shoot_through",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Bullet_blocker,
                    "Bullet_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Bullet_blocker_raycast,
                    "Bullet_blocker_raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Trigger,
                    "Trigger",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Trigger_raycast,
                    "Trigger_raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Heavy_Ragdoll,
                    "Heavy_Ragdoll",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Player_Collision,
                    "Player_Collision",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Player,
                    "Player",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::AI_or_Remote_Player,
                    "AI_or_Remote_Player",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Humanoid_blocker,
                    "Humanoid_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Player_blocker,
                    "Player_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Foot_placement,
                    "Foot_placement",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_but_humanoid,
                    "Dynamic_but_humanoid",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Humanoid_raycast_movement,
                    "Humanoid_raycast_movement",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Humanoid_Ragdoll,
                    "Humanoid_Ragdoll",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Ragdoll_no_collision_vs_static,
                    "Ragdoll_no_collision_vs_static",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Impact_effect_Event_Query,
                    "Impact_effect_Event_Query",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Combat_AI,
                    "Combat_AI",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Humanoid_movement_helper,
                    "Humanoid_movement_helper",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Projectile,
                    "Projectile",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ_Pullable_Shoot_Through,
                    "Dynamic_HQ_Pullable_Shoot_Through",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_Pullable,
                    "Dynamic_Pullable",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::AI_static,
                    "AI_static",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Flying_Mount_blocker_raycast,
                    "Flying_Mount_blocker_raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::vs_Humanoids,
                    "vs_Humanoids",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ_Pullable,
                    "Dynamic_HQ_Pullable",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Flying_Mount,
                    "Flying_Mount",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Ragdoll_stopper,
                    "Ragdoll_stopper",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Mortally_wounded,
                    "Mortally_wounded",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ_but_humanoid,
                    "Dynamic_HQ_but_humanoid",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Proxy_player,
                    "Proxy_player",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Blocks_AI_Hearing,
                    "Blocks_AI_Hearing",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Pullables_blocker,
                    "Pullables_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Flying_Mount_blocker,
                    "Flying_Mount_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_Pullables_blocker,
                    "Dynamic_Pullables_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Blocks_vision,
                    "Blocks_vision",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Player_Ragdoll,
                    "Player_Ragdoll",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_No_Collision,
                    "Dynamic_No_Collision",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Blocks_AI_Hearing_Raycast,
                    "Blocks_AI_Hearing_Raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Weapon_blocker,
                    "Weapon_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_blocker,
                    "Dynamic_blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Focus_Object,
                    "Focus_Object",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Focus_Object_Raycast,
                    "Focus_Object_Raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static_But_Humanoid,
                    "Static_But_Humanoid",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::vs_Bullet_Blocker_Only,
                    "vs_Bullet_Blocker_Only",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Particles_Collision,
                    "Particles_Collision",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Ray_vs_Static,
                    "Ray_vs_Static",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Special_Object,
                    "Special_Object",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static_But_Humanoid_but_NavMesh,
                    "Static_But_Humanoid_but_NavMesh",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Camera_Obstruction,
                    "Camera_Obstruction",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Navigation_Mesh,
                    "Navigation_Mesh",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Vault_Query,
                    "Vault_Query",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::SVO_Query,
                    "SVO_Query",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::SVO_Blocker,
                    "SVO_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Special_Object_Blocker,
                    "Special_Object_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Navigation_Mesh_Hard_Obstacle,
                    "Navigation_Mesh_Hard_Obstacle",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Camera_Blocker_Raycast,
                    "Camera_Blocker_Raycast",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Camera_Collision,
                    "Camera_Collision",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static_but_Navigation_Mesh,
                    "Static_but_Navigation_Mesh",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_but_Navigation_Mesh,
                    "Dynamic_but_Navigation_Mesh",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Foot_Support,
                    "Foot_Support",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ_but_FOOT_Support,
                    "Dynamic_HQ_but_FOOT_Support",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Humanoid_raycast_movement_no_ragdoll,
                    "Humanoid_raycast_movement_no_ragdoll",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Camera_Blocker,
                    "Camera_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Player_and_Camera_Blocker,
                    "Player_and_Camera_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_Shoot_Through__No_Camera,
                    "Dynamic_Shoot_Through (No_Camera)",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Air_Movement_Blocker,
                    "Air_Movement_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_HQ_but_human_and_Air_Movement,
                    "Dynamic_HQ_but_human_and_Air_Movement",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_But_Ragdolls,
                    "Dynamic_But_Ragdolls",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Foliage,
                    "Foliage",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Foliage_Query,
                    "Foliage_Query",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Soft_Lock_Blocker,
                    "Soft_Lock_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Soft_Lock_Blocker_Query,
                    "Soft_Lock_Blocker_Query",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_but_FOOT_Support,
                    "Dynamic_but_FOOT_Support",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Projectile_and_vs_Bullet_Blocker,
                    "Projectile_and_vs_Bullet_Blocker",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Static_Debug,
                    "Static_Debug",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Dynamic_Debug,
                    "Dynamic_Debug",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Lightbake_Visibility,
                    "Lightbake_Visibility",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Debug_Draw,
                    "Debug_Draw",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::Density_Debug,
                    "Density_Debug",
                );
                ui.selectable_value(
                    &mut self.collision_layer,
                    EPhysicsCollisionLayerGame::No_Collision,
                    "No_Collision",
                );
            });

        ui.label("line distance");
        egui::DragValue::new(&mut self.line_distance)
            .speed(0.5)
            .ui(ui);
        ui.separator();

        ui.label("Mover");
        egui::DragValue::new(&mut self.move_distance)
            .speed(0.5)
            .ui(ui);
        if ui.button("move!").clicked() {
            let player = Player::get_local_player(0);
            let player = unsafe { &*player };
            let entity = unsafe { &*player.entity };

            let mut transform = GlamTransform::from(entity.get_transform().clone());
            transform.pos += (transform.rot.up().normalize() * self.move_distance).as_dvec3();
            Mover::override_movement(entity.mover, &transform.into(), 10.0, false);
        }
    }

    pub fn handle_inspect_click(&mut self) {
        let player = Player::get_local_player(0);
        let player = unsafe { &*player };
        // log!("{player:#?}");

        // crosshair pos
        let camera_transform =
            unsafe { &*player.get_last_active_camera().unwrap() }.get_transform();
        let player_transform = GlamTransform::from(unsafe { &*player.entity }.get_transform());

        let mut end_transform = GlamTransform::from(camera_transform.clone());
        end_transform.pos += player_transform.pos;
        end_transform.rot += player_transform.rot;

        end_transform.pos =
            end_transform.pos + end_transform.rot.y_axis.as_dvec3() * self.line_distance;

        let mut ray_hit_pos = WorldPosition::default();
        let mut ray_vec3 = Vec3::default();
        let mut ray_float: f32 = 0.0;
        let mut ray_entity = std::ptr::null_mut();
        let mut ray_void = std::ptr::null_mut();
        let mut ray_i32 = 0i32;
        let mut ray_u32 = 0u32;

        let result = GCore::IntersectLine(
            &camera_transform.pos,
            &end_transform.pos.into(),
            self.collision_layer,
            if self.player_as_entity {
                player.entity as *const _
            } else {
                std::ptr::null()
            },
            self.flag_unk_a,
            self.flag_unk_b,
            0,
            &mut ray_hit_pos,
            &mut ray_vec3,
            &mut ray_float,
            &mut ray_entity,
            &mut ray_void,
            &mut ray_i32,
            &mut ray_u32,
        );

        log!("focus", "ray hit? {result}");

        log!("focus", "ray pos {ray_hit_pos:#?}");
        log!("focus", "ray dir {ray_vec3:#?}");
        log!("focus", "ray float {ray_float:#?}");
        log!("focus", "ray entity {ray_entity:p}");
        // if !ray_entity.is_null() {
        //     let entity = *ray_entity;
        //     log!("focus", "entity: {entity:#?}");
        // }
        log!("focus", "ray void: {ray_void:p}"); // todo: investigate if this is a RTTIRefObject
        // if !ray_void.is_null() {
        //     let void = *ray_void;
        //     log!("focus", "void: {void:p}");
        // }
        log!("focus", "ray i32: {ray_i32}");
        log!("focus", "ray u32: {ray_u32}");
    }
}

unsafe impl Send for FocusEditor {}
unsafe impl Sync for FocusEditor {}
