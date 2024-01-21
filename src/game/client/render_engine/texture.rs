use glam::UVec2;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

use crate::file_utilities::{file_name_from_path, read_file_to_byte_vec};

pub struct Texture {
  texture_name: String,

  diffuse_bytes: Vec<u8>,
  diffuse_image: DynamicImage,
  diffuse_rgba: ImageBuffer<Rgba<u8>, Vec<u8>>,
  dimensions: UVec2,

  diffuse_texture: Option<wgpu::Texture>,
}

impl Texture {
  pub fn new(path: &str) -> Self {
    let diffuse_bytes = read_file_to_byte_vec(path);
    let diffuse_image = image::load_from_memory(diffuse_bytes.as_slice()).unwrap();
    let diffuse_rgba: ImageBuffer<Rgba<u8>, Vec<u8>> = diffuse_image.to_rgba8();
    let dimensions = diffuse_image.dimensions();

    Texture {
      texture_name: file_name_from_path(path),

      diffuse_bytes,
      diffuse_image,
      diffuse_rgba,
      dimensions: UVec2::new(dimensions.0, dimensions.1),

      diffuse_texture: None,
    }
  }

  pub fn generate_wgpu_buffer(&mut self, device: &wgpu::Device) {
    let texture_size = wgpu::Extent3d {
      width: self.dimensions.x,
      height: self.dimensions.y,
      depth_or_array_layers: 1,
    };

    self.diffuse_texture = Some(device.create_texture(&wgpu::TextureDescriptor {
      // All textures are stored as 3D, we represent our 2D texture
      // by setting depth to 1.
      size: texture_size,
      mip_level_count: 1, // We'll talk about this a little later
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      // Most images are stored using sRGB, so we need to reflect that here.
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
      // COPY_DST means that we want to copy data to this texture
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      label: Some(&self.texture_name),
      // This is the same as with the SurfaceConfig. It
      // specifies what texture formats can be used to
      // create TextureViews for this texture. The base
      // texture format (Rgba8UnormSrgb in this case) is
      // always supported. Note that using a different
      // texture format is not supported on the WebGL2
      // backend.
      view_formats: &[],
    }));
  }
}
