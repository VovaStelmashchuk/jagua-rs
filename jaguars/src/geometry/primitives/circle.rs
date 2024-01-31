use std::cmp::Ordering;
use std::f64::consts::PI;

use crate::geometry::geo_enums::GeoPosition;
use crate::geometry::geo_traits::{CollidesWith, DistanceFrom, Shape, Transformable, TransformableFrom};
use crate::geometry::primitives::aa_rectangle::AARectangle;
use crate::geometry::primitives::edge::Edge;
use crate::geometry::primitives::point::Point;
use crate::geometry::transformation::Transformation;

#[derive(Clone, Debug, PartialEq)]
pub struct Circle {
    center: Point,
    radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64) -> Self {
        if !radius.is_finite() || radius < 0.0 {
            panic!("invalid circle radius: {}", radius);
        }
        if !center.0.is_finite() || !center.1.is_finite() {
            panic!("invalid circle center: {:?}", center);
        }
        Self { center, radius }
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn bounding_circle(circles: &[Circle]) -> Circle {
        //Returns the smallest possible circle that fully contains all in circles (including self)
        let mut circles_iter = circles.iter();
        let mut bounding_circle = circles_iter.next().expect("no circles provided").clone();

        for circle in circles_iter {
            let distance_between_centers = bounding_circle.center().distance(&circle.center());
            match bounding_circle.radius.partial_cmp(&(distance_between_centers + circle.radius)).unwrap() {
                Ordering::Less => {
                    //bounding circle needs to expand
                    let edge = Edge::new(bounding_circle.center(), circle.center())
                        .extend_at_front(bounding_circle.radius())
                        .extend_at_back(circle.radius());

                    let new_radius = edge.diameter() / 2.0;
                    let new_center = edge.centroid();

                    bounding_circle = Circle::new(new_center, new_radius);
                }
                _ => ()
            }
        }
        bounding_circle
    }
}

impl Transformable for Circle {
    fn transform(&mut self, t: &Transformation) -> &mut Self {
        let Circle { center, radius: _ } = self;
        center.transform(t);
        self
    }
}

impl TransformableFrom for Circle {
    fn transform_from(&mut self, reference: &Self, t: &Transformation) -> &mut Self {
        let Circle { center, radius: _ } = self;
        center.transform_from(&reference.center, t);
        self
    }
}

impl CollidesWith<Circle> for Circle {
    fn collides_with(&self, other: &Circle) -> bool {
        let (cx1, cx2) = (self.center.0, other.center.0);
        let (cy1, cy2) = (self.center.1, other.center.1);
        let (r1, r2) = (self.radius, other.radius);

        let dx = cx1 - cx2;
        let dy = cy1 - cy2;
        let sq_d = dx * dx + dy * dy;

        sq_d <= (r1 + r2) * (r1 + r2)
    }
}

impl CollidesWith<Edge> for Circle {
    fn collides_with(&self, edge: &Edge) -> bool {
        edge.sq_distance(&self.center) <= self.radius.powi(2)
    }
}

impl CollidesWith<AARectangle> for Circle {
    #[inline(always)]
    fn collides_with(&self, rect: &AARectangle) -> bool {
        //Based on: https://yal.cc/rectangle-circle-intersection-test/

        //TODO: benchmark this against approach which first checks only center

        let Point(c_x, c_y) = self.center;

        let nearest_x = f64::max(rect.x_min(), f64::min(c_x, rect.x_max()));
        let nearest_y = f64::max(rect.y_min(), f64::min(c_y, rect.y_max()));

        (nearest_x - c_x).powi(2) + (nearest_y - c_y).powi(2) <= self.radius.powi(2)
    }
}

impl CollidesWith<Point> for Circle {
    fn collides_with(&self, point: &Point) -> bool {
        point.sq_distance(&self.center()) <= self.radius.powi(2)
    }
}

impl DistanceFrom<Point> for Circle {
    fn sq_distance(&self, other: &Point) -> f64 {
        self.distance(other).powi(2)
    }

    fn distance(&self, point: &Point) -> f64 {
        let Point(x, y) = point;
        let Point(cx, cy) = self.center;
        let sq_d = (x - cx).powi(2) + (y - cy).powi(2);
        return if sq_d < self.radius.powi(2) {
            0.0 //point is inside circle
        } else {
            //point is outside circle
            f64::sqrt(sq_d) - self.radius
        };
    }

    fn distance_from_border(&self, point: &Point) -> (GeoPosition, f64) {
        let Point(x, y) = point;
        let Point(cx, cy) = self.center;
        let d_center = f64::sqrt((x - cx).powi(2) + (y - cy).powi(2));
        match d_center.partial_cmp(&self.radius).unwrap() {
            Ordering::Less | Ordering::Equal => (GeoPosition::Interior, self.radius - d_center),
            Ordering::Greater => (GeoPosition::Exterior, d_center - self.radius)
        }
    }

    fn sq_distance_from_border(&self, point: &Point) -> (GeoPosition, f64) {
        let (pos, distance) = self.distance_from_border(point);
        (pos, distance.powi(2))
    }
}

impl Shape for Circle {
    fn centroid(&self) -> Point {
        self.center.clone()
    }

    fn area(&self) -> f64 {
        self.radius * self.radius * PI
    }

    fn bbox(&self) -> AARectangle {
        let (r, x, y) = (self.radius, self.center.0, self.center.1);
        AARectangle::new(x - r, y - r, x + r, y + r)
    }

    fn diameter(&self) -> f64 {
        self.radius * 2.0
    }
}