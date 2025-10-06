use std::io::Cursor;

use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::from_str;
use takumi::{
  layout::{Viewport, node::NodeKind},
  rendering::{ImageOutputFormat, RenderOptionsBuilder, render, write_image},
};
use tokio::task::spawn_blocking;

use crate::{AxumResult, AxumState};

#[derive(Deserialize)]
pub struct GenerateImageQuery {
  pub format: Option<ImageOutputFormat>,
  pub quality: Option<u8>,
  pub payload: String,
  pub draw_debug_border: Option<bool>,
  pub width: u32,
  pub height: u32,
}

pub async fn generate_image_handler(
  Query(query): Query<GenerateImageQuery>,
  State(state): AxumState,
) -> AxumResult<Response> {
  let root_node: NodeKind = from_str(&query.payload).map_err(|err| {
    (
      StatusCode::BAD_REQUEST,
      format!("Failed to parse node: {err}"),
    )
  })?;

  let format = query.format.unwrap_or(ImageOutputFormat::WebP);

  let buffer = spawn_blocking(move || -> AxumResult<Vec<u8>> {
    let viewport = Viewport::new(query.width, query.height);
    let options = RenderOptionsBuilder::default()
      .viewport(viewport)
      .node(root_node)
      .global(&state.context)
      .draw_debug_border(query.draw_debug_border.unwrap_or(false))
      .build()
      .unwrap();

    let image = render(options).map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to render image: {err:?}"),
      )
    })?;

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    write_image(&image, &mut cursor, format, query.quality).map_err(|err| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to write image: {err}"),
      )
    })?;

    Ok(buffer)
  })
  .await
  .map_err(|err| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Image generation task panicked: {err}"),
    )
  })??;

  Ok(([("content-type", format.content_type())], buffer).into_response())
}
