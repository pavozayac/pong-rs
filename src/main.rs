extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate parry2d_f64;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use parry2d_f64::bounding_volume::BoundingVolume;
use parry2d_f64::math::Vector;
use parry2d_f64::na::{Isometry2, Matrix2, Vector2};
use parry2d_f64::shape::{Ball, Cuboid};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, Event, Input, Key, PressEvent, ReleaseEvent};

const PADDLE_SPEED: f64 = 300.0;
const PADDLE_WIDTH: f64 = 5.0;
const PADDLE_HEIGHT: f64 = 100.0;
const PONG_RADIUS: f64 = 10.0;
const PADDING: f64 = 20.0;
pub struct Pong {
    pos: Vector2<f64>,
    vel: Vector2<f64>,
}

impl Pong {
    fn update(&mut self, dt: f64) {
        self.pos += self.vel * dt;
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    left_y: f64,
    right_y: f64,
    left_key: Option<Key>,
    right_key: Option<Key>,
    pong: Pong,
    window_size: [f64; 2],
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.window_size = args.window_size;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let paddle = rectangle::rectangle_by_corners(0.0, 0.0, PADDLE_WIDTH, PADDLE_HEIGHT);
        let paddle2 = rectangle::rectangle_by_corners(0.0, 0.0, PADDLE_WIDTH, PADDLE_HEIGHT);
        let pong_rect =
            rectangle::rectangle_by_corners(0.0, 0.0, PONG_RADIUS * 2.0, PONG_RADIUS * 2.0);

        let (px, py) = (self.pong.pos[0], self.pong.pos[1]);
        let (x1_left, y1_left) = (PADDING, self.left_y);
        let (x1_right, y1_right) = (args.window_size[0] - PADDING - PADDLE_WIDTH, self.right_y);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            let transform_left = c.transform.trans(x1_left, y1_left);
            let transform_right = c.transform.trans(x1_right, y1_right);
            let transform_pong = c.transform.trans(px - PONG_RADIUS, py - PONG_RADIUS);

            rectangle(WHITE, paddle, transform_left, gl);
            rectangle(WHITE, paddle2, transform_right, gl);
            ellipse(WHITE, pong_rect, transform_pong, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        match self.left_key {
            Some(Key::W) => self.left_y -= PADDLE_SPEED * args.dt,
            Some(Key::S) => self.left_y += PADDLE_SPEED * args.dt,
            Some(_) => {}
            None => {}
        }

        match self.right_key {
            Some(Key::Up) => self.right_y -= PADDLE_SPEED * args.dt,
            Some(Key::Down) => self.right_y += PADDLE_SPEED * args.dt,
            Some(_) => {}
            None => {}
        }

        self.pong.update(args.dt);

        let pong_ball: Ball = Ball::new(PONG_RADIUS);

        let paddle: Cuboid = Cuboid::new(Vector::new(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0));

        let pong_box = pong_ball.aabb(&Isometry2::new(self.pong.pos, 0.0));

        let paddle_box_left = paddle.aabb(&Isometry2::new(
            Vector2::new(
                PADDING - PADDLE_WIDTH / 2.0,
                self.left_y + PADDLE_HEIGHT / 2.0,
            ),
            0.0,
        ));
        let paddle_box_right = paddle.aabb(&Isometry2::new(
            Vector2::new(
                self.window_size[0] - PADDING - PADDLE_WIDTH / 2.0,
                self.right_y + PADDLE_HEIGHT / 2.0,
            ),
            0.0,
        ));

        if paddle_box_left.intersects(&pong_box) || paddle_box_right.intersects(&pong_box) {
            self.pong.vel = Matrix2::new(-1.0, 0.0, 0.0, 1.0) * self.pong.vel;
        }

        if paddle_box_left.intersects(&pong_box) {
            match self.left_key {
                Some(Key::W) => self.pong.vel -= Vector2::new(0.0, 30.0),
                Some(Key::S) => self.pong.vel += Vector2::new(0.0, 30.0),
                Some(_) => {}
                None => {}
            }
        }

        if paddle_box_right.intersects(&pong_box) {
            match self.right_key {
                Some(Key::Up) => self.pong.vel -= Vector2::new(0.0, 30.0),
                Some(Key::Down) => self.pong.vel += Vector2::new(0.0, 30.0),
                Some(_) => {}
                None => {}
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("pong", [400, 400])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        left_y: 200.0,
        right_y: 200.0,
        left_key: None,
        right_key: None,
        pong: Pong {
            pos: [200.0, 200.0].into(),
            vel: [100.0, 3.0].into(),
        },
        window_size: [
            window.window.inner_size().width as f64,
            window.window.inner_size().height as f64,
        ],
    };

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Up => app.right_key = Some(Key::Up),
                    Key::Down => app.right_key = Some(Key::Down),
                    Key::W => app.left_key = Some(Key::W),
                    Key::S => app.left_key = Some(Key::S),
                    _ => {}
                }
            }
        }

        if let Some(button) = e.release_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Up | Key::Down => app.right_key = None,
                    Key::W | Key::S => app.left_key = None,
                    _ => {}
                }
            }
        }

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
