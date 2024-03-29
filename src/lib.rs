use bevy::prelude::*;
use std::cmp::Ordering;

#[derive(Component, Clone, Copy, Debug)]
///Setting mass to 0.0 will cause divion by zero panics :D
pub struct MassPoint {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub force: Vec2,
    //Makes the bool immovable for the jelly systems
    pub movable: bool,
}

impl MassPoint {
    fn update_mass_point(&mut self, time: &Res<Time>) {
        if !self.movable {
            return;
        }

        self.velocity += (self.force * time.delta_seconds()) / self.mass;

        self.position += self.velocity * time.delta_seconds();

        self.force = Vec2::ZERO;
    }

    pub fn update_mass_points(mut query: Query<&mut MassPoint>, time: Res<Time>) {
        for mut mp in query.iter_mut() {
            mp.update_mass_point(&time);
        }
    }
}

impl Default for MassPoint {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
            force: Vec2::ZERO,
            movable: true,
        }
    }
}

#[derive(Component)]
pub struct Spring {
    ///mp stand for mass point
    pub mp_a: Entity,
    pub mp_b: Entity,
    pub stiffness: f32,
    pub rest_length: f32,
    pub damping_factor: f32,
}

impl Spring {
    fn get_force(
        mp_a: MassPoint,
        mp_b: MassPoint,
        stiffness: f32,
        damping_factor: f32,
        rest_length: f32,
    ) -> (Vec2, Vec2) {
        let force = stiffness * (mp_b.position.distance(mp_a.position) - rest_length);

        let corresponding_velocity_difference = mp_b.velocity - mp_a.velocity;

        let normalized_direction_vector = (mp_b.position - mp_a.position).normalize_or_zero();

        let dot_product = corresponding_velocity_difference.dot(normalized_direction_vector);

        //Total spring force
        let total_spring_force = force + dot_product * damping_factor;

        let a_change =
            mp_a.force + total_spring_force * (mp_b.position - mp_a.position).normalize_or_zero();

        let b_change =
            mp_b.force + total_spring_force * (mp_a.position - mp_b.position).normalize_or_zero();

        (a_change, b_change)
    }

    fn move_mass_points(&mut self, query: &mut Query<&mut MassPoint>) {
        let mp_arr = query.get_many_mut([self.mp_a, self.mp_b]).unwrap();

        let [mut mp_a, mut mp_b] = mp_arr;

        let changes = Spring::get_force(
            mp_a.clone(),
            mp_b.clone(),
            self.stiffness,
            self.damping_factor,
            self.rest_length,
        );

        if mp_a.movable {
            mp_a.force += changes.0;
        }

        if mp_b.movable {
            mp_b.force += changes.1;
        }
    }

    pub fn update_springs(mut query: Query<&mut Spring>, mut mp_query: Query<&mut MassPoint>) {
        for mut spring in query.iter_mut() {
            spring.move_mass_points(&mut mp_query)
        }
    }
}

///SIZE has to be the same as the amont of points in the Shape
#[derive(Component)]
pub struct Shape {
    pub points: Vec<Entity>,
    pub springs: Vec<Entity>,
}

impl Shape {
    ///The points of the bounding box is retunred as follows min_x, max_x, min_y, max_y
    pub fn get_bounding_box(&self, query: &Query<&MassPoint>) -> (f32, f32, f32, f32) {
        let mut min_x = 0.0;
        let mut max_x = 0.0;
        let mut min_y = 0.0;
        let mut max_y = 0.0;

        for point in &self.points {
            let point = query.get(*point).unwrap();

            if point.position.x < min_x {
                min_x = point.position.x;
            }

            if point.position.x > max_x {
                max_x = point.position.x;
            }

            if point.position.y < min_y {
                min_y = point.position.y;
            }

            if point.position.y < min_y {
                max_y = point.position.y;
            }
        }

        (min_x, max_x, min_y, max_y)
    }

    pub fn resolve_collisions(
        mut shapes_query: Query<&mut Shape>,
        mut mut_points_query: Query<&mut MassPoint>,
        time: Res<Time>,
    ) {
        //Tests all the points on each of the shapes against all of the other shapes.
        let mut combinations = shapes_query.iter_combinations_mut();
        while let Some([mut a, mut b]) = combinations.fetch_next() {
            Self::shape_collision(&mut a, &mut b, &mut mut_points_query, &time);
            Self::shape_collision(&mut b, &mut a, &mut mut_points_query, &time);
        }
    }

    fn shape_collision(
        shape_a: &mut Shape,
        shape_b: &mut Shape,
        query: &mut Query<&mut MassPoint>,
        time: &Res<Time>,
    ) {
        for point in shape_a.points.iter() {
            let point_point = *query.get(*point).unwrap();
            match Self::point_to_polygon_collision_detection(point_point, shape_b, query) {
                //Resolves the collision it it detected any.
                Some(v) => {

                    Self::resolve_collision(v.0, v.1, point, query, time);

                }
                _ => {}
            }
        }
    }

    fn resolve_collision(
        line: [Entity; 2],
        closest_point: Vec2,
        point: &Entity,
        query: &mut Query<&mut MassPoint>,
        time: &Res<Time>,
    ) {
        let [mut a, mut b, mut point] = query.get_many_mut([line[0], line[1], *point]).unwrap();

        if !a.movable && !b.movable && !point.movable {
            return;
        }

        let average_line_mass = (a.mass + b.mass) / 2.0;

        let average_line_velocity = (a.velocity + b.velocity) / 2.0;

        //Skip the rest of the calculations if the line is immovable
        if !a.movable && !b.movable && point.movable {
            point.position = closest_point;

            let ((x, y), _) =
                calculate_new_velocities(&point, average_line_velocity, average_line_mass);

            point.position.x += x * time.delta_seconds();
            point.position.y += y * time.delta_seconds();

            return;
        }

        let line_move_multiplier = if point.movable {
            average_line_mass / (average_line_mass + point.mass)
        } else {
            1.0
        };

        //Account for the different masses
        let line_a_mass_move_multiplier = 1.0 + ((a.mass + b.mass) / 2.0);

        //Account for the closest point not being in the middle
        let line_a_slant_multiplier = 1.0 + a.position.distance(closest_point) / a.position.distance(b.position);

        //Average them for the final multiplier
        let a_move_multiplier = (line_a_mass_move_multiplier + line_a_slant_multiplier) / 2.0;

        //Where the points should meet
        let meet = point
            .position
            .lerp(closest_point, 1.0 - line_move_multiplier);

        //What the point needs to change by
        let point_change = meet - point.position;

        //What the line needs to change by
        let line_change = meet - closest_point;

        //What the start of the line needs to change by
        let line_a_change = line_change * a_move_multiplier;

        //What the end of the line needs to change by
        let line_b_change = line_change * (2.0 - a_move_multiplier);

        if point.movable {
            point.position += point_change;
        }

        a.position += line_a_change;

        b.position += line_b_change;


        //Update thier velocities
        let new_velocities =
            calculate_new_velocities(&point, average_line_velocity, average_line_mass);
        //Move them away from each other to avoid recalculating the new velocity multiple times

        if point.movable {
            point.position.x += new_velocities.0 .0 * time.delta_seconds();
            point.position.y += new_velocities.0 .1 * time.delta_seconds();
        }

        a.position.x += new_velocities.1 .0 * time.delta_seconds();
        a.position.y += new_velocities.1 .1 * time.delta_seconds();

        b.position.x += new_velocities.1 .0 * time.delta_seconds();
        b.position.y += new_velocities.1 .1 * time.delta_seconds();

        if point.movable {
            //Set thier new velocities
            point.velocity.x = new_velocities.0 .0;
            point.velocity.y = new_velocities.0 .1;
        }

        a.velocity.x = new_velocities.1 .0;
        a.velocity.y = new_velocities.1 .1;

        b.velocity.x = new_velocities.1 .0;
        b.velocity.y = new_velocities.1 .1;
    }

    fn point_to_polygon_collision_detection(
        point: MassPoint,
        shape: &Shape,
        query: &mut Query<&mut MassPoint>,
    ) -> Option<([Entity; 2], Vec2)> {
        let mut collision = false;
        let b_length = shape.points.len();

        //The lines which we have collided with
        let mut collision_lines = vec![];

        let shape_points = &shape.points;

        let shape: Vec<&MassPoint> = query.iter_many(&shape.points).collect();

        for current in 0..b_length {
            //Get the next vertice in shape and wrap around to zero if we hit the end
            let mut next = current + 1;
            if next == b_length {
                next = 0
            };

            let current_vertice = shape[current].position;
            let next_vertice = shape[next].position;

            let point_position = point.position;

            collision_lines.push((
                [shape_points[current], shape_points[next]],
                find_nearest_point_on_line(point.position, &current_vertice, &next_vertice),
            ));

            //Check if the point is inside the shape utilising black magic.
            if ((current_vertice.y > point_position.y) != (next_vertice.y > point_position.y))
                && (point_position.x
                    < (next_vertice.x - current_vertice.x) * (point_position.y - current_vertice.y)
                        / (next_vertice.y - current_vertice.y)
                        + current_vertice.x)
            {
                collision = !collision;
            }
        }

        if collision {
            let lengths_to_lines: Vec<f32> = collision_lines
                .iter()
                .map(|v| point.position.distance(v.1).abs())
                .collect();

            let best_line_index = lengths_to_lines
                .iter()
                .enumerate()
                .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap_or(Ordering::Equal))
                .unwrap()
                .0;
            return Some((
                [
                    collision_lines[best_line_index].0[0],
                    collision_lines[best_line_index].0[1],
                ],
                collision_lines[best_line_index].1,
            ));
        }

        None
    }
}

pub fn find_nearest_point_on_line(point: Vec2, origin: &Vec2, end: &Vec2) -> Vec2 {
    //Get heading
    let heading = *end - *origin;
    let magnitude_max = (heading.x.powi(2) + heading.y.powi(2)).sqrt();
    let heading = heading.normalize();

    //Do projection from the point but clamp it
    let lhs = point - *origin;
    let dot_product = lhs.dot(heading);
    let dot_product = dot_product.clamp(0.0, magnitude_max);
    *origin + heading * dot_product
}

fn calculate_new_velocities(
    point: &MassPoint,
    average_line_velocity: Vec2,
    average_line_mass: f32,
) -> ((f32, f32), (f32, f32)) {
    //Update thier velocities
    let new_point_x_velocity = (point.velocity.x * (point.mass - average_line_mass)
        + (2.0 * average_line_mass * average_line_velocity.x))
        / (point.mass + average_line_mass);

    let new_point_y_velocity = (point.velocity.y * (point.mass - average_line_mass)
        + (2.0 * average_line_mass * average_line_velocity.y))
        / (point.mass + average_line_mass);

    let new_line_x_velocity = (average_line_velocity.x * (average_line_mass - point.mass)
        + (2.0 * point.mass * point.velocity.x))
        / (point.mass + average_line_mass);

    let new_line_y_velocity = (average_line_velocity.y * (average_line_mass - point.mass)
        + (2.0 * point.mass * point.velocity.y))
        / (point.mass + average_line_mass);

    return (
        (new_point_x_velocity, new_point_y_velocity),
        (new_line_x_velocity, new_line_y_velocity),
    );
}

#[derive(Component)]
pub struct ShapeMatcher {
    pub orginal_shape: Vec<Vec2>,
    pub spring_stiffnes: f32,
    pub spring_damping_factor: f32,
}

pub fn match_shapes(mut query: Query<(&mut Shape, &ShapeMatcher)>, mp_query: Query<&MassPoint>) {
    for mut shape in query.iter_mut() {
        if shape.0.points.len() != shape.1.orginal_shape.len() {
            eprintln!(
                "A Shape and its corresponding ShapeMatcher must be of the same length. Skiping"
            );
            continue;
        }

        let current_x_posistion: f32 = mp_query
            .iter_many(&shape.0.points)
            .map(|v| v.position.x)
            .sum::<f32>()
            / shape.0.points.len() as f32;
        let current_y_posistion: f32 = mp_query
            .iter_many(&shape.0.points)
            .map(|v| v.position.y)
            .sum::<f32>()
            / shape.0.points.len() as f32;

        for point in shape.0.points.iter_mut().enumerate() {}
    }
}
