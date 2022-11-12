use nannou::prelude::*;
use std::ops::Add;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::RefreshSync)
        .run();
}

const DEG_PERIOD: usize = 360;

struct Model {
    _window: window::Id,
    points: Vec<Vec2>,
    wheel: Wheel,
    inner: Circle,
    outer: Circle,
    delta_theta: f32,
    theta: f32,
    clobber: f32,
    first_frame: bool,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    let mut outer = Circle::new(200.0);
    outer.set_color_func(&Circle::white);

    let radius_ratio = 2.0 / 3.0;

    let inner = Circle::new((1.0 - radius_ratio) * outer.radius);

    let wheel = Wheel::new(0.0, outer.radius - inner.radius, inner.pt_at(0.0), 1.5);
    Model {
        _window,
        wheel,
        points: Vec::<Vec2>::new(),
        inner,
        outer,
        delta_theta: 0.5,
        theta: 0.0,
        clobber: 0.003,
        first_frame: true,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    _model.theta = _model.theta + _model.delta_theta;
    let distance = _model.delta_theta * _model.outer.circumference() / (DEG_PERIOD as f32);
    let position = _model.inner.pt_at(_model.theta);

    //println!("distance: {}\nposition: {:?}\nroll {}", distance, position, _model.wheel.roll_phase);
    _model.wheel.roll(distance, position);

    _model.points.append(&mut vec![_model.wheel.pen_location()]);
    if _model.first_frame == true {
        _model.first_frame = false;
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();

    if _model.first_frame {
        // on the first frame, draw a black background
        draw.background().color(BLACK);
    } else {
        // otherwise fade out the old drawing by the clobber amount
        let r = app.window_rect();
        draw.rect()
            .xy(r.xy())
            .wh(r.wh())
            .color(rgba(0.0, 0.0, 0.0, _model.clobber));
    }

    _model.wheel.draw(&draw);

    _model.wheel.draw_guides(&draw);

    draw.to_frame(app, &frame).unwrap();
}

struct Circle {
    radius: f32,
    centre: Vec2,
    color_func: &'static dyn Fn(f32) -> Rgba<f32>,
}

#[allow(dead_code)]
impl Circle {
    fn white(_: f32) -> Rgba<f32> {
        rgba::<f32>(1.0, 1.0, 1.0, 0.001)
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
        let alpha = 0.5;

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
        (self.pt_at(degs), (self.color_func)(degs.to_radians()))
        //(self.pt_at(degs), rgba(0.3, 0.3, 0.6, 1.0))
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
    pen_offset: f32,
}

impl Wheel {
    fn new(initial_roll_phase: f32, radius: f32, location: Vec2, pen_offset: f32) -> Wheel {
        Wheel {
            roll_phase: initial_roll_phase,
            geometry: Circle::new_at(location, radius),
            pen_offset,
        }
    }

    fn roll(&mut self, distance: f32, end: Vec2) {
        self.geometry.centre = end;
        let angle_traversed = 2.0 * PI * distance / self.geometry.circumference();
        self.roll_phase = self.roll_phase - angle_traversed;
    }

    fn pen_location(&self) -> Vec2 {
        self.geometry.centre.lerp(
            self.geometry.pt_at(self.roll_phase.to_degrees()),
            self.pen_offset,
        )
    }

    fn draw_guides(&self, draw: &Draw) {
        self.geometry.draw(draw);
        let edge = self.geometry.pt_at(self.roll_phase.to_degrees());
        draw.line()
            .stroke_weight(1.0)
            .start_cap_round()
            .color(rgba(0.1, 0.6, 0.8, 0.1))
            .start(self.pen_location())
            .end(edge);
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.pen_location())
            .w_h(5.0, 5.0)
            .color(Circle::rainbow(self.roll_phase.to_degrees()));
    }
}
