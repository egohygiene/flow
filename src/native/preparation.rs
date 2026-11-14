//! Explicit byte preparation is a precondition, never runtime authority.
use super::*;
use crate::ai_preparation::{
    State,
    setup::{self, documents},
};

pub(super) fn current(
    pin: &FilePin,
    output: &Path,
    token: &dyn CancellationSignal,
) -> Result<(), Error> {
    pin.current(16_384, token)?;
    let marker = path(&pin.path)?;
    if marker.file_name().and_then(|s| s.to_str()) != Some("setup.json") {
        return Err(error(
            Code::InvalidInput,
            "preparation must pin the existing SDK setup.json marker",
        ));
    }
    let root = marker
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "preparation parent required"))?;
    let guard = || token.is_cancelled();
    let value = documents::status(root, &guard).map_err(|_| {
        error(
            Code::StalePlan,
            "selected preparation documents or assets changed",
        )
    })?;
    if !matches!(
        value.current.report.state,
        State::Disabled | State::PreparedBytes
    ) {
        return Err(error(
            Code::Unavailable,
            "selected preparation is not disabled or matched bytes; follow preparation-status actions",
        ));
    }
    let request = setup::Request::from_json_slice(
        &documents::read(&root.join("request.json"), 131_072)
            .map_err(|_| error(Code::StalePlan, "preparation request unavailable"))?,
    )
    .map_err(|_| error(Code::StalePlan, "preparation request unavailable"))?;
    for source in
        std::iter::once(root.to_path_buf()).chain(request.assets.into_iter().map(|a| a.path))
    {
        if output.starts_with(&source) || source.starts_with(output) {
            return Err(error(
                Code::InvalidInput,
                "native output overlaps explicitly selected preparation assets or documents",
            ));
        }
    }
    cancelled(token)?;
    pin.current(16_384, token)
}
