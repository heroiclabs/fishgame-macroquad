use macroquad::{
    camera::Camera2D,
    math::{vec2, Rect, Vec2},
    window::{screen_height, screen_width},
};

pub struct Camera {
    bounds: Rect,
    viewport_height: f32,
    follow_buffer: Vec<Vec2>,
}

impl Camera {
    const BUFFER_CAPACITY: usize = 20;

    pub fn new(bounds: Rect, viewport_height: f32) -> Camera {
        Camera {
            bounds,
            follow_buffer: vec![],
            viewport_height,
        }
    }

    pub fn update(&mut self, pos: Vec2) -> Camera2D {
        self.follow_buffer.insert(0, pos);
        self.follow_buffer.truncate(Self::BUFFER_CAPACITY);

        let mut sum = (0.0f64, 0.0f64);
        for pos in &self.follow_buffer {
            sum.0 += pos.x as f64;
            sum.1 += pos.y as f64;
        }
        let pos = vec2(
            (sum.0 / self.follow_buffer.len() as f64) as f32,
            (sum.1 / self.follow_buffer.len() as f64) as f32,
        );
        let mut camera_x = pos.x;
        let mut camera_y = pos.y;

        let aspect = screen_width() / screen_height();

        let viewport_width = self.viewport_height * aspect;

        if camera_x < viewport_width / 2. {
            camera_x = viewport_width / 2.;
        }

        if camera_x > self.bounds.w as f32 - viewport_width / 2. {
            camera_x = self.bounds.w as f32 - viewport_width / 2.;
        }
        if camera_y < self.viewport_height / 2. {
            camera_y = self.viewport_height / 2.;
        }

        if camera_y > self.bounds.h as f32 - self.viewport_height / 2. {
            camera_y = self.bounds.h as f32 - self.viewport_height / 2.;
        }

        Camera2D {
            zoom: vec2(
                1.0 / viewport_width as f32 * 2.,
                -1.0 / self.viewport_height as f32 * 2.,
            ),
            target: vec2(camera_x, camera_y),
            ..Default::default()
        }
    }
}
