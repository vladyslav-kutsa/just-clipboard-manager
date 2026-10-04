use crate::application::ports::ImageStore;
use image::{DynamicImage, RgbaImage};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct FsImageStore {
    dir: PathBuf,
}

impl FsImageStore {
    const EXT: &str = "webp";
    const THUMB_DIR: &str = "thumbs";

    pub fn new(path: PathBuf) -> FsImageStore {
        FsImageStore { dir: path }
    }

    fn hash(rgba: &[u8], width: u32, height: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rgba);
        hasher.update(width.to_le_bytes());
        hasher.update(height.to_le_bytes());
        let digest = hasher.finalize();
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }

    fn hashes_in(dir: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        let mut hash_set = HashSet::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                hash_set.insert(stem.to_string());
            }
        }

        Ok(hash_set)
    }
}

impl ImageStore for FsImageStore {
    fn save(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let hash = Self::hash(rgba, width, height);
        let original = self.original_path(&hash);
        let thumb = self.thumb_path(&hash);

        if original.exists() {
            return Ok(hash);
        }

        if let Some(parent) = thumb.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let img = RgbaImage::from_raw(width, height, rgba.to_vec())
            .ok_or("rgba не відповідає розмірам")?;
        img.save(&original)?;
        let dyn_img = DynamicImage::from(img);

        let thumb_img = if dyn_img.width() <= 256 && dyn_img.height() <= 256 {
            dyn_img
        } else {
            dyn_img.thumbnail(256, 256)
        };

        thumb_img.save(&thumb)?;

        Ok(hash)
    }

    fn load_rgba(&self, hash: &str) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
        let path = self.original_path(hash);
        let img = image::open(&path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let rgba_raw = rgba.into_raw();
        Ok((rgba_raw, width, height))
    }

    fn thumb_path(&self, hash: &str) -> PathBuf {
        self.dir
            .join(Self::THUMB_DIR)
            .join(format!("{hash}.{}", Self::EXT))
    }

    fn original_path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.{}", Self::EXT))
    }

    fn delete(&self, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let original_path = self.original_path(&hash);
        let thumb_path = self.thumb_path(&hash);

        if original_path.exists() {
            std::fs::remove_file(&original_path)?;
        }

        if thumb_path.exists() {
            std::fs::remove_file(&thumb_path)?;
        }

        Ok(())
    }

    fn cleanup(&self, keep: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let keep: HashSet<&String> = keep.iter().collect();
        let mut hashes = Self::hashes_in(&self.dir)?;
        hashes.extend(Self::hashes_in(&self.dir.join(Self::THUMB_DIR))?);
        for hash in hashes {
            if !keep.contains(&hash) {
                self.delete(&hash)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("cm-test-{}", Uuid::now_v7()))
    }

    #[test]
    fn hash_matches_known_value() {
        let h = FsImageStore::hash(&[1, 2, 3, 4], 1, 1);
        assert_eq!(
            h,
            "e2056a4167d8d2aabc757f2ad11c6d912de7d4a628684c4717703a30e45ca393"
        );
    }

    #[test]
    fn hash_differs_by_dimensions() {
        let buf = [1, 2, 3, 4];
        assert_ne!(
            FsImageStore::hash(&buf, 1, 1),
            FsImageStore::hash(&buf, 4, 1),
        );
    }

    #[test]
    fn hash_is_64_hex_chars() {
        assert_eq!(FsImageStore::hash(&[], 0, 0).len(), 64);
    }

    #[test]
    fn save_is_idempotent() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let h1 = store.save(&rgba, 2, 2).unwrap();
        let path = store.original_path(&h1);
        std::fs::write(&path, b"test").unwrap();
        store.save(&rgba, 2, 2).unwrap();
        let file = std::fs::read(&path).unwrap();

        assert_eq!(file, b"test");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_writes_file() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let h = store.save(&rgba, 2, 2).unwrap();
        let some_path = store.original_path(&h);
        let file_exists = some_path.exists();

        assert!(file_exists);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_load_round_trip() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = (0..16).collect::<Vec<u8>>();
        let hash = store.save(&rgba, 2, 2).unwrap();
        let (rgba_back, width, height) = store.load_rgba(&hash).unwrap();

        assert_eq!(rgba_back, rgba);
        assert_eq!(width, 2);
        assert_eq!(height, 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_writes_thumbnail() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let hash = store.save(&rgba, 2, 2).unwrap();
        let some_path = store.thumb_path(&hash);
        let file_exists = some_path.exists();

        let thumb_path = store.thumb_path(&hash);
        let img = image::open(&thumb_path).unwrap();
        let width = img.width();
        let height = img.height();

        assert!(file_exists);
        assert!(width == 2 && height == 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn thumbnail_shrinks_oversized() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 2400];
        let hash = store.save(&rgba, 300, 2).unwrap();
        let some_path = store.thumb_path(&hash);
        let file_exists = some_path.exists();

        let thumb_path = store.thumb_path(&hash);
        let img = image::open(&thumb_path).unwrap();
        let width = img.width();
        let height = img.height();

        assert!(file_exists);
        assert!(width <= 256 && height <= 256);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_removes_both_files() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let hash = store.save(&rgba, 2, 2).unwrap();
        let original_path = store.original_path(&hash);
        let thumb_path = store.thumb_path(&hash);
        store.delete(&hash).unwrap();

        assert!(original_path.exists() == false);
        assert!(thumb_path.exists() == false);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_is_idempotent() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let hash = store.save(&rgba, 2, 2).unwrap();
        store.delete(&hash).unwrap();
        let result = store.delete(&hash);

        assert!(result.is_ok());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cleanup_removes_orphans() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba_a = vec![255u8; 16];
        let rgba_b = vec![0u8; 16];
        let hash_a = store.save(&rgba_a, 2, 2).unwrap();
        let hash_b = store.save(&rgba_b, 2, 2).unwrap();
        let keep = vec![hash_a.clone()];
        store.cleanup(&keep).unwrap();

        assert_eq!(
            (
                store.original_path(&hash_a).exists(),
                store.thumb_path(&hash_a).exists()
            ),
            (true, true)
        );
        assert_eq!(
            (
                store.original_path(&hash_b).exists(),
                store.thumb_path(&hash_b).exists()
            ),
            (false, false)
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cleanup_removes_orphan_thumb() {
        let dir = temp_dir();
        let store = FsImageStore::new(dir.clone());
        let rgba = vec![255u8; 16];
        let hash = store.save(&rgba, 2, 2).unwrap();
        let original_path = store.original_path(&hash);
        let thumb_path = store.thumb_path(&hash);
        std::fs::remove_file(original_path).unwrap();
        store.cleanup(&[]).unwrap();

        assert!(!thumb_path.exists(), "мініатюра-сирота мала зникнути");

        std::fs::remove_dir_all(&dir).ok();
    }
}
