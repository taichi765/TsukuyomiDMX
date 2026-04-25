//! Sets up UI. For instance, register UI callbacks, instantiate `Model`s etc.

pub mod effect_editor;
pub mod effect_tree_view;
pub mod fixture_list_view;
pub mod preview_2d;
pub mod universe_view;

// TODO: macroにするかも
/// spanを作って、経過時間が長かった場合warn!()で出力する
pub fn wrap_callback(name: &'static str, mut f: impl FnMut()) {
    let _span = tracing::debug_span!("callback_wrapper_span", name).entered();
    let now = std::time::Instant::now();

    f();

    if now.elapsed() >= std::time::Duration::from_millis(16) {
        tracing::warn!(name, elapsed = ?now.elapsed(), "callback took too long");
    }
}
