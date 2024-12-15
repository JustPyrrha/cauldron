use crate::renderer::backend::dx12::RendererOutput;
use windows::core::Result;

pub mod backend;
pub mod input;
pub mod msg_filter;
pub mod pipeline;

pub trait RenderEngine {
    type RenderTarget;

    fn render(
        &mut self,
        ctx: &egui::Context,
        output: RendererOutput,
        render_target: Self::RenderTarget,
    ) -> Result<()>;
}
