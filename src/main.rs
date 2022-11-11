use nannou::prelude::*;
use std::ops::Add;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::Wait)
        .run();
}

struct Model {
    _window: window::Id,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    Model { _window }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let mut outer = Circle::new(300.0);
    outer.set_color_func(&Circle::white);
    outer.draw(&draw);

    let radius_ratio = 1.0 / (2.0 * PI);

    let inner = Circle::new((1.0 - radius_ratio) * outer.radius);
    //for i in 0..90 {
    //    let c = Circle::new_at(inner.pt_at((i * 4) as f32), outer.radius - inner.radius);
    //    c.draw(&draw);
    //}

    let mut wheel = Wheel::new(0.0, outer.radius - inner.radius, inner.pt_at(0.0));

    let delta_theta = 3;
    for i in 0..360 {
        if i % delta_theta != 0 {
            continue;
        }
        wheel.roll(
            (delta_theta as f32) * outer.circumference() / 360.0,
            inner.pt_at(i as f32),
        );
        wheel.draw(&draw);
    }

    draw.to_frame(app, &frame).unwrap();
}

struct Circle {
    radius: f32,
    centre: Vec2,
    color_func: &'static dyn Fn(f32) -> Rgba<f32>,
}

impl Circle {
    fn white(_: f32) -> Rgba<f32> {
        rgba::<f32>(1.0, 1.0, 1.0, 1.0)
    }

    fn grey(_: f32) -> Rgba<f32> {
        rgba::<f32>(0.3, 0.3, 0.3, 1.0)
    }

    fn black(_: f32) -> Rgba<f32> {
        rgba::<f32>(0.0, 0.0, 0.0, 1.0)
    }

    fn invisible(_: f32) -> Rgba<f32> {
        rgba::<f32>(0.0, 0.0, 0.0, 0.0)
    }

    fn rainbow(degs: f32) -> Rgba<f32> {
        let angle = deg_to_rad(degs);
        let phase_angle = 2.0 * PI / 3.0;
        let intensity = 0.8;
        let alpha = 0.2;

        rgba(
            angle.sin() * intensity,
            (angle + phase_angle).sin() * intensity,
            (angle + 2.0 * phase_angle).sin() * intensity,
            alpha,
        )
    }

    fn new(radius: f32) -> Circle {
        Circle {
            radius,
            centre: pt2(0.0, 0.0),
            color_func: &Circle::white,
        }
    }

    fn new_at(centre: Vec2, radius: f32) -> Circle {
        Circle {
            radius,
            centre,
            color_func: &Circle::white,
        }
    }

    fn set_color_func(&mut self, f: &'static dyn Fn(f32) -> Rgba<f32>) {
        self.color_func = f;
    }

    fn pt_at(&self, degs: f32) -> Vec2 {
        let angle = deg_to_rad(degs);

        pt2(angle.cos() * self.radius, angle.sin() * self.radius).add(self.centre)
    }

    fn edge_at(&self, degs: f32) -> (Vec2, Rgba<f32>) {
        //(self.pt_at(degs), self.color_func(degs.to_radians()))
        (self.pt_at(degs), rgba(0.3, 0.3, 0.6, 1.0))
    }

    fn draw(&self, draw: &Draw) {
        draw.polyline()
            .stroke_weight(1.0)
            .points_colored_closed((0..360).map(|i| self.edge_at(i as f32)));
    }

    fn circumference(&self) -> f32 {
        2.0 * PI
            * match self.radius {
                x if !x.is_zero() => x,
                _ => 1.0,
            }
    }
}

struct Wheel {
    roll_phase: f32,
    geometry: Circle,
}

impl Wheel {
    fn new(initial_roll_phase: f32, radius: f32, location: Vec2) -> Wheel {
        Wheel {
            roll_phase: initial_roll_phase,
            geometry: Circle::new_at(location, radius),
        }
    }

    fn roll(&mut self, distance: f32, end: Vec2) {
        self.geometry.centre = end;
        let angle_traversed = 2.0 * PI * distance / self.geometry.circumference();
        self.roll_phase = self.roll_phase - angle_traversed;
    }

    fn draw(&self, draw: &Draw) {
        self.geometry.draw(draw);
        let (centre_point, edge) = (
            self.geometry.centre,
            self.geometry.pt_at(self.roll_phase.to_degrees()),
        );
        let in_between = centre_point.lerp(edge, 0.5);
        draw.line()
            .stroke_weight(4.0)
            .start_cap_round()
            .color(WHITE)
            .start(in_between)
            .end(edge);
    }
}
