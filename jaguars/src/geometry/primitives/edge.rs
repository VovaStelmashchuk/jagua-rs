use crate::geometry::primitives::aa_rectangle::AARectangle;
use crate::geometry::geo_traits::{CollidesWith, DistanceFrom, Shape, Transformable, TransformableFrom};
use crate::geometry::primitives::point::Point;
use crate::geometry::geo_enums::GeoPosition;
use crate::geometry::transformation::Transformation;

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    start: Point,
    end: Point,
}

impl Edge {
    pub fn new(start: Point, end: Point) -> Self {
        if start == end {
            panic!("degenerate edge, {start:?} == {end:?}");
        }

        Edge { start, end }
    }

    pub fn extend_to_x(mut self, x: f64) -> Self {
        let (dx, dy) = (self.end.0 - self.start.0, self.end.1 - self.start.1);
        if dx != 0.0 {
            if (dx > 0.0 && x < self.start.0) || (dx < 0.0 && x > self.start.0) {
                //expand in the back
                self.start.0 = x;
                self.start.1 = self.end.1 + dy * (x - self.end.0) / dx;
            } else if dx > 0.0 && x > self.end.0 || dx < 0.0 && x < self.end.0 {
                //expand in front
                self.end.0 = x;
                self.end.1 = self.start.1 + dy * (x - self.start.0) / dx;
            }
        }
        self
    }

    pub fn extend_to_y(mut self, y: f64) -> Self {
        let (dx, dy) = (self.end.0 - self.start.0, self.end.1 - self.start.1);
        if dy != 0.0 {
            if y < self.start.1 {
                self.start.1 = y;
                self.start.0 = self.end.0 + dx * (y - self.end.1) / dy;
            } else if y > self.end.1 {
                self.end.1 = y;
                self.end.0 = self.start.0 + dx * (y - self.start.1) / dy;
            }
        }
        self
    }

    pub fn extend_at_front(mut self, d: f64) -> Self {
        //extend the line at the front by distance d
        let (dx, dy) = (self.end.0 - self.start.0, self.end.1 - self.start.1);
        let l = self.diameter();
        self.start.0 -= dx * (d / l);
        self.start.1 -= dy * (d / l);
        self
    }

    pub fn extend_at_back(mut self, d: f64) -> Self {
        //extend the line at the back by distance d
        let (dx, dy) = (self.end.0 - self.start.0, self.end.1 - self.start.1);
        let l = self.diameter();
        self.end.0 += dx * (d / l);
        self.end.1 += dy * (d / l);
        self
    }

    pub fn scale(mut self, factor: f64) -> Self {
        let (dx, dy) = (self.end.0 - self.start.0, self.end.1 - self.start.1);
        self.start.0 = self.start.0 - (dx * (factor - 1.0) / 2.0);
        self.start.1 = self.start.1 - (dy * (factor - 1.0) / 2.0);
        self.end.0 = self.end.0 + (dx * (factor - 1.0) / 2.0);
        self.end.1 = self.end.1 + (dy * (factor - 1.0) / 2.0);
        self
    }

    pub fn reverse(mut self) -> Self {
        std::mem::swap(&mut self.start, &mut self.end);
        self
    }

    pub fn collides_at(&self, other: &Edge) -> Option<Point> {
        match edge_intersection(self, other, true) {
            Intersection::No => None,
            Intersection::Yes(point) => Some(point.expect("Intersection::Yes, but returned no point when this was requested")),
        }
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn end(&self) -> Point {
        self.end
    }

    pub fn x_min(&self) -> f64 {
        f64::min(self.start.0, self.end.0)
    }

    pub fn y_min(&self) -> f64 {
        f64::min(self.start.1, self.end.1)
    }

    pub fn x_max(&self) -> f64 {
        f64::max(self.start.0, self.end.0)
    }

    pub fn y_max(&self) -> f64 {
        f64::max(self.start.1, self.end.1)
    }
}

impl Transformable for Edge {
    fn transform(&mut self, t: &Transformation) -> &mut Self {
        let Edge { start, end } = self;
        start.transform(t);
        end.transform(t);

        self
    }
}

impl TransformableFrom for Edge {
    fn transform_from(&mut self, reference: &Self, t: &Transformation) -> &mut Self {
        let Edge { start, end } = self;
        start.transform_from(&reference.start, t);
        end.transform_from(&reference.end, t);

        self
    }
}

impl Shape for Edge {
    fn centroid(&self) -> Point {
        Point(
            (self.start.0 + self.end.0) / 2.0,
            (self.start.1 + self.end.1) / 2.0,
        )
    }

    fn area(&self) -> f64 {
        0.0
    }

    fn bbox(&self) -> AARectangle {
        AARectangle::new(
            self.x_min(),
            self.y_min(),
            self.x_max(),
            self.y_max(),
        )
    }

    fn diameter(&self) -> f64 {
        self.start().distance(&self.end())
    }
}

impl DistanceFrom<Point> for Edge {
    fn sq_distance(&self, point: &Point) -> f64 {
        //from https://stackoverflow.com/a/6853926
        let Point(x1, y1) = self.start;
        let Point(x2, y2) = self.end;
        let Point(x, y) = point;

        let a = x - x1;
        let b = y - y1;
        let c = x2 - x1;
        let d = y2 - y1;

        let dot = a * c + b * d;
        let len_sq = c * c + d * d;
        let mut param = -1.0;
        if len_sq != 0.0 {
            param = dot / len_sq;
        }
        let (xx, yy) = match param {
            p if p < 0.0 => (x1, y1), //start is closest point
            p if p > 1.0 => (x2, y2), //end is closest point
            _ => (x1 + param * c, y1 + param * d) //closest point is on the edge
        };

        let (dx, dy) = (x - xx, y - yy);
        dx.powi(2) + dy.powi(2)
    }

    fn distance(&self, point: &Point) -> f64 {
        f64::sqrt(self.sq_distance(point))
    }

    fn distance_from_border(&self, point: &Point) -> (GeoPosition, f64) {
        (GeoPosition::Exterior, self.distance(point))
    }

    fn sq_distance_from_border(&self, point: &Point) -> (GeoPosition, f64) {
        (GeoPosition::Exterior, self.sq_distance(point))
    }
}

impl CollidesWith<Edge> for Edge {
    fn collides_with(&self, other: &Edge) -> bool {
        match edge_intersection(self, other, false) {
            Intersection::No => false,
            Intersection::Yes(_) => true,
        }
    }
}

fn edge_intersection(e1: &Edge, e2: &Edge, calculate_location: bool) -> Intersection {
    if f64::max(e1.x_min(), e2.x_min()) > f64::min(e1.x_max(), e2.x_max()) {
        return Intersection::No;
    }
    if f64::max(e1.y_min(), e2.y_min()) > f64::min(e1.y_max(), e2.y_max()) {
        return Intersection::No;
    }

    //https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line_segment
    let Point(x1, y1) = e1.start;
    let Point(x2, y2) = e1.end;
    let Point(x3, y3) = e2.start;
    let Point(x4, y4) = e2.end;

    let t_nom = (x2 - x4) * (y4 - y3) - (y2 - y4) * (x4 - x3);
    let t_denom = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);

    let t_nom = t_nom;
    let t_denom = t_denom;

    if t_denom == 0.0 {
        //parallel edges
        return Intersection::No;
    } else if t_denom > 0.0 {
        if t_nom < 0.0 || t_nom > t_denom {
            return Intersection::No;
        }
    } else {
        if t_nom > 0.0 || t_nom < t_denom {
            return Intersection::No;
        }
    }

    let u_nom = (x2 - x4) * (y2 - y1) - (y2 - y4) * (x2 - x1);
    let u_denom = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);

    let u_nom = u_nom;
    let u_denom = u_denom;

    if u_denom == 0.0 {
        //parallel edges
        return Intersection::No;
    } else if u_denom > 0.0 {
        if u_nom < 0.0 || u_nom > u_denom {
            return Intersection::No;
        }
    } else {
        if u_nom > 0.0 || u_nom < u_denom {
            return Intersection::No;
        }
    }

    match calculate_location {
        true => {
            let t: f64 = (t_nom / t_denom).into();
            Intersection::Yes(Some(Point(x2 + t * (x1 - x2), y2 + t * (y1 - y2))))
        }
        false => Intersection::Yes(None),
    }
}

enum Intersection {
    Yes(Option<Point>),
    No,
}
