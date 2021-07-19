extern crate rand;

mod Color;
mod camera;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod rtweekend;
mod sphere;
mod vec3;

#[allow(clippy::float_cmp)]
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

use crate::camera::Camera;
use crate::hittable::HitRecord;
use crate::material::{Dielectric, Lambertian};
use crate::material::Metal;
use crate::rtweekend::clamp;
use crate::rtweekend::infinity;
pub use crate::vec3::color;
use crate::vec3::Point3;
pub use hittable::Hittable;
pub use hittable_list::HittableList;
use rand::Rng;
pub use ray::Ray;
use sphere::Sphere;
use std::rc::Rc;
pub use vec3::Vec3;

fn ray_color(r: Ray, world: &HittableList, depth: i32) -> color {
    let mut rec = HitRecord::new();

    if depth <= 0 {
        return color::new(0.0, 0.0, 0.0);
    }
    if world.hit(r, 0.001, infinity, &mut rec) {
        //let target = rec.p + rec.normal + Vec3::random_unit_vector();
        //return ray_color(Ray::new(rec.p, target - rec.p), world, depth - 1) * 0.5;
        let mut scattered = Ray::default_new();
        let mut attenuation = color::zero();
        if rec
            .mat_ptr
            .scatter(r, &rec, &mut attenuation, &mut scattered)
        {
            return ray_color(scattered, world, depth - 1) * attenuation;
        }
        color::zero();
    }

    let unit_direction = r.direction().unit();
    let t = (unit_direction.y + 1.0) * 0.5;
    return color::new(1.0, 1.0, 1.0) * (1.0 - t) + color::new(0.5, 0.7, 1.0) * t;
}

fn hit_sphere(center: Point3, radius: f64, r: Ray) -> f64 {
    let oc = r.origin() - center;
    let a = r.direction().squared_length();
    let half_b = Vec3::dot(oc, r.direction());
    let c = oc.squared_length() - radius * radius;
    let delta = half_b.powi(2) - a * c;

    if delta < 0.0 {
        return -1.0;
    }

    return (-half_b - delta.sqrt()) / a;
}

fn main() {
    //image
    let aspect_ratio = 3.0 / 2.0;
    let image_width: f64 = 1200.0;
    let image_height: f64 = image_width / aspect_ratio;
    let samples_per_pixel = 500.0;
    let max_depth = 50;

    let world = random_scene();
    //camera

    let lookfrom = Point3::new(12.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    //rand
    let mut rng = rand::thread_rng();

    //render
    println!("P3\n{} {} \n255\n", image_width, image_height);
    let mut img: RgbImage = ImageBuffer::new(image_width as u32, image_height as u32);
    let bar = ProgressBar::new(image_width as u64);
    let mut j_ = image_height - 1.0;

    while j_ >= 0.0 {
        let mut i_ = 0.0;
        while i_ < image_width {
            let mut s_ = 0.0;
            let mut pixel_color = color::new(0.0, 0.0, 0.0);
            while s_ < samples_per_pixel {
                let u_ = (i_ + rng.gen::<f64>()) / (image_width - 1.0);
                let v_ = (j_ + rng.gen::<f64>()) / (image_height - 1.0);
                let r_ = cam.get_ray(u_, v_);
                pixel_color += ray_color(r_, &world, max_depth);
                s_ = s_ + 1.0;
            }
            //color write
            let pixel = img.get_pixel_mut(i_ as u32, j_ as u32);
            let mut r_ = pixel_color.x;
            let mut g_ = pixel_color.y;
            let mut b_ = pixel_color.z;
            let scale = 1.0 / samples_per_pixel;
            r_ = (scale * r_).sqrt();
            g_ = (scale * g_).sqrt();
            b_ = (scale * b_).sqrt();
            clamp(r_, 0.0, 0.999);
            clamp(g_, 0.0, 0.999);
            clamp(b_, 0.0, 0.999);
            let r_ = r_ * 255.999;
            let g_ = g_ * 255.999;
            let b_ = b_ * 255.999;
            let r_ = r_ as i64;
            let g_ = g_ as i64;
            let b_ = b_ as i64;
            color::wrt_color(&pixel_color, samples_per_pixel);
            *pixel = image::Rgb([r_ as u8, g_ as u8, b_ as u8]);
            i_ = i_ + 1.0;
        }
        bar.inc(1);
        j_ = j_ - 1.0;
    }
    img.save("output/test.png").unwrap();
    bar.finish();
}

pub fn random_scene() -> HittableList {
    let mut world = HittableList::new_default();

    let ground_material: Rc<Lambertian> = Rc::new(Lambertian::new(color::new(0.5, 0.5, 0.5)));
    world.add(Rc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material.clone(),
    )));

    let mut rng = rand::thread_rng();

    let mut a = -11.0;
    while a < 11.0 {
        let mut b = -11.0;
        while b < 11.0 {
            let choose_mat = rng.gen::<f64>();
            let center = Point3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = color::random() * color::random();
                    let sphere_material = Rc::new(Lambertian::new(albedo));
                    world.add(Rc::new(Sphere::new(center, 0.2, sphere_material.clone())));
                } else if choose_mat < 0.95 {
                    let albedo = color::random_range(0.5, 1.0);
                    let fuzz = rng.gen_range(0.0..0.5);
                    let sphere_material = Rc::new(Metal::new(albedo, fuzz));
                    world.add(Rc::new(Sphere::new(center, 0.2, sphere_material.clone())));
                } else {
                    let sphere_material = Rc::new(Dielectric::new(1.5));
                    world.add(Rc::new(Sphere::new(center, 0.2, sphere_material.clone())));
                }
            }
            b = b + 1.0;
        }
        a = a + 1.0;
    }

    let material1 = Rc::new(Dielectric::new(1.5));
    world.add(Rc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1.clone(),
    )));

    let material2 = Rc::new(Lambertian::new(color::new(0.4, 0.2, 0.1)));
    world.add(Rc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2.clone(),
    )));

    let material3 = Rc::new(Metal::new(color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Rc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3.clone(),
    )));
    world
}