pub fn format_error_chain(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current_error = e.source();
    while let Some(cause) = current_error {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current_error = cause.source();
    }
    Ok(())
}
