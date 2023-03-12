use bevy::prelude::*;

#[derive(Resource)]
pub struct Gravity(pub Vec2);

#[derive(Component)]
pub struct MassPoint {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub force: Vec2,
}

impl MassPoint {
    fn update_mass_point(&mut self, time: &Res<Time>, gravity: &Res<Gravity>) {
        let mut force = self.force;
        force += gravity.0;

        self.velocity += (force * time.delta_seconds()) / self.mass;

        self.position += self.velocity * time.delta_seconds();

        self.force = Vec2::ZERO;
    }

    pub fn update_mass_points(
        mut query: Query<&mut MassPoint>,
        time: Res<Time>,
        gravity: Res<Gravity>,
    ) {
        for mut mp in query.iter_mut() {
            mp.update_mass_point(&time, &gravity);
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
    fn get_force(&mut self, query: &Query<&mut MassPoint>) -> f32 {
        let mp_a = query.get(self.mp_a).unwrap();
        let mp_b = query.get(self.mp_b).unwrap();

        let force = self.stiffness * (mp_b.position.distance(mp_a.position) - self.rest_length);

        let corresponding_velocity_difference = mp_b.velocity - mp_a.velocity;

        let normalized_direction_vector = (mp_b.position - mp_a.position).normalize_or_zero();

        let dot_product = corresponding_velocity_difference.dot(normalized_direction_vector);

        //Total spring force
        force + dot_product * self.damping_factor
    }

    fn move_mass_points(&mut self, query: &mut Query<&mut MassPoint>) {
        let total_spring_force = self.get_force(query);

        let mp_arr = query.get_many_mut([self.mp_a, self.mp_b]).unwrap();

        let mut mp_a;
        let mut mp_b;

        match mp_arr {
            [a, b] => {
                mp_a = a;
                mp_b = b
            }
        }

        mp_a.force =
            mp_a.force + total_spring_force * (mp_b.position - mp_a.position).normalize_or_zero();
        mp_b.force =
            mp_b.force + total_spring_force * (mp_a.position - mp_b.position).normalize_or_zero();
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

    pub fn resolve_collisons(
        mut shapes_query: Query<&mut Shape>,
        mut points_query: Query<&mut MassPoint>,
    ) {
        let mut combinations = shapes_query.iter_combinations_mut();
        while let Some([mut a, mut b]) = combinations.fetch_next() {}
    }

    fn check_for_collision(shape_a: &Shape, shape_b: &Shape, query: &Query<&MassPoint>) -> bool {
        let mut collision = false;
        let b_length = shape_b.points.len();

        for point in query.iter_many(&shape_a.points) {
            let mut shape_b = query.iter_many(&shape_b.points);

            let mut next = 0;
            for current in 0..b_length {
                let current_vertice = shape_b
                    .nth(current)
                    .expect("THe vertice does not exist :(")
                    .position;
                let next_vertice = shape_b
                    .nth(next)
                    .expect("THe vertice does not exist :( 2")
                    .position;

                let point_position = point.position;

                //Check if the point is inside the shape utilising black magic. 
                if (((current_vertice.y > point_position.y) != (next_vertice.y > point_position.y))
                    && (point_position.x
                        < (next_vertice.x - current_vertice.x)
                            * (point_position.y - current_vertice.y)
                            / (next_vertice.y - current_vertice.y)
                            + current_vertice.x))
                {
                    collision = !collision;
                }

                //Get the next vertice in shape and wrap around to zero if we hit the end
                next = current + 1;
            }
        }

        collision
    }
}
