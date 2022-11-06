use crate::renderer::Renderer;

pub mod renderer;

extern crate glm;

use glm::{ivec2, IVec2, Vector2, Vector3};
use glm::{cross, dot, normalize};
use renderer::{color, Color};

pub type Vec2 = Vector2<f64>;
pub type Vec3 = Vector3<f64>;

fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2{x, y}
}

fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3{x, y, z}
}


pub const ASPECT_RATIO : f64 = 36.0 / 24.0;

pub const WINDOW_HEIGHT : usize = 512;
pub const WINDOW_WIDTH : usize = (WINDOW_HEIGHT as f64 * ASPECT_RATIO) as usize;

// In this project, 1 unit of space = 1 meter

// return an engine space representation of n centimeters;
fn centimeters(n : f64) -> f64 {
    return n / 100.0;
}

// return an engine space representation of n millimeters
fn millimeters(n : f64) -> f64 {
    return n / 1000.0;
}


struct Sphere {
    center : Vec3,
    radius : f64,
}

#[derive(Debug)]
struct Ray {
    origin : Vec3,
    dir : Vec3,
}

#[derive(Copy, Clone)]
struct HitRecord {
    dist: f64,
    norm: Vec3,
    point: Vec3,
    //mat: Material
}


impl Ray {
    fn new(origin : Vec3, dir : Vec3) -> Self {
        Self {origin, dir}
    }

    fn cast(&self) -> Vec3 {
        let sphere = Sphere {center : vec3(0.0, 0.0, 0.0), radius: 2.0};
        let sphere2 = Sphere {center : vec3(0.0, 3.0, 0.0), radius: 1.0};
    
        let sphere_hit = ray_sphere_intersection(self, &sphere, 0.0, 10_000.0);
        let sphere2_hit = ray_sphere_intersection(self, &sphere2, 0.0, 10_000.0);

        if let Some(hit) = sphere_hit {
            if let Some(hit2) = sphere2_hit {
                return match hit.dist < hit2.dist {
                    true => vec3(1.0, 0.0, 0.0),
                    false => vec3(0.0, 1.0, 0.0)
                };
            }
            return vec3(1.0, 0.0, 0.0);
        }

        if let Some(_hit2) = sphere2_hit {
            return vec3(0.0, 1.0, 0.0);
        }

        vec3(1.0, 0.0, 1.0)
    }
}


struct RayIterator {
    i : usize,

    samples_per_pixel : usize,
    total_samples : usize,

    sensor_bottom_left: Vec3,
    sensor_x_axis : Vec3,
    sensor_y_axis : Vec3,

    sensor : Vec2,

    focal_point : Vec3,
}


impl RayIterator {
    fn new(cam : &Camera) -> Self {
        // TODO: a lot of this logic should belong to the Camera itself

        // Find the focal point by advancing focal_length along the look vector
        let focal_point = cam.position + (cam.look * cam.focal_length);
        
        let sensor_x_axis = cross(cam.look, vec3(0.0, 1.0, 0.0));
        let sensor_y_axis = cross(cam.look, sensor_x_axis);

        // TODO: Consider constructing a matrix to convert from x, y to "sensor space"?
        let sensor_bottom_left = cam.position - 
            (sensor_x_axis * cam.sensor.x * 0.5) -
            (sensor_y_axis * cam.sensor.y * 0.5);

        // This is kind of arbitrary, but it will work for now
        let samples_per_pixel : usize = (100.0 * cam.exposure) as usize;
        
        // TODO: don't use the window width and height directly like this?
        let total_samples = samples_per_pixel * WINDOW_WIDTH * WINDOW_HEIGHT;

        let sensor = cam.sensor;

        Self {
            i: 0, 
            samples_per_pixel, 
            total_samples,
            sensor_bottom_left,
            sensor_x_axis, 
            sensor_y_axis,
            sensor,
            focal_point,
        }
    }
}


impl Iterator for RayIterator {
    type Item = (IVec2, Ray);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        self.i += 1;

        if i >= self.total_samples {
            return None
        }

        let pixel_index = i / self.samples_per_pixel;
        let pixel = ivec2((pixel_index % WINDOW_WIDTH) as i32, (pixel_index / WINDOW_WIDTH) as i32); 

        // How far to move along the sensor from the bottom left, (0, 0) = bottom left, (1, 1) = top right
        let advance = vec2(pixel.x as f64 / WINDOW_WIDTH as f64, pixel.y as f64 / WINDOW_HEIGHT as f64);
        
        // Finally, the actual point on the sensor we are looking for!
        let ray_origin = self.sensor_bottom_left + self.sensor_x_axis * advance.x * self.sensor.x + self.sensor_y_axis * advance.y * self.sensor.y;
        
        // And of course, our ray direction is just toward the focal point!
        // Maybe adding randomness here to simulate an imperfect lens would be fun?
        let ray_dir = normalize(self.focal_point - ray_origin);

        Some((pixel, Ray::new(ray_origin, ray_dir)))
    }
}


// The goal here is to create a more realistic camera
// so I'm going to define a lot of variables which I can
// hopefully use to generate a better set of rays to cast per frame
#[derive(Copy, Clone)]
struct Camera {
    position : Vec3,    // Position of the camera
    look : Vec3,        // Normalized "look direction" vector
    sensor : Vec2,      // Sensor size
    exposure : f64,     // Amount of "time" to expose for, higher values generate more rays
    focal_length : f64, // The distance from sensor to where the light crosses over
    iso : f64,          // How much color to add to the image for each ray
}


impl Camera {
    fn new(position : Vec3, look : Vec3, sensor : Vec2, exposure : f64, focal_length : f64, iso : f64) -> Self {
        Self {
            position,
            look,
            sensor,
            exposure,
            focal_length,
            iso
        }
    }

    // Create an iterator that generates rays from this camera 
    fn rays(&self) -> RayIterator {
        return RayIterator::new(self);
    }
}


fn ray_sphere_intersection(ray: &Ray, sphere: &Sphere, t_min : f64, t_max : f64) -> Option<HitRecord> {
    let oc = ray.origin - sphere.center;
    let a = glm::dot(ray.dir, ray.dir);
    let half_b = glm::dot(oc, ray.dir);
    let c = glm::dot(oc, oc) - sphere.radius * sphere.radius;

    let disc = half_b*half_b - a*c;

    if disc < 0.0 {
        return None;
    }

    let sqrtd = disc.sqrt();

    let mut root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return None;
        }
    }


    let dist = root;
    let point = ray.origin + ray.dir * dist;
    let norm = (point - sphere.center) / sphere.radius;
    return Some(HitRecord{dist, norm, point});
    
}



fn to_color(x: Vec3) -> Color {
    color((x.x * 255.0) as u8, (x.y * 255.0) as u8, (x.z * 255.0) as u8)
}


fn main() {
    let mut renderer = Renderer::create(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
    renderer.initialize();

    let cam_pos = vec3(10.0, 10.0, 10.0);
    let cam_look = normalize(vec3(0.0, 0.0, 0.0) - cam_pos);

    let sensor_size = vec2(millimeters(36.0), millimeters(24.0));

    let mut cam = Camera::new(cam_pos, cam_look, sensor_size, 0.01, millimeters(50.0), 1.0);


    let mut t : f64 = 0.0;

    while !renderer.should_close() {
        t += 0.1;
        cam.focal_length += millimeters(t.sin() * 5.0);
        for (pixel, ray) in cam.rays() {
            //println!("{:?}: {:?} -> {:?}", pixel, ray, ray.cast());
            renderer.set_pixel(pixel.x as u32, pixel.y as u32,  &to_color(ray.cast()));
        }
    

        renderer.update();
        
    }
}
