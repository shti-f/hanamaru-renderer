extern crate num;
extern crate image;
extern crate time;
extern crate rand;
extern crate rayon;

use image::GenericImage;
use std::fs::File;
use std::path::Path;
use std::fs;
use std::io::{BufWriter, Write};
use num::Float;
use self::rand::{Rng, SeedableRng, StdRng};

mod config;
mod vector;
mod matrix;
mod scene;
mod camera;
mod renderer;
mod material;
mod bsdf;
mod color;
mod texture;
mod math;
mod loader;
mod bvh;

use vector::Vector3;
use matrix::Matrix44;
use scene::{Scene, BvhScene, Sphere, Cuboid, BvhMesh, Skybox};
use bvh::Aabb;
use camera::{Camera, LensShape};
use material::{Material, SurfaceType};
use texture::Texture;
use renderer::{Renderer, DebugRenderer, DebugRenderMode, PathTracingRenderer};
use color::{Color, hsv_to_rgb};
use loader::ObjLoader;

fn tee(f: &mut BufWriter<File>, message: &String) {
    println!("{}", message);
    let _ = f.write_all(message.as_bytes());
    let _ = f.write(b"\n");
}

fn init_scene_simple() -> (Camera, Scene) {
    let camera = Camera::new(
        Vector3::new(0.0, 2.0, 9.0), // eye
        Vector3::new(0.0, 1.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0).normalize(), // y_up
        10.0, // fov

        LensShape::Circle, // lens shape
        0.2 * 0.0,// aperture
        8.8// focus_distance
    );

    let radius = 0.6;

    let scene = Scene {
        elements: vec![
            Box::new(Sphere {
                center: Vector3::new(0.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.99)),
                },
            }),

            // 光源
            Box::new(Sphere {
                center: Vector3::new(3.0, 2.0 + radius, -2.0),
                radius: radius * 0.2,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::black(),
                    emission: Texture::from_color(Color::new(200.0, 10.0, 10.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 光源
            Box::new(Sphere {
                center: Vector3::new(-3.0, 2.0 + radius, -2.0),
                radius: radius * 0.2,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::black(),
                    emission: Texture::from_color(Color::new(10.0, 200.0, 10.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    //albedo:  Texture::white(),
                    //albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    albedo: Texture::from_path("textures/2d/checkered_diagonal_10_0.5_1.0_512.png"),
                    //albedo: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_c_diffuse.tiff"),
                    emission: Texture::black(),
                    //roughness: Texture::white(),
                    roughness: Texture::from_path("textures/2d/checkered_diagonal_10_0.1_0.6_512.png"),
                    //roughness: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png"),
                }
            }),
        ],
        skybox: Skybox::new(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
            &Vector3::zero()
        ),
    };

    (camera, scene)
}

fn init_scene_material_examples() -> (Camera, Scene) {
    let camera = Camera::new(
        Vector3::new(0.0, 2.0, 9.0), // eye
        Vector3::new(0.0, 1.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0).normalize(), // y_up
        10.0, // fov

        LensShape::Circle, // lens shape
        0.2, // * 0.0,// aperture
        8.8// focus_distance
    );

    let radius = 0.4;

    let scene = Scene {
        elements: vec![
            // 球体
            Box::new(Sphere {
                center: Vector3::new(-2.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(-1.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(0.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Specular,
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(1.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(2.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::GGXRefraction { f0: 0.8, refractive_index: 1.5 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 光源
            Box::new(Sphere {
                center: Vector3::new(0.0, 2.0 + radius, -2.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::black(),
                    emission: Texture::from_color(Color::from_one(20.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::Diffuse,
                    //albedo:  Texture::white(),
                    //albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    albedo: Texture::from_path("textures/2d/checkered_diagonal_10_0.5_1.0_512.png"),
                    //albedo: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_c_diffuse.tiff"),
                    emission: Texture::black(),
                    //roughness: Texture::white(),
                    roughness: Texture::from_path("textures/2d/checkered_diagonal_10_0.1_0.6_512.png"),
                    //roughness: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png"),
                }
            }),
        ],
        skybox: Skybox::one(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
        ),
    };

    (camera, scene)
}

fn init_scene_rtcamp5() -> (Camera, Scene) {
    let seed: &[_] = &[870, 2000, 304, 2];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let camera = Camera::new(
        Vector3::new(0.0, 2.5, 9.0), // eye
        Vector3::new(0.0, 1.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0).normalize(), // y_up
        17.0, // fov

        LensShape::Circle, // lens shape
        0.15, // * 0.0,// aperture
        8.5// focus_distance
    );

    let mut scene = Scene {
        elements: vec![
            // うさぎ右
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000.obj",
                Matrix44::scale_linear(1.5) * Matrix44::translate(1.2, 0.0, 0.0) * Matrix44::rotate_y(0.2),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
                    albedo: Texture::from_color(Color::new(0.7, 0.7, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),
            // うさぎ左
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/bunny/bunny_face1000_flip.obj",
                Matrix44::scale(1.5, 1.5, 1.5) * Matrix44::translate(-1.2, 0.0, 0.0) * Matrix44::rotate_y(-0.2),
                Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(1.0, 0.04, 0.04)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            ))),
            // 背後にある地図ガラス
            /*Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-4.0, 0.0, -3.6),
                    max: Vector3::new(4.0, 3.0, -3.5),
                },
                material: Material {
                    surface: SurfaceType::GGXReflection { refractive_index: 1.2 },
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.3)),
                }
            }),*/

            // 固定のダイヤモンド
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/dia/dia.obj",
                Matrix44::translate(3.1, 0.0, 0.8) * Matrix44::scale_linear(1.0) * Matrix44::rotate_y(-0.5) * Matrix44::rotate_x(40.35.to_radians()),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 2.42 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::black(),
                },
            ))),
            // 地球のテクスチャを光源にした球体
            Box::new(Sphere {
                center: Vector3::new(0.0, 0.5, -0.5),
                radius: 0.5,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(5.0, 5.0, 2.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            // 地球のテクスチャをラフネスにした球体
            Box::new(Sphere {
                center: Vector3::new(-3.5, 0.5, 0.0),
                radius: 0.5,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(1.0, 1.0, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_path("textures/2d/earth_inverse_2048.jpg"),
                },
            }),
            // カラフルな球体
            Box::new(Sphere {
                center: Vector3::new(0.5018854352719382, 0.3899602675366644, 1.8484239850862165),
                radius: 0.3899602675366644,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.2, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(-0.5748933256792994, 0.2951263257801348, 2.266298272012876),
                radius: 0.2951263257801348,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.4, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(-0.9865234498515534, 0.3386858117447873, 2.9809338871934585),
                radius: 0.3386858117447873,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.6, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.02)),
                },
            }),
            Box::new(Sphere {
                center: Vector3::new(0.6946459502665004, 0.2764689077971783, 2.7455446851003025),
                radius: 0.2764689077971783,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.05, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.0)),
                },
            }),
            /*Box::new(Sphere {
                center: Vector3::new( 1.4192264328563055, 0.3, 1.6181489825435929),
                radius:  0.3,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 }{ metalness: 1.0 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.7, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),*/
            Box::new(Sphere {
                center: Vector3::new(3.7027464198816952, 0.3917608374245498, -0.40505849281451556),
                radius: 0.3917608374245498,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(hsv_to_rgb(Color::new(0.8, 1.0, 1.0))),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
            }),
            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    //albedo:  Texture::white(),
                    //albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    //albedo: Texture::from_path("textures/2d/checkered_diagonal_10_0.5_1.0_512.png"),
                    albedo: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_c_diffuse.tiff"),
                    emission: Texture::black(),
                    //roughness: Texture::white(),
                    //roughness: Texture::from_path("textures/2d/checkered_diagonal_10_0.1_0.6_512.png"),
                    roughness: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png"),
                }
            }),
        ],
        skybox: Skybox::one(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
        ),
    };

    // 金属の球体
    let mut count = 0;
    while count < 0 {
        let px = rng.gen_range(-2.5, 3.5);
        let py = 0.0;//rng.gen_range(0.0, 3.0);
        let pz = rng.gen_range(-2.0, 3.0);
        let r = rng.gen_range(0.2, 0.4);

        if scene.add_with_check_collisions((Box::new(Sphere {
            center: Vector3::new(px, r + py, pz),
            radius: r,
            material: Material {
                surface: SurfaceType::GGX{ f0: 0.8 },
                albedo: Texture::from_color(hsv_to_rgb(Color::new(0.2 + 0.1 * count as f64, 1.0, 1.0))),
                emission: Texture::black(),
                roughness: Texture::from_color(Color::from_one(rng.gen_range(0.0, 0.2))),
            },
        }))) {
            println!("{}, {}, {} : {}", px, r, pz, 0.2 + 0.1 * count as f64);
            count += 1;
        }
    }

    // 床に落ちているダイヤモンド
    count = 0;
    while count < 12 {
        let px = rng.gen_range(-4.5, 4.5);
        let py = 0.0;
        let pz = rng.gen_range(-2.5, 4.5);
        let s = rng.gen_range(0.7, 1.1);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(40.35.to_radians()),
            Material {
                surface: SurfaceType::Refraction { refractive_index: 2.42 },
                albedo: Texture::white(),
                emission: Texture::black(),
                roughness: Texture::black(),
            },
        )))) {
            count += 1;
        }
    }

    // 空中浮遊しているダイヤモンド
    count = 0;
    while count < 30 {
        let px = rng.gen_range(-4.5, 4.5);
        let py = rng.gen_range(0.0, 4.0);
        let pz = rng.gen_range(-4.5, 3.5);
        let s = rng.gen_range(0.6, 1.1);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());
        let rx = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(rx),
            Material {
                surface: SurfaceType::Refraction { refractive_index: 2.42 },
                albedo: Texture::white(),
                emission: Texture::black(),
                roughness: Texture::black(),
            },
        )))) {
            count += 1;
        }
    }

    (camera, scene)
}

fn init_scene_tbf3() -> (Camera, Scene) {
    let seed: &[_] = &[870, 2000, 304, 1];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let camera = Camera::new(
        Vector3::new(0.0, 2.5, 9.0), // eye
        //Vector3::new(0.0, 15.5, 1.0), // eye
        Vector3::new(0.0, 1.5, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0).normalize(), // y_up
        19.0, // fov

        LensShape::Circle, // lens shape
        0.18, // * 0.0,// aperture
        7.0// focus_distance
    );

    let mut scene = Scene {
        elements: vec![
            // KLab logo
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/klab_logo/klab_logo_triangle.obj",
                Matrix44::scale_linear(0.4) * Matrix44::translate(0.0, 3.1782, 2.0) * Matrix44::rotate_y(-0.5),
                /*Material {
                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
                    albedo: Texture::from_color(Color::new(0.7, 0.7, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },*/
                Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(0.4, 0.4, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            ))),
            // 背後にある地図ガラス
            /*Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-4.0, 0.0, -5.1),
                    max: Vector3::new(4.0, 4.0, -5.0),
                },
                material: Material {
                    surface: SurfaceType::GGXRefraction { refractive_index: 1.2 },
                    albedo: Texture::white(),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.3)),
                }
            }),*/
            // 固定のダイヤモンド（右）
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/dia/dia.obj",
                Matrix44::translate(1.3, 0.0, 2.2) * Matrix44::scale_linear(1.0) * Matrix44::rotate_y(-0.4) * Matrix44::rotate_x(40.35.to_radians()),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 2.42 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::black(),
                },
            ))),

            // 固定のダイヤモンド（中央）
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/dia/dia.obj",
                Matrix44::translate(-0.1, 0.0, 2.4) * Matrix44::scale_linear(1.0) * Matrix44::rotate_y(-1.4) * Matrix44::rotate_x(40.35.to_radians()),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 2.42 },
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::black(),
                },
            ))),

            // 光源の球体（手前）
            Box::new(Sphere {
                center: Vector3::new(-1.0, 0.4, 4.0),
                radius: 0.4,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::one()),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),

            // 光源の球体（奥）
            Box::new(Sphere {
                center: Vector3::new(-3.0, 0.4, -3.5),
                radius: 0.4,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(0.5, 1.0, 1.0)),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(1.0, 3.0, 3.5)),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),

            // 光源の球体（奥）
            Box::new(Sphere {
                center: Vector3::new(4.0, 0.2, -4.5),
                radius: 0.2,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(0.3, 0.7, 1.0)),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(3.0, 3.0, 1.1)),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),

            Box::new(Sphere {
                center: Vector3::new(3.0, 0.2, -4.2),
                radius: 0.2,
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(1.0, 0.7, 0.9)),
                    emission: Texture::new("textures/2d/earth_inverse_2048.jpg", Color::new(2.0, 3.0, 1.0)),
                    roughness: Texture::from_color(Color::from_one(0.01)),
                },
            }),

            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    //albedo:  Texture::white(),
                    //albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    //albedo: Texture::from_path("textures/2d/checkered_diagonal_10_0.5_1.0_512.png"),
                    albedo: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_c_diffuse.tiff"),
                    emission: Texture::black(),
                    //roughness: Texture::white(),
                    //roughness: Texture::from_path("textures/2d/checkered_diagonal_10_0.1_0.6_512.png"),
                    roughness: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png"),
                }
            }),
        ],
        skybox: Skybox::new(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
            &Vector3::new(2.0, 2.0, 3.0),
        ),
    };

    // 金属の球体
    let mut count = 0;
    while count < 8  {
        let px = rng.gen_range(-3.0, 3.0);
        let py = 0.0;//rng.gen_range(0.0, 3.0);
        let pz = rng.gen_range(-5.0, 5.0);
        let r = rng.gen_range(0.2, 0.4);

        if scene.add_with_check_collisions((Box::new(Sphere {
            center: Vector3::new(px, r + py, pz),
            radius: r,
            material: Material {
                surface: SurfaceType::GGX{ f0: 0.8 },
                albedo: Texture::from_color(hsv_to_rgb(Color::new(0.2 + 0.1 * count as f64, 1.0, 1.0))),
                emission: Texture::black(),
                roughness: Texture::from_color(Color::from_one(rng.gen_range(0.0, 0.2))),
            },
        }))) {
            println!("{}, {}, {} : {}", px, r, pz, 0.2 + 0.1 * count as f64);
            count += 1;
        }
    }

    // 床に落ちているダイヤモンド
    count = 0;
    while count < 20 {
        let px = rng.gen_range(-4.0, 4.0);
        let py = 0.0;
        let pz = rng.gen_range(-5.0, 5.0);
        let s = rng.gen_range(0.7, 1.1);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(40.35.to_radians()),
            Material {
                surface: SurfaceType::Refraction { refractive_index: 2.42 },
                albedo: Texture::white(),
                emission: Texture::black(),
                roughness: Texture::black(),
            },
        )))) {
            count += 1;
        }
    }

    // 空中浮遊しているダイヤモンド
    count = 0;
    while count < 0 {
        let px = rng.gen_range(-4.5, 4.5);
        let py = rng.gen_range(0.0, 7.0);
        let pz = rng.gen_range(-4.5, 3.5);
        let s = rng.gen_range(0.6, 1.1);
        let ry = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());
        let rx = rng.gen_range(-180.0.to_radians(), 180.0.to_radians());

        if scene.add_with_check_collisions(Box::new(BvhMesh::from_mesh(ObjLoader::load(
            "models/dia/dia.obj",
            Matrix44::translate(px, py, pz) * Matrix44::scale_linear(s) * Matrix44::rotate_y(ry) * Matrix44::rotate_x(rx),
            Material {
                surface: SurfaceType::Refraction { refractive_index: 2.42 },
                albedo: Texture::white(),
                emission: Texture::black(),
                roughness: Texture::black(),
            },
        )))) {
            count += 1;
        }
    }

    (camera, scene)
}

fn init_scene_rtcamp6() -> (Camera, Scene) {
    let camera = Camera::new(
        Vector3::new(0.0, 2.0, 9.0), // eye
        Vector3::new(0.0, 1.0, 0.0), // target
        Vector3::new(0.0, 1.0, 0.0).normalize(), // y_up
        10.0, // fov

        LensShape::Circle, // lens shape
        0.2 * 0.0,// aperture
        8.8// focus_distance
    );

    let radius = 0.6;

    let scene = Scene {
        elements: vec![
            Box::new(Sphere {
                center: Vector3::new(0.0, radius, 0.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::white(),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),

            // 光源
            /*Box::new(Sphere {
                center: Vector3::new(3.0, 2.0 + radius, -2.0),
                radius: radius,
                material: Material {
                    surface: SurfaceType::Diffuse,
                    albedo: Texture::black(),
                    emission: Texture::from_color(Color::from_one(20.0)),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },
            }),*/

            // Mesh
            Box::new(BvhMesh::from_mesh(ObjLoader::load(
                "models/houdini_boss.obj",
                Matrix44::scale_linear(0.4) * Matrix44::translate(0.0, 3.1782, 2.0) * Matrix44::rotate_y(-0.5),
                Material {
                    surface: SurfaceType::Refraction { refractive_index: 1.5 },
                    albedo: Texture::from_color(Color::new(0.7, 0.7, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.1)),
                },
                /*Material {
                    surface: SurfaceType::GGX{ f0: 0.8 },
                    albedo: Texture::from_color(Color::new(0.4, 0.4, 1.0)),
                    emission: Texture::black(),
                    roughness: Texture::from_color(Color::from_one(0.05)),
                },*/
            ))),

            // 床
            Box::new(Cuboid {
                aabb: Aabb {
                    min: Vector3::new(-5.0, -1.0, -5.0),
                    max: Vector3::new(5.0, 0.0, 5.0),
                },
                material: Material {
                    surface: SurfaceType::Diffuse,
                    //albedo:  Texture::white(),
                    //albedo: Texture::from_path("textures/2d/stone03.jpg"),
                    albedo: Texture::from_path("textures/2d/checkered_diagonal_10_0.5_1.0_512.png"),
                    //albedo: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_c_diffuse.tiff"),
                    emission: Texture::black(),
                    //roughness: Texture::white(),
                    roughness: Texture::from_path("textures/2d/checkered_diagonal_10_0.1_0.6_512.png"),
                    //roughness: Texture::from_path("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png"),
                }
            }),
        ],
        skybox: Skybox::new(
            "textures/cube/LancellottiChapel/posx.jpg",
            "textures/cube/LancellottiChapel/negx.jpg",
            "textures/cube/LancellottiChapel/posy.jpg",
            "textures/cube/LancellottiChapel/negy.jpg",
            "textures/cube/LancellottiChapel/posz.jpg",
            "textures/cube/LancellottiChapel/negz.jpg",
            &Vector3::new(2.0, 2.0, 3.0),
        ),
    };

    (camera, scene)
}

fn render<R: Renderer>(renderer: &mut R, width: u32, height: u32, camera: &Camera, scene: Scene) -> u32 {
    let mut imgbuf = image::ImageBuffer::new(width, height);
    let sampled = renderer.render(&BvhScene::from_scene(scene), camera, &mut imgbuf);
    let _ = image::ImageRgb8(imgbuf).save("result.png");
    sampled
}

fn main() {
    //inspect_image();
    //return;

    let mut f = BufWriter::new(fs::File::create("result.txt").unwrap());

    let total_begin = time::now();
    {

        //let (width, height, sampling) = (1280, 720, 10);// 16:9 HD 921,600 pixel
        //let (width, height, sampling) = (1920, 1080, 1000);// 16:9 FHD 2,073,600 pixel
        let (width, height, sampling) = (1920 / 2, 1080 / 2, 20);
        //let (width, height, sampling) = (800, 600, 10);// 4:3 SVGA 480,000 pixel
        //let (width, height, sampling) = (1280, 960, 1000);// 4:3 960p 1,228,800 pixel
        //let (width, height, sampling) = (1440, 1080, 1000);// 4:3 1080p 1,555,200 pixel
        //let (width, height, sampling) = (2592, 3625, 1000);// B5 + とんぼ(2508 + 42 *2, 3541 + 42 *2)
        //let (width, height, sampling) = (2592/4, 3625/4, 100);// B5 + とんぼ(2508 + 42 *2, 3541 + 42 *2)
        //let (width, height, sampling) = (1024, 1024, 1000);

        let mut renderer = DebugRenderer { mode: DebugRenderMode::Shading };
        let mut renderer = PathTracingRenderer::new(sampling);

        tee(&mut f, &format!("num threads: {}.", rayon::current_num_threads()));
        tee(&mut f, &format!("resolution: {}x{}.", width, height));
        tee(&mut f, &format!("max sampling: {}x{} spp.", sampling, config::SUPERSAMPLING * config::SUPERSAMPLING));
        tee(&mut f, &format!("time limit: {:.2} sec.", config::TIME_LIMIT_SEC));

        let init_scene_begin = time::now();

        let (camera, scene) = init_scene_rtcamp5();
        //let (camera, scene) = init_scene_rtcamp6();
        //let (camera, scene) = init_scene_material_examples();
        //let (camera, scene) = init_scene_tbf3();
        //let (camera, scene) = init_scene_simple();

        let init_scene_end = time::now();
        let init_scene_sec = (init_scene_end - init_scene_begin).num_milliseconds() as f64 * 0.001;
        tee(&mut f, &format!("init scene: {:.2} sec.", init_scene_sec));

        let sampled = render(&mut renderer, width, height, &camera, scene);
        tee(&mut f, &format!("sampled: {}x{} spp.", sampled, config::SUPERSAMPLING * config::SUPERSAMPLING));
    }
    let total_end = time::now();

    let total_sec = (total_end - total_begin).num_milliseconds() as f64 * 0.001;
    let used_percent = total_sec / config::TIME_LIMIT_SEC as f64 * 100.0;
    let progress_per_used = 100.0 / used_percent;
    tee(&mut f, &format!("total {} sec. used {:.2} % (x {:.2})", total_sec, used_percent, progress_per_used));
}

fn inspect_image() {
    let img = image::open(&Path::new("textures/2d/MarbleFloorTiles2/TexturesCom_MarbleFloorTiles2_1024_roughness.png")).unwrap();
    let mut min = 255.0;
    let mut max = 0.0;
    let mut avg = 0.0;
    for (_, _, pixel) in img.pixels() {
        let p = pixel.data[0] as f64;
        min = min.min(p);
        max = max.max(p);
        avg += p;
    }
    avg /= (img.width() * img.height()) as f64;

    println!("min: {} max: {} avg: {}", min, max, avg);
}
