use ocipkg::{ImageName, distribution::get_image, local::{data_dir, image_dir}};
use std::path::PathBuf;

pub fn pull_image(name: &str, update: bool) -> anyhow::Result<PathBuf> {
  let image = ImageName::parse(name)?;
  let dir = image_dir(&image)?;

  if !update && dir.exists() {
    return Ok(dir);
  }

  get_image(&image)?;

  Ok(dir)
}

#[test]
fn test_pull_image() {
  let name = "registry.hub.docker.com/library/alpine:latest";
  pull_image(name, false).unwrap();
  println!("{}", data_dir().unwrap().display());
}