pub(super) fn content_padding(available_width: f32) -> i8 {
    let max_width = available_width.min(1600.0);
    ((available_width - max_width) / 2.0).max(10.0) as i8
}
