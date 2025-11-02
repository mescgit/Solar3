use bevy::prelude::*;

#[derive(Clone, Copy)]
pub struct Quad {
    pub center: Vec2,
    pub half_size: f32,
}
impl Quad {
    pub fn new(center: Vec2, half_size: f32) -> Self { Self { center, half_size } }
    pub fn contains(&self, p: Vec2) -> bool {
        (p.x >= self.center.x - self.half_size) &&
        (p.x <= self.center.x + self.half_size) &&
        (p.y >= self.center.y - self.half_size) &&
        (p.y <= self.center.y + self.half_size)
    }
    pub fn subdivide(&self) -> [Quad; 4] {
        let hs = self.half_size * 0.5;
        [
            Quad::new(self.center + Vec2::new(-hs,  hs), hs), // NW
            Quad::new(self.center + Vec2::new( hs,  hs), hs), // NE
            Quad::new(self.center + Vec2::new(-hs, -hs), hs), // SW
            Quad::new(self.center + Vec2::new( hs, -hs), hs), // SE
        ]
    }
    pub fn size(&self) -> f32 { self.half_size * 2.0 }
}

pub enum Node {
    Empty(Quad),
    Leaf { quad: Quad, pos: Vec2, mass: f32 },
    Internal { quad: Quad, mass: f32, com: Vec2, children: [Box<Node>; 4] },
}

pub struct QuadTree {
    pub root: Box<Node>,
}

impl QuadTree {
    pub fn new(bounds: Quad) -> Self { Self { root: Box::new(Node::Empty(bounds)) } }
    pub fn insert(&mut self, p: Vec2, mass: f32) { Self::insert_node(&mut self.root, p, mass); }

    fn insert_node(node: &mut Box<Node>, p: Vec2, mass: f32) {
        match node.as_mut() {
            Node::Empty(q) => {
                if !q.contains(p) { return; }
                *node = Box::new(Node::Leaf { quad: *q, pos: p, mass });
            }
            Node::Leaf { quad, pos, mass: m } => {
                let quads = quad.subdivide();
                let mut children: [Box<Node>; 4] = quads.map(|q| Box::new(Node::Empty(q)));
                Self::insert_node(&mut children[Self::child_index(*pos, *quad)], *pos, *m);
                Self::insert_node(&mut children[Self::child_index(p, *quad)], p, mass);
                *node = Box::new(Node::Internal { quad: *quad, mass: 0.0, com: Vec2::ZERO, children });
            }
            Node::Internal { quad, children, .. } => {
                let idx = Self::child_index(p, *quad);
                Self::insert_node(&mut children[idx], p, mass);
            }
        }
    }

    fn child_index(p: Vec2, quad: Quad) -> usize {
        let right = (p.x > quad.center.x) as usize;
        let top = (p.y > quad.center.y) as usize;
        (top << 1) | right
    }

    pub fn build_mass_centers(&mut self) {
        fn compute(node: &mut Node) -> (f32, Vec2) {
            match node {
                Node::Empty(_) => (0.0, Vec2::ZERO),
                Node::Leaf { mass, pos, .. } => (*mass, *pos),
                Node::Internal { children, mass, com, .. } => {
                    let mut total_m = 0.0;
                    let mut weighted = Vec2::ZERO;
                    for c in children.iter_mut() {
                        let (m, p) = compute(c);
                        total_m += m;
                        weighted += p * m;
                    }
                    *mass = total_m.max(0.0);
                    *com = if total_m > 0.0 { weighted / total_m } else { Vec2::ZERO };
                    (*mass, *com)
                }
            }
        }
        compute(&mut self.root);
    }

    pub fn get_density_factor(&self, p: Vec2) -> f32 {
        const MAX_DEPTH: u32 = 12;
        let mut node = &self.root;
        let mut depth = 0;
        while let Node::Internal { quad, children, .. } = node.as_ref() {
            if !quad.contains(p) || depth >= MAX_DEPTH {
                break;
            }
            let idx = Self::child_index(p, *quad);
            node = &children[idx];
            depth += 1;
        }
        (depth as f32 / MAX_DEPTH as f32).min(1.0)
    }

    pub fn approx_acc(&self, p: Vec2, g: f32, theta: f32, soft2: f32) -> Vec2 {
        fn walk(node: &Node, p: Vec2, g: f32, theta2: f32, soft2: f32) -> Vec2 {
            match node {
                Node::Empty(_) => Vec2::ZERO,
                Node::Leaf { pos, mass, .. } => {
                    let r = *pos - p;
                    let dist2 = r.length_squared() + soft2;
                    if dist2 == 0.0 { return Vec2::ZERO; }
                    let inv = 1.0 / dist2.sqrt().powi(3);
                    g * *mass * r * inv
                }
                Node::Internal { quad, mass, com, children } => {
                    if *mass == 0.0 { return Vec2::ZERO; }
                    let r = *com - p;
                    let d = r.length();
                    let s = quad.size();
                    if d == 0.0 {
                        let mut a = Vec2::ZERO;
                        for c in children.iter() { a += walk(c, p, g, theta2, soft2); }
                        return a;
                    }
                    if (s * s) / (d * d) < theta2 {
                        let dist2 = d * d + soft2;
                        let inv = 1.0 / dist2.sqrt().powi(3);
                        return g * *mass * r * inv;
                    } else {
                        let mut a = Vec2::ZERO;
                        for c in children.iter() { a += walk(c, p, g, theta2, soft2); }
                        return a;
                    }
                }
            }
        }
        walk(&self.root, p, g, theta * theta, soft2)
    }
}
