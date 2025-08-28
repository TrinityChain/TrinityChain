// This file will contain the geometric primitives like Point and Triangle.

// Placeholder types to make the code compile.
// These will be replaced with actual implementations later.
pub type TriangleId = u64;
pub type PublicKey = Vec<u8>;
#[derive(Debug, Clone)]
pub struct TriangleMetadata {}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>, // 3D support for AR/VR applications
}

impl Point {
    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
            z: match (&self.z, &other.z) {
                (Some(z1), Some(z2)) => Some((z1 + z2) / 2.0),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Point; 3],
    pub id: TriangleId,
    pub owner: Option<PublicKey>,
    pub subdivision_level: u8,
    pub parent: Option<TriangleId>,
    pub children: Vec<TriangleId>,
    pub metadata: TriangleMetadata,
}

impl Triangle {
    // NOTE: A basic `new` function is added to make the code from the README compile.
    pub fn new(vertices: [Point; 3], subdivision_level: u8, parent: Option<TriangleId>) -> Self {
        Self {
            vertices,
            id: 0, // Placeholder ID
            owner: None,
            subdivision_level,
            parent,
            children: vec![],
            metadata: TriangleMetadata {},
        }
    }

    /// Calculate area using the shoelace formula
    pub fn area(&self) -> f64 {
        let [a, b, c] = &self.vertices;
        0.5 * ((a.x * (b.y - c.y)) + (b.x * (c.y - a.y)) + (c.x * (a.y - b.y))).abs()
    }

    /// Calculate perimeter as sum of edge lengths
    pub fn perimeter(&self) -> f64 {
        let [a, b, c] = &self.vertices;
        a.distance_to(b) + b.distance_to(c) + c.distance_to(a)
    }

    /// Calculate centroid (geometric center)
    pub fn centroid(&self) -> Point {
        let [a, b, c] = &self.vertices;
        Point {
            x: (a.x + b.x + c.x) / 3.0,
            y: (a.y + b.y + c.y) / 3.0,
            z: match (&a.z, &b.z, &c.z) {
                (Some(z1), Some(z2), Some(z3)) => Some((z1 + z2 + z3) / 3.0),
                _ => None,
            },
        }
    }

    /// Check if a point lies within the triangle using barycentric coordinates
    pub fn contains_point(&self, point: &Point) -> bool {
        let [a, b, c] = &self.vertices;
        let denom = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);

        if denom.abs() < f64::EPSILON {
            return false; // Degenerate triangle
        }

        let alpha = ((b.y - c.y) * (point.x - c.x) + (c.x - b.x) * (point.y - c.y)) / denom;
        let beta = ((c.y - a.y) * (point.x - c.x) + (a.x - c.x) * (point.y - c.y)) / denom;
        let gamma = 1.0 - alpha - beta;

        alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0
    }

    /// Generate Sierpinski triangle subdivision
    pub fn sierpinski_subdivide(&self) -> [Triangle; 3] {
        let [a, b, c] = &self.vertices;
        let ab_mid = a.midpoint(b);
        let bc_mid = b.midpoint(c);
        let ca_mid = c.midpoint(a);

        [
            Triangle::new([a.clone(), ab_mid.clone(), ca_mid.clone()], self.subdivision_level + 1, Some(self.id)),
            Triangle::new([ab_mid, b.clone(), bc_mid.clone()], self.subdivision_level + 1, Some(self.id)),
            Triangle::new([ca_mid, bc_mid, c.clone()], self.subdivision_level + 1, Some(self.id)),
        ]
    }
}
