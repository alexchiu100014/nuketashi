//! ゲームのアセット（画像などの素材）を扱うための
//! モジュール．

use piston_window::*;
use std::collections::HashMap;

use std::cell::RefCell;
use std::env::{current_dir, current_exe};
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::*;

lazy_static! {
    static ref ASSETS_MAP: Mutex<HashMap<&'static str, G2dTexture>> = Mutex::new(HashMap::new());
    static ref ASSETS_DIR: PathBuf = find_assets().expect("Assets folder is missing.");
}

thread_local!(static TEX_CTX: RefCell<Option<G2dTextureContext>> = RefCell::new(None));

/// TextureContextをWindowから受け取る．
pub fn set_texture_context(c: G2dTextureContext) {
    TEX_CTX
        .try_with(|t| {
            *(t.borrow_mut()) = Some(c);
        })
        .expect("Failed to obtain the texture context");
}

/// "Assets"フォルダを検索する．
pub fn find_assets() -> Option<PathBuf> {
    // macOSの場合はResources/以下を捜索
    if cfg!(target_os = "macos") {
        let mut p = current_exe().unwrap();
        p.pop();
        p.pop();
        p.push("Resources/Assets");

        if p.is_dir() {
            return Some(p);
        }
    }

    // 実行ファイルと作業ディレクトリの周りを探す
    let mut p = current_exe().unwrap();
    p.pop();
    p.push("Assets");

    if p.is_dir() {
        return Some(p);
    }

    let mut p = current_dir().unwrap();
    p.push("swarm-game");
    p.push("Assets");

    if p.is_dir() {
        return Some(p);
    }

    let mut p = current_dir().unwrap();
    p.push("Assets");

    if p.is_dir() {
        return Some(p);
    }

    None
}

/// 画像を読み込み，テクスチャとして取得する．
pub fn load_image(path: &'static str) -> Option<G2dTexture> {
    TEX_CTX
        .try_with(|tc| {
            if let Some(t) = ASSETS_MAP.try_lock().ok()?.get(&path) {
                let t = t.clone();
                return Some(t);
            }

            let p = get_path(path)?;

            let t = Texture::from_path(
                tc.borrow_mut().as_mut()?,
                &p,
                Flip::None,
                &TextureSettings::new(),
            );

            t.map(|v| {
                let mut assets_map = ASSETS_MAP.try_lock().ok()?;
                assets_map.insert(path, v.clone());
                Some(v)
            })
            .unwrap_or(None)
        })
        .unwrap_or(None)
}

/// RGBAから画像を読み込む
pub fn load_image_from_rgba(
    rgba: &[u8],
    width: usize,
    height: usize,
    id: &'static str,
) -> G2dTexture {
    TEX_CTX
        .try_with(|tc| {
            Texture::from_image(
                tc.borrow_mut().as_mut()?,
                &::image::RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())?,
                &TextureSettings::new(),
            )
            .ok()
        })
        .unwrap()
        .unwrap()
}

/// アセットデータのパスの取得
pub fn get_path(path: &str) -> Option<PathBuf> {
    let mut p = ASSETS_DIR.clone();
    p.push(path);

    if p.is_file() {
        Some(p)
    } else {
        None
    }
}
