use napi::{Task, bindgen_prelude::Buffer};
use takumi::resources::image::{PersistentImageStore, load_image_source_from_bytes};

pub struct PutPersistentImageTask<'s> {
  pub src: Option<String>,
  pub store: &'s PersistentImageStore,
  pub buffer: Buffer,
}

impl Task for PutPersistentImageTask<'_> {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let image = load_image_source_from_bytes(&self.buffer).unwrap();
    self.store.insert(self.src.take().unwrap(), image);

    Ok(())
  }

  fn resolve(&mut self, _env: napi::Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(())
  }
}
