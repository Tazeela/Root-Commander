pub struct Point {
    pub x_coord: f32,
    pub y_coord: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x_coord: x,
            y_coord: y,
        }
    }
}

// Calculate angle function produces radians, need to convert that to degrees
const RAD2DEG: f32 = 57.2957795130823209;

// Given 3 points calculate the center of the circle and the radius
pub fn calculate_radius_and_center(p1: &Point, p2: &Point, p3: &Point) -> (Point, f32) {
    let x12 = p1.x_coord - p2.x_coord;
    let x13 = p1.x_coord - p3.x_coord;

    let y12 = p1.y_coord - p2.y_coord;
    let y13 = p1.y_coord - p3.y_coord;

    let y31 = p3.y_coord - p1.y_coord;
    let y21 = p2.y_coord - p1.y_coord;

    let x31 = p3.x_coord - p1.x_coord;
    let x21 = p2.x_coord - p1.x_coord;

    let sx13 = p1.x_coord.powf(2.0) - p3.x_coord.powf(2.0);
    let sy13 = p1.y_coord.powf(2.0) - p3.y_coord.powf(2.0);
    let sx21 = p2.x_coord.powf(2.0) - p1.x_coord.powf(2.0);
    let sy21 = p2.y_coord.powf(2.0) - p1.y_coord.powf(2.0);

    let f = ((sx13) * (x12) + (sy13) * (x12) + (sx21) * (x13) + (sy21) * (x13))
        / (2.0 * ((y31) * (x12) - (y21) * (x13)));
    let g = ((sx13) * (y12) + (sy13) * (y12) + (sx21) * (y13) + (sy21) * (y13))
        / (2.0 * ((x31) * (y12) - (x21) * (y13)));

    let c =
        -p1.x_coord.powf(2.0) - p1.y_coord.powf(2.0) - 2.0 * g * p1.x_coord - 2.0 * f * p1.y_coord;
    let h = -g;
    let k = -f;

    return (Point::new(h, k), (h.powf(2.0) + k.powf(2.0) - c).sqrt());
}

// Calculate the number of degrees (from straight up) this point is at.
// result is between -180 and 180
pub fn calculate_degrees_of_point(point: &Point, center: &Point) -> f32 {
    // this is the number of degrees from the right, need from the top
    let res = (point.y_coord - center.y_coord).atan2(point.x_coord - center.x_coord) * RAD2DEG;

    // TODO: Do I need it to be right facing?
    // Depending on the quadrant do fastest conversion
    if res < -90.0 {
        return -1.0 * (270.0 + res);
    } else if res <= 0.0 {
        return 90.0 - res;
    } else if res <= 90.0 {
        return 90.0 - res;
    } else {
        return -1.0 * (res - 90.0);
    }
}

// Given the points calculate the number of degrees to turn
// If we need to turn counter-clockwise returns negative number.
pub fn calculate_degrees_of_rotation(
    p1: &Point,
    p2: &Point,
    p3: &Point,
    center: &Point,
    is_final: bool,
) -> f32 {
    let degrees_p1 = calculate_degrees_of_point(p1, center);

    let degrees_difference_p1_p2 = calculate_degrees_of_point(p2, center) - degrees_p1;
    let degrees_difference_p1_p3 = calculate_degrees_of_point(p3, center) - degrees_p1;

    // to determine which direction to rotate
    let mut clockwise = false;

    // Calculate if we need to rotate clockwise or counterclockwise
    if degrees_difference_p1_p2 > 0.0 {
        clockwise =
            degrees_difference_p1_p3 > degrees_difference_p1_p2 || degrees_difference_p1_p3 < 0.0;
    } else if degrees_difference_p1_p2 < 0.0 {
        clockwise = degrees_difference_p1_p3 > degrees_difference_p1_p2;
    }

    let final_degrees = if is_final {
        degrees_difference_p1_p3
    } else {
        degrees_difference_p1_p2
    };
    return if clockwise && final_degrees < 0.0 {
        final_degrees + 360.0
    } else if !clockwise && final_degrees > 0.0 {
        final_degrees - 360.0
    } else {
        final_degrees
    };
}

// Calculate the angle from point to point
pub fn calculate_angle(p1: &Point, p2: &Point) -> f32 {
    // TODO: Remove the hardcoded values? Issues with precision?
    if p1.x_coord == p2.x_coord && p1.y_coord == p2.y_coord {
        return 0.0;
    } else if p1.x_coord == p2.x_coord {
        if p1.y_coord < p2.y_coord {
            return 0.0;
        } else {
            return 180.0;
        }
    } else if p1.y_coord == p2.y_coord {
        if p1.x_coord < p2.x_coord {
            return 90.0;
        } else {
            return -90.0;
        }
    } else {
        // if its not easy, we have to do the math
        return RAD2DEG
            * (((p2.x_coord - p1.x_coord) as f32).atan2((p2.y_coord - p1.y_coord) as f32));
    }
}

// Calculate the distance from point 1 to point 2
pub fn calculate_distance(p1: &Point, p2: &Point) -> f32 {
    ((p2.x_coord - p1.x_coord).powf(2.0) + (p2.y_coord - p1.y_coord).powf(2.0)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_calculate_distance() {
        // Special case doesnt move if we are already at destination
        assert_eq!(
            calculate_distance(&Point::new(100.0, 200.0), &Point::new(100.0, 200.0)),
            0.0
        );
        assert_eq!(
            calculate_distance(&Point::new(100.0, 200.0), &Point::new(200.0, 200.0)),
            100.0
        );
        assert_eq!(
            calculate_distance(&Point::new(100.0, 100.0), &Point::new(200.0, 200.0)),
            141.42136
        );
        assert_eq!(
            calculate_distance(&Point::new(200.0, 200.0), &Point::new(100.0, 100.0)),
            141.42136
        ); // make sure its positive regardless of direction
    }

    #[test]
    fn can_calculate_angle() {
        // Special case doesn't rotate if we are already at destination
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(0.0, 0.0)),
            0.0
        );
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(0.0, 10.0)),
            0.0
        );
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(10.0, 10.0)),
            45.0
        );
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(10.0, 0.0)),
            90.0
        );
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(0.0, -10.0)),
            180.0
        );
        assert_eq!(
            calculate_angle(&Point::new(0.0, 0.0), &Point::new(-10.0, 0.0)),
            -90.0
        );
    }

    #[test]
    fn can_calculate_degrees_of_point() {
        assert_eq!(
            calculate_degrees_of_point(&Point::new(10.0, 10.0), &Point::new(0.0, 0.0)),
            45.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(10.0, -10.0), &Point::new(0.0, 0.0)),
            135.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(-10.0, -10.0), &Point::new(0.0, 0.0)),
            -135.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(-10.0, 10.0), &Point::new(0.0, 0.0)),
            -45.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(10.0, 0.0), &Point::new(0.0, 0.0)),
            90.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(0.0, 10.0), &Point::new(0.0, 0.0)),
            0.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(-10.0, 0.0), &Point::new(0.0, 0.0)),
            -90.0
        );

        assert_eq!(
            calculate_degrees_of_point(&Point::new(0.0, -10.0), &Point::new(0.0, 0.0)),
            180.0
        );
    }

    #[test]
    fn can_calculate_degree_of_point() {
        assert_eq!(
            calculate_degrees_of_point(&Point::new(10.0, 10.0), &Point::new(0.0, 0.0)),
            45.0
        );
    }

    #[test]
    fn can_calculate_degrees_of_rotation() {
        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(-10.0, 0.0),
                &Point::new(0.0, 10.0),
                &Point::new(10.0, 0.0),
                &Point::new(0.0, 0.0),
                false
            ),
            90.0
        );

        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(10.0, 0.0),
                &Point::new(0.0, 10.0),
                &Point::new(-10.0, 0.0),
                &Point::new(0.0, 0.0),
                false
            ),
            -90.0
        );

        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(-10.0, 0.0),
                &Point::new(0.0, 10.0),
                &Point::new(10.0, 0.0),
                &Point::new(0.0, 0.0),
                true
            ),
            180.0
        );

        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(0.0, -10.0),
                &Point::new(0.0, 10.0),
                &Point::new(10.0, 0.0),
                &Point::new(0.0, 0.0),
                true
            ),
            270.0
        );

        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(-1.0, -1.0),
                &Point::new(0.0, 2.0),
                &Point::new(9.0, -1.0),
                &Point::new(4.0, -1.0),
                false
            ),
            36.869904
        );

        assert_eq!(
            calculate_degrees_of_rotation(
                &Point::new(-1.0, -1.0),
                &Point::new(8.0, -4.0),
                &Point::new(9.0, -1.0),
                &Point::new(4.0, -1.0),
                false
            ),
            -143.1301
        );
    }
}
