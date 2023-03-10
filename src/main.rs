use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use jello2d::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup)
        .insert_resource(Gravity(Vec2::new(-10.0, 0.0)))
        .add_system(Spring::update_springs)
        .add_system(MassPoint::update_mass_points)
        .add_system(draw_springs)
        .run();
}

//TODO: FIXA FJÄDRARNA

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let p1 = MassPoint {
        position: Vec2 { x: 100.0, y: 50.0 },
        ..default()
    };

    let p2 = MassPoint {
        position: Vec2 { x: 500.0, y: 200.0 },
        ..default()
    };

    let p3 = MassPoint {
        position: Vec2 {
            x: 500.0,
            y: -100.0,
        },
        mass: 500.0,
        ..default()
    };

    let p1_id = commands.spawn(p1).id();
    let p2_id = commands.spawn(p2).id();
    let p3_id = commands.spawn(p3).id();

    let triangle = Shape {
        points: [p1_id, p2_id, p3_id],
    };

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
        stiffness: 10.0,
        rest_length: 200.0,
        damping_factor: 0.1,
    };

    let spring_c = Spring {
        mp_a: p3_id,
        mp_b: p1_id,
        stiffness: 1.0,
        rest_length: 200.0,
        damping_factor: 0.5,
    };

    commands.spawn(triangle);
    commands.spawn(spring_a);
    commands.spawn(spring_b);
    commands.spawn(spring_c);
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
