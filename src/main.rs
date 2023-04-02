use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use jelly2d::*;

const GRAVITY: f32 = 100.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup)
        .add_system(Spring::update_springs)
        .add_system(MassPoint::update_mass_points)
        .add_system(apply_gravity)
        //.add_system(draw_springs)
        .add_system(draw_shapes)
        .add_system(Shape::resolve_collisions)
        .add_system(dampen_velocities)
        .run();
}

//TODO: FIXA FJÄDRARNA

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    build_square(
        Vec2::new(-200.0, 100.0),
        &mut commands,
        Vec2::new(00.0, 0.0),
        200.0,
        2.0,
    );
    // build_square(
    //     Vec2::new(0.0, 300.0),
    //     &mut commands,
    //     Vec2::new(00.0, 0.0),
    //     200.0,
    //     1.0,
    // );

    // build_square(
    //     Vec2::new(-400.0, 300.0),
    //     &mut commands,
    //     Vec2::new(200.0, 0.0),
    //     250.0,
    //     2.0,
    // );

    creat_floor(commands);
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

fn draw_shapes(query: Query<&Shape>, query_mp: Query<&MassPoint>, mut lines: ResMut<DebugLines>) {
    for shape in query.iter() {
        let points: Vec<&MassPoint> = query_mp.iter_many(&shape.points).collect();

        for current in 0..points.len() {
            let mut next = current + 1;

            if next == points.len() {
                next = 0;
            }

            let pos_a = points[current].position.extend(0.0);
            let pos_b = points[next].position.extend(0.0);

            lines.line_colored(pos_a, pos_b, 0.0, Color::GREEN);
        }
    }
}

fn build_square(offset: Vec2, commands: &mut Commands, velocity: Vec2, size: f32, mass: f32) {
    const STIFFNESS: f32 = 5.0;
    const DAMPING_FACTOR: f32 = 0.95;

    let p1 = MassPoint {
        position: Vec2 { x: 0.0, y: 0.0 } + offset,
        velocity,
        mass,
        ..default()
    };

    let p2 = MassPoint {
        position: Vec2 { x: size, y: 0.0 } + offset,
        velocity,
        mass,
        ..default()
    };

    let p3 = MassPoint {
        position: Vec2 { x: size, y: -size } + offset,
        velocity,
        mass,
        ..default()
    };

    let p4 = MassPoint {
        position: Vec2 { x: 0.0, y: -size } + offset,
        velocity,
        mass,
        ..default()
    };

    let p1_id = commands.spawn(p1).id();
    let p2_id = commands.spawn(p2).id();
    let p3_id = commands.spawn(p3).id();
    let p4_id = commands.spawn(p4).id();

    let spring_a = Spring {
        mp_a: p1_id,
        mp_b: p2_id,
        stiffness: STIFFNESS,
        rest_length: size,
        damping_factor: DAMPING_FACTOR,
    };

    let spring_b = Spring {
        mp_a: p3_id,
        mp_b: p2_id,
        stiffness: STIFFNESS,
        rest_length: size,
        damping_factor: DAMPING_FACTOR,
    };

    let spring_c = Spring {
        mp_a: p3_id,
        mp_b: p4_id,
        stiffness: STIFFNESS,
        rest_length: size,
        damping_factor: DAMPING_FACTOR,
    };

    let spring_d = Spring {
        mp_a: p4_id,
        mp_b: p1_id,
        stiffness: STIFFNESS,
        rest_length: size,
        damping_factor: DAMPING_FACTOR,
    };

    let spring_e = Spring {
        mp_a: p1_id,
        mp_b: p3_id,
        stiffness: STIFFNESS,
        rest_length: size * 2f32.sqrt(),
        damping_factor: DAMPING_FACTOR,
    };

    let spring_f = Spring {
        mp_a: p2_id,
        mp_b: p4_id,
        stiffness: STIFFNESS,
        rest_length: size * 2f32.sqrt(),
        damping_factor: DAMPING_FACTOR,
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

fn apply_gravity(mut query: Query<&mut MassPoint>, time: Res<Time>) {
    for mut point in query.iter_mut() {
        if point.movable {
            point.position.y -= GRAVITY * time.delta_seconds();
        }
    }
}

fn dampen_velocities(mut query: Query<&mut MassPoint>, time: Res<Time>) {
    for mut point in query.iter_mut() {
        point.velocity *= 1.0 - (0.5 * time.delta_seconds());
    }
}

fn creat_floor(mut commands: Commands) {
    let p1 = MassPoint {
        position: Vec2 {
            x: -300.0,
            y: -200.0,
        },
        movable: false,
        ..default()
    };

    let p2 = MassPoint {
        position: Vec2 {
            x: 300.0,
            y: -200.0,
        },
        movable: false,
        ..default()
    };

    let p3 = MassPoint {
        position: Vec2 {
            x: 300.0,
            y: -300.0,
        },
        movable: false,
        ..default()
    };

    let p4 = MassPoint {
        position: Vec2 {
            x: -300.0,
            y: -300.0,
        },
        movable: false,
        ..default()
    };

    let p1 = commands.spawn(p1).id();
    let p2 = commands.spawn(p2).id();
    let p3 = commands.spawn(p3).id();
    let p4 = commands.spawn(p4).id();

    let floor = Shape {
        points: vec![p1, p2, p3, p4],
        springs: vec![],
    };

    commands.spawn(floor);
}
