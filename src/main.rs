use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use std::ops::Add;

fn main() {
    nannou::app(init)
        .update(update)
        .loop_mode(LoopMode::RefreshSync)
        .run();
}

const DEG_PERIOD: usize = 360;
const RADIUS_RATIO: f32 = 2.0 / 3.0;
const PEN_OFFSET: f32 = 1.5;
const OUTER_RADIUS: f32 = 200.0;

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
    egui: Egui,
}

fn init(app: &App) -> Model {
    let _window = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let initial_roll = 0.0;
    let initial_theta = 0.0;

    let (inner, outer, wheel) = init_spiro(
        RADIUS_RATIO,
        PEN_OFFSET,
        OUTER_RADIUS,
        initial_roll,
        initial_theta,
    );

    let egui = Egui::from_window(&app.window(_window).unwrap());

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
        egui,
    }
}

fn init_spiro(
    radius_ratio: f32,
    pen_offset: f32,
    outer_radius: f32,
    initial_roll: f32,
    initial_theta: f32,
) -> (Circle, Circle, Wheel) {
    let outer = Circle::new(outer_radius);
    let inner = Circle::new((1.0 - radius_ratio) * outer_radius);
    let wheel = Wheel::new(
        initial_roll,
        outer.radius - inner.radius,
        inner.pt_at(initial_theta),
        pen_offset,
    );

    (inner, outer, wheel)
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    _model.theta = _model.theta + _model.delta_theta;
    let distance = _model.delta_theta * _model.outer.circumference() / (DEG_PERIOD as f32);
    let position = _model.inner.pt_at(_model.theta);

    _model.wheel.roll(distance, position);

    _model.points.append(&mut vec![_model.wheel.pen_location()]);
    if _model.first_frame == true {
        _model.first_frame = false;
    }

    let egui = &mut _model.egui;
    egui.set_elapsed_time(_update.since_start);
    let ctx = egui.begin_frame();

    // begin variables exposed via the ui
    let clobber = &mut _model.clobber;
    let wheel = &mut _model.wheel;

    egui::Window::new("Controls").show(&ctx, |ui| {
        ui.label("fade");
        ui.add(egui::Slider::new(clobber, 0.0..=0.01));

        ui.label("pen offset");
        ui.add(egui::Slider::new(&mut wheel.pen_offset, -2.0..=2.0));
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    if model.first_frame {
        // on the first frame, draw a black background
        draw.background().color(BLACK);
    } else {
        // otherwise fade out the old drawing by the clobber amount
        let r = app.window_rect();
        draw.rect()
            .xy(r.xy())
            .wh(r.wh())
            .color(rgba(0.0, 0.0, 0.0, model.clobber));
    }

    model.wheel.draw(&draw);

    model.wheel.draw_guides(&draw);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
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
        draw.line()
            .stroke_weight(1.0)
            .start_cap_round()
            .color(rgba(0.1, 0.6, 0.8, 0.1))
            .start(self.pen_location())
            .end(self.geometry.centre);
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.pen_location())
            .w_h(5.0, 5.0)
            .color(Circle::rainbow(self.roll_phase.to_degrees()));
    }
}

#[allow(dead_code)]
struct Spiro {
    wheel_radius: f32,
    wheel_theta: f32,
    theta: f32,
    pen_offset: f32,
    outer_radius: f32,
    delta_theta: f32,
}
