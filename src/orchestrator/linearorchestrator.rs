use crate::{irobot::root::{MarkerPosition, RootRobot}, utils::{Point, calculate_distance, calculate_angle, calculate_radius_and_center, calculate_degrees_of_rotation}};

pub struct LinearOrchestrator {
    current_x_coord: f32,
    current_y_coord: f32,
    current_heading: f32, // in degrees
}

impl LinearOrchestrator {
    pub fn new() -> LinearOrchestrator {
        LinearOrchestrator {
            current_x_coord: 0.0,
            current_y_coord: 0.0,
            current_heading: 0.0,
        }
    }

    // Rotate the robot to an exact heading
    async fn rotate_to_new_heading(&mut self, robot: &RootRobot, new_heading: f32) {
        if new_heading != self.current_heading {
            let mut rotation_amount = new_heading - self.current_heading;

            // Sometimes we can shortcut rotation in the other direction
            if rotation_amount > 180.0 {
                rotation_amount = rotation_amount - 360.0;
            } else if rotation_amount < -180.0 {
                rotation_amount = rotation_amount + 360.0;
            }

            robot.rotate_angle((rotation_amount * 10.0) as i32).await;

            self.current_heading = new_heading;
        }
    }

    // Move to a specified location
    async fn move_straight_line(
        &mut self,
        robot: &RootRobot,
        destination: &Point,
        marker_down: bool,
    ) {
        if destination.x_coord == self.current_x_coord
            && destination.y_coord == self.current_y_coord
        {
            // Already there no work needed
            return;
        }

        //calculate how to move from current location to new location
        let rotate_angle: f32 = calculate_angle(
            &Point::new(self.current_x_coord,
            self.current_y_coord),
            destination,
        );
        self.rotate_to_new_heading(robot, rotate_angle).await;

        let distance = calculate_distance(
            &Point::new(self.current_x_coord, self.current_y_coord),
            destination,
        );

        if distance != 0.0 {
            if marker_down {
                robot.set_marker_position(MarkerPosition::MarkerDown).await;
            }

            println!("Driving forward {}", distance);
            robot.drive_distance(distance.trunc() as i32).await;

            if marker_down {
                robot.set_marker_position(MarkerPosition::NothingDown).await;
            }
        }

        self.current_x_coord = destination.x_coord;
        self.current_y_coord = destination.y_coord;
    }

    // To smooth out arcs we calculate an arc between current point, next point and the point after
    // Then if there is going to be another point we only render it from this point to the next point.
    // If this is the final point then just draw it from first to end.
    async fn draw_arc(
        &mut self,
        robot: &RootRobot,
        start: &Point,
        mid: &Point,
        end: &Point,
        is_final: bool,
    ) {
        // calculate the center + radius
        let (center, radius) = calculate_radius_and_center(start, mid, end);
        let arc = calculate_degrees_of_rotation(start, mid, end, &center, is_final);

        // Rotate so we are facing perpindicular to center point (note that direction doesnt matter)
        self.rotate_to_new_heading(
            robot,
            calculate_angle(start, &center) - 90.0,
        )
        .await;

        let destination = if is_final { end } else { mid };

        println!(
            "Arc center is {},{}, radius is {} drawing for {} degrees",
            center.x_coord, center.y_coord, radius, arc
        );
        // actually draw
        robot.set_marker_position(MarkerPosition::MarkerDown).await;
        robot.drive_arc(arc as i32 * 10, radius as i32).await;
        robot.set_marker_position(MarkerPosition::NothingDown).await;

        // update the heading and coordinates
        self.current_x_coord = destination.x_coord;
        self.current_y_coord = destination.y_coord;
        self.current_heading = calculate_angle( destination, &center) - 90.0;
    }

    // Simple orchestrator which takes a set of lines (list of points) to draw
    pub async fn orchestrate(&mut self, robot: &RootRobot, points: Vec<Vec<Point>>) {
        for line in points.iter() {
            if line.len() == 1 {
                // if a vector has 1 point, draw a line straight to the point
                self.move_straight_line(robot, line.get(0).unwrap(), true)
                    .await;
            } else if line.len() == 2 {
                // if a vector has 2 points, move to the first point, then draw a line to the second
                self.move_straight_line(robot, line.get(0).unwrap(), false)
                    .await;
                self.move_straight_line(robot, line.get(1).unwrap(), true)
                    .await;
            } else if line.len() > 2 {
                // if a vector has 3 or more points, move to the first point, then draw an arc between lines
                self.move_straight_line(robot, line.get(0).unwrap(), false)
                    .await;
                let mut counter = 0;

                // Go through the arcs
                while counter + 3 <= line.len() {
                    self.draw_arc(
                        robot,
                        line.get(counter).unwrap(),
                        line.get(counter + 1).unwrap(),
                        line.get(counter + 2).unwrap(),
                        counter + 3 == line.len(),
                    )
                    .await;

                    counter += 1;
                }
            } else {
                // TODO: What should I do if I have more then 2, calculate best fit?
            }
        }
    }
}