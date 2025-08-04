pub struct ContentFile {
  pub bytes: &'static [u8],
  pub name: &'static str,
  pub sha256: &'static str,
  pub mime: &'static str,
}

pub static CSS: &ContentFile = &ContentFile {
  bytes: include_bytes!(env!("NUTWG_CLIENT_CSS_PATH")),
  name: env!("NUTWG_CLIENT_CSS_NAME"),
  sha256: env!("NUTWG_CLIENT_CSS_SHA256"),
  mime: "text/css",
};

pub static JS: &ContentFile = &ContentFile {
  bytes: include_bytes!(env!("NUTWG_CLIENT_JS_PATH")),
  name: env!("NUTWG_CLIENT_JS_NAME"),
  sha256: env!("NUTWG_CLIENT_JS_SHA256"),
  mime: "text/javascript",
};

pub static ICON: &ContentFile = &ContentFile {
  bytes: include_bytes!(env!("NUTWG_CLIENT_ICON_PATH")),
  name: env!("NUTWG_CLIENT_ICON_NAME"),
  sha256: env!("NUTWG_CLIENT_ICON_SHA256"),
  mime: "image/svg+xml",
};

pub static SPRITE_SHEET: &ContentFile = &ContentFile {
  bytes: include_bytes!(env!("NUTWG_CLIENT_SPRITE_SHEET_PATH")),
  name: env!("NUTWG_CLIENT_SPRITE_SHEET_NAME"),
  sha256: env!("NUTWG_CLIENT_SPRITE_SHEET_SHA256"),
  mime: "image/svg+xml",
};

impl ContentFile {
  #[inline]
  pub fn short_hash(&self) -> &str {
    &self.sha256[0..8]
  }
}
