use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use jelly2d::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup)
        .insert_resource(Gravity(Vec2::new(0.0, 0.0)))
        .add_system(Spring::update_springs)
        .add_system(MassPoint::update_mass_points)
        .add_system(draw_springs)
        .add_system(Shape::resolve_collisons)
        .run();
}

//TODO: FIXA FJÄDRARNA

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    build_square(Vec2::new(-400.0, 0.0), &mut commands);
    build_square(Vec2::ZERO, &mut commands);
    build_square(Vec2::splat(10.0), &mut commands);
}

fn draw_springs(query: Query<&Spring>, query_mp: Query<&MassPoint>, mut lines: ResMut<DebugLines>) {
    for spring in query.iter() {
        let pos_a = query_mp.get(spring.mp_a).unwrap().position;
        let pos_b = query_mp.get(spring.mp_b).unwrap().position;

        let pos_a = Vec3 {
            x: pos_a.x,
            y: pos_a.y,
            z: 0.0,
        };
        let pos_b = Vec3 {
            x: pos_b.x,
            y: pos_b.y,
            z: 0.0,
        };

        lines.line_colored(pos_a, pos_b, 0.0, Color::RED);
    }
}

fn build_square(offset: Vec2, commands: &mut Commands) {
    let p1 = MassPoint {
        position: Vec2 { x: 100.0, y: 50.0 } + offset,
        ..default()
    };

    let p2 = MassPoint {
        position: Vec2 { x: 500.0, y: 200.0 } + offset,
        ..default()
    };

    let p3 = MassPoint {
        position: Vec2 {
            x: 500.0,
            y: -100.0,
        } + offset,
        ..default()
    };

    let p4 = MassPoint {
        position: Vec2 {
            x: -300.0,
            y: -200.0,
        } + offset,
        ..default()
    };

    let p1_id = commands.spawn(p1).id();
    let p2_id = commands.spawn(p2).id();
    let p3_id = commands.spawn(p3).id();
    let p4_id = commands.spawn(p4).id();

    let spring_a = Spring {
        mp_a: p1_id,
        mp_b: p2_id,
        stiffness: 1.0,
        rest_length: 200.0,
        damping_factor: 0.5,
    };

    let spring_b = Spring {
        mp_a: p3_id,
        mp_b: p2_id,
        stiffness: 1.0,
        rest_length: 200.0,
        damping_factor: 0.5,
    };

    let spring_c = Spring {
        mp_a: p3_id,
        mp_b: p4_id,
        stiffness: 1.0,
        rest_length: 200.0,
        damping_factor: 0.5,
    };

    let spring_d = Spring {
        mp_a: p4_id,
        mp_b: p1_id,
        stiffness: 1.0,
        rest_length: 200.0,
        damping_factor: 0.5,
    };

    let spring_e = Spring {
        mp_a: p1_id,
        mp_b: p3_id,
        stiffness: 1.0,
        rest_length: 200.0 * 2f32.sqrt(),
        damping_factor: 0.5,
    };

    let spring_f = Spring {
        mp_a: p2_id,
        mp_b: p4_id,
        stiffness: 1.0,
        rest_length: 200.0 * 2f32.sqrt(),
        damping_factor: 0.5,
    };

    let s_a_id = commands.spawn(spring_a).id();
    let s_b_id = commands.spawn(spring_b).id();
    let s_c_id = commands.spawn(spring_c).id();
    let s_d_id = commands.spawn(spring_d).id();
    let s_e_id = commands.spawn(spring_e).id();
    let s_f_id = commands.spawn(spring_f).id();

    let square = Shape {
        points: vec![p1_id, p2_id, p3_id, p4_id],
        springs: vec![s_a_id, s_b_id, s_c_id, s_d_id, s_e_id, s_f_id],
    };

    commands.spawn(square);
}
