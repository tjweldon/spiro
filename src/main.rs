use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use std::ops::{Add, Mul};

fn main() {
    nannou::app(init)
        .update(update)
        .loop_mode(LoopMode::RefreshSync)
        .run();
}

const RADIUS_RATIO: f32 = 2.0 / 3.0;
const PEN_OFFSET: f32 = 1.5;
const OUTER_RADIUS: f32 = 200.0;

struct Model {
    _window: window::Id,
    clobber: f32,
    spiro: Spiro,
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

    let spiro = Spiro::new(
        OUTER_RADIUS * RADIUS_RATIO,
        OUTER_RADIUS,
        PEN_OFFSET,
        (0.5).to_radians(),
    );

    let egui = Egui::from_window(&app.window(_window).unwrap());

    Model {
        _window,
        spiro,
        clobber: 0.003,
        first_frame: true,
        egui,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let spiro = &mut _model.spiro;
    spiro.update();

    if _model.first_frame == true {
        _model.first_frame = false;
    }

    let egui = &mut _model.egui;
    egui.set_elapsed_time(_update.since_start);
    let ctx = egui.begin_frame();

    // begin variables exposed via the ui
    let clobber = &mut _model.clobber;

    egui::Window::new("Controls").show(&ctx, |ui| {
        ui.label("fade");
        ui.add(egui::Slider::new(clobber, 0.0..=0.01));

        ui.label("pen offset");
        ui.add(egui::Slider::new(&mut spiro.pen_offset, -4.0..=4.0));

        ui.label("wheel radius");
        ui.add(egui::Slider::new(&mut spiro.wheel_radius, 1.0..=spiro.outer_radius));
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

    model.spiro.draw(&draw);

    model.spiro.draw_guides(&draw);

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

struct Spiro {
    wheel_radius: f32,
    theta: f32,
    pen_offset: f32,
    outer_radius: f32,
    delta_theta: f32,
}

impl Spiro {
    fn new(wheel_radius: f32, outer_radius: f32, pen_offset: f32, delta_theta: f32) -> Spiro {
        Spiro {
            wheel_radius,
            theta: 0.0,
            pen_offset,
            outer_radius,
            delta_theta,
        }
    }

    fn get_wheel_theta(&self) -> f32 {
        -self.theta * self.outer_radius / self.wheel_radius
    }

    fn get_wheel_centre(&self) -> Vec2 {
        let wheel_path_radius = self.outer_radius - self.wheel_radius;

        vec2(
            wheel_path_radius * self.theta.cos(),
            wheel_path_radius * self.theta.sin(),
        )
    }

    fn get_spoke_end(&self) -> Vec2 {
        vec2(self.get_wheel_theta().cos(), self.get_wheel_theta().sin())
            .mul(self.wheel_radius)
            .add(self.get_wheel_centre())
    }

    fn get_arm(&self) -> (Vec2, Vec2) {
        (self.get_wheel_centre(), self.pen_location())
    }

    fn pen_location(&self) -> Vec2 {
        self.get_wheel_centre()
            .lerp(self.get_spoke_end(), self.pen_offset)
    }

    fn update(&mut self) {
        self.theta = self.theta + self.delta_theta;
    }

    fn get_wheel_circle(&self) -> Circle {
        Circle {
            radius: self.wheel_radius,
            centre: self.get_wheel_centre(),
            color_func: &Circle::rainbow,
        }
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.pen_location())
            .w_h(5.0, 5.0)
            .color(Circle::rainbow(self.get_wheel_theta().to_degrees()));
    }

    fn draw_guides(&self, draw: &Draw) {
        let mut wheel = self.get_wheel_circle();
        wheel.set_color_func(&Circle::white);
        wheel.draw(draw);

        let (start, end) = self.get_arm();
        draw.line()
            .stroke_weight(1.0)
            .start_cap_round()
            .color(rgba(0.1, 0.6, 0.8, 0.1))
            .start(start)
            .end(end);
    }
}
