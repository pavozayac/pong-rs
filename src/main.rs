extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate parry2d_f64;
extern crate piston;
extern crate piston_window;
extern crate rust_embed;
extern crate rusttype;

use glutin_window::GlutinWindow as Window;

use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use parry2d_f64::bounding_volume::{Aabb, BoundingVolume};
use parry2d_f64::math::Vector;
use parry2d_f64::na::{Isometry2, Matrix2, Point2, Vector2};
use parry2d_f64::query::{Ray, RayCast};
use parry2d_f64::shape::{Ball, Cuboid, Segment};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, Key, PressEvent, ReleaseEvent};
use rusttype::Font;

const PADDLE_SPEED: f64 = 300.0;
const PADDLE_WIDTH: f64 = 5.0;
const PADDLE_HEIGHT: f64 = 100.0;
const PONG_RADIUS: f64 = 5.0;
const PADDING: f64 = 20.0;
const PONG_SPEED: f64 = 200.0;
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
    left_score: u64,
    right_score: u64,
    left_y: f64,
    right_y: f64,
    left_key: Option<Key>,
    right_key: Option<Key>,
    pong: Pong,
    window_size: [f64; 2],
    border_left: Segment,
    border_right: Segment,
    border_top: Segment,
    border_bottom: Segment,
}

impl App {
    fn render(&mut self, args: &RenderArgs, gc: &mut GlyphCache) {
        use graphics::*;

        self.border_top =
            Segment::new(Point2::new(0.0, 0.0), Point2::new(self.window_size[0], 0.0));

        self.border_bottom = Segment::new(
            Point2::new(0.0, self.window_size[1] as f64),
            Point2::new(self.window_size[0], self.window_size[1]),
        );

        self.border_left = Segment::new(
            Point2::new(PADDING + PADDLE_WIDTH / 2.0, 0.0),
            Point2::new(PADDING + PADDLE_WIDTH / 2.0, self.window_size[1]),
        );

        self.border_right = Segment::new(
            Point2::new(self.window_size[0] - PADDING - PADDLE_WIDTH / 2.0, 0.0),
            Point2::new(
                self.window_size[0] - PADDING - PADDLE_WIDTH / 2.0,
                self.window_size[1],
            ),
        );

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
        let (lscore, rscore) = (self.left_score, self.right_score);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            let transform_left = c.transform.trans(x1_left, y1_left);
            let transform_right = c.transform.trans(x1_right, y1_right);
            let transform_pong = c.transform.trans(px - PONG_RADIUS, py - PONG_RADIUS);

            let left_text_tranform = c
                .transform
                .trans(args.window_size[0] / 2.0 - 50.0, PADDING * 2.0);
            let right_text_tranform = c
                .transform
                .trans(args.window_size[0] / 2.0 + 50.0, PADDING * 2.0);

            rectangle(WHITE, paddle, transform_left, gl);
            rectangle(WHITE, paddle2, transform_right, gl);
            ellipse(WHITE, pong_rect, transform_pong, gl);
            text::Text::new_color(WHITE, 20)
                .draw(
                    &lscore.to_string(),
                    gc,
                    &c.draw_state,
                    left_text_tranform,
                    gl,
                )
                .unwrap();
            text::Text::new_color(WHITE, 20)
                .draw(
                    &rscore.to_string(),
                    gc,
                    &c.draw_state,
                    right_text_tranform,
                    gl,
                )
                .unwrap();
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

        let pong_box: Aabb = pong_ball.aabb(&Isometry2::new(self.pong.pos, 0.0));

        let paddle_box_left: Aabb = paddle.aabb(&Isometry2::new(
            Vector2::new(
                PADDING + PADDLE_WIDTH / 2.0,
                self.left_y + PADDLE_HEIGHT / 2.0,
            ),
            0.0,
        ));
        let paddle_box_right: Aabb = paddle.aabb(&Isometry2::new(
            Vector2::new(
                self.window_size[0] - PADDING - PADDLE_WIDTH / 2.0,
                self.right_y + PADDLE_HEIGHT / 2.0,
            ),
            0.0,
        ));

        if paddle_box_left.intersects(&pong_box) {
            let ray_int = paddle_box_left.cast_local_ray_and_get_normal(
                &Ray::new(
                    Point2::from(self.pong.pos),
                    paddle_box_left.center() - Point2::from(self.pong.pos),
                ),
                100.0,
                true,
            );

            if let Some(int) = ray_int {
                println!("{:?}", int.normal);
                if int.normal == Vector2::new(1.0, 0.0) {
                    self.pong.vel = Matrix2::new(-1.0, 0.0, 0.0, 1.0) * self.pong.vel;
                }
            }
        }

        if paddle_box_right.intersects(&pong_box) {
            let ray_int = paddle_box_right.cast_local_ray_and_get_normal(
                &Ray::new(
                    Point2::from(self.pong.pos),
                    paddle_box_right.center() - Point2::from(self.pong.pos),
                ),
                100.0,
                true,
            );

            if let Some(int) = ray_int {
                println!("{:?}", int.normal);
                if int.normal == Vector2::new(-1.0, 0.0) {
                    self.pong.vel = Matrix2::new(-1.0, 0.0, 0.0, 1.0) * self.pong.vel;
                }
            }
        }

        // if paddle_box_left.intersects(&pong_box) || paddle_box_right.intersects(&pong_box) {
        //     self.pong.vel = Matrix2::new(-1.0, 0.0, 0.0, 1.0) * self.pong.vel;
        // }

        if paddle_box_left.intersects(&pong_box) {
            match self.left_key {
                Some(Key::W) => self.pong.vel = Vector2::new(self.pong.vel.x, -PONG_SPEED),
                Some(Key::S) => self.pong.vel = Vector2::new(self.pong.vel.x, PONG_SPEED),
                Some(_) => {}
                None => {}
            }
        }

        if paddle_box_right.intersects(&pong_box) {
            match self.right_key {
                Some(Key::Up) => self.pong.vel = Vector2::new(self.pong.vel.x, -PONG_SPEED),
                Some(Key::Down) => self.pong.vel = Vector2::new(self.pong.vel.x, PONG_SPEED),
                Some(_) => {}
                None => {}
            }
        }

        if pong_box.intersects(&self.border_bottom.local_aabb())
            || pong_box.intersects(&self.border_top.local_aabb())
        {
            self.pong.vel = Matrix2::new(1.0, 0.0, 0.0, -1.0) * self.pong.vel;
        }

        if pong_box.intersects(&self.border_left.local_aabb()) {
            self.pong.pos = Vector2::new(self.window_size[0] / 2.0, self.window_size[1] / 2.0);
            self.pong.vel = Vector2::new(150.0, 0.0);
            self.right_score += 1;
        }

        if pong_box.intersects(&self.border_right.local_aabb()) {
            self.pong.pos = Vector2::new(self.window_size[0] / 2.0, self.window_size[1] / 2.0);
            self.pong.vel = Vector2::new(-PONG_SPEED, 0.0);
            self.left_score += 1;
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

    let font_data = include_bytes!("../assets/FiraSans-Regular.ttf");
    let font: Font = Font::try_from_bytes(font_data).unwrap();
    let mut gc = GlyphCache::from_font(font, (), TextureSettings::new());

    let mut app = App {
        gl: GlGraphics::new(opengl),
        left_score: 0,
        right_score: 0,
        left_y: 200.0,
        right_y: 200.0,
        left_key: None,
        right_key: None,
        pong: Pong {
            pos: [
                window.window.inner_size().width as f64 / 2.0,
                window.window.inner_size().height as f64 / 2.0,
            ]
            .into(),
            vel: [PONG_SPEED, 0.0].into(),
        },
        window_size: [
            window.window.inner_size().width as f64,
            window.window.inner_size().height as f64,
        ],
        border_top: Segment::new(
            Point2::new(0.0, 0.0),
            Point2::new(window.window.inner_size().width as f64, 0.0),
        ),
        border_bottom: Segment::new(
            Point2::new(0.0, window.window.inner_size().height as f64),
            Point2::new(
                window.window.inner_size().width as f64,
                window.window.inner_size().height as f64,
            ),
        ),
        border_left: Segment::new(
            Point2::new(0.0, 0.0),
            Point2::new(0.0, window.window.inner_size().height as f64),
        ),
        border_right: Segment::new(
            Point2::new(window.window.inner_size().width as f64, 0.0),
            Point2::new(
                window.window.inner_size().width as f64,
                window.window.inner_size().height as f64,
            ),
        ),
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
            app.render(&args, &mut gc);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
