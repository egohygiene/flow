//! Coverage intent over the existing matrix; no new evidence or admission port.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub tool: Tool,
    pub interface: Interface,
    pub target: String,
    pub profile: String,
    #[serde(default)]
    pub row_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub cohort_id: String,
    pub required: Vec<Expected>,
}
impl Request {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 1_048_576 {
            return Err(Error::Invalid("coverage document bound"));
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| Error::Invalid("closed coverage request required"))?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.release-coverage-request/v1"
            || !text(&self.cohort_id)
            || self.required.is_empty()
            || self.required.len() > 512
        {
            return Err(Error::Invalid(
                "explicit nonempty bounded coverage required",
            ));
        }
        let mut seen = BTreeSet::new();
        for row in &self.required {
            if !text(&row.target)
                || !text(&row.profile)
                || row.row_id.as_ref().is_some_and(|id| !text(id))
                || !seen.insert(
                    serde_json::to_vec(row)
                        .map_err(|_| Error::Invalid("coverage selector encoding"))?,
                )
            {
                return Err(Error::Invalid("duplicate or invalid coverage selector"));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    Missing,
    Ambiguous,
    Blocked,
    Unverified,
    Green,
}
#[derive(Debug, Serialize)]
pub struct Required {
    pub expected: Expected,
    pub matching_rows: Vec<String>,
    pub state: CoverageState,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: String,
    pub request_sha256: String,
    pub matrix: Matrix,
    pub required: Vec<Required>,
    pub green: usize,
    pub missing: usize,
    pub ambiguous: usize,
    pub blocked: usize,
    pub unverified: usize,
    pub unrequested_rows: Vec<String>,
    pub all_requested_green: bool,
    pub whole_suite_qualified: bool,
}
fn disposition(row: &MatrixRow) -> CoverageState {
    if row.green {
        CoverageState::Green
    } else if row.declaration.implementation != Implementation::Implemented
        || row.declaration.artifact.is_none()
        || row
            .cells
            .iter()
            .any(|c| c.observation.state == State::Failed)
    {
        CoverageState::Blocked
    } else {
        CoverageState::Unverified
    }
}
/// The matrix must come from its normal evidence owner. Recheck its subject,
/// cell completeness and derived flags; declaration/coverage is not authority.
pub fn evaluate(
    matrix: Matrix,
    request: &Request,
    token: &dyn CancellationSignal,
) -> Result<Report, Error> {
    request.validate()?;
    if matrix.schema_version != MATRIX_SCHEMA
        || matrix.cohort_id != request.cohort_id
        || matrix.rows.is_empty()
        || matrix.rows.len() > 4096
    {
        return Err(Error::Invalid("coverage and matrix scope differ"));
    }
    let mut ids = BTreeSet::new();
    for row in &matrix.rows {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let expected = subject(&row.declaration)?;
        if !ids.insert(&row.declaration.id)
            || row.subject_sha256 != expected
            || row.cells.len() != 5
        {
            return Err(Error::Invalid("coverage matrix subjects/cells differ"));
        }
        for (cell, dimension) in row.cells.iter().zip([
            Dimension::Contracts,
            Dimension::Publication,
            Dimension::Installation,
            Dimension::Runtime,
            Dimension::Qualification,
        ]) {
            let observation = &cell.observation;
            if cell.dimension != dimension
                || observation.subject_sha256 != expected
                || !text(&observation.reason)
                || observation
                    .evidence_sha256
                    .as_ref()
                    .is_some_and(|v| !is_digest(v))
                || observation.state == State::Passed && observation.evidence_sha256.is_none()
            {
                return Err(Error::Invalid("coverage cell is unbound or incomplete"));
            }
        }
        let green = row.declaration.implementation == Implementation::Implemented
            && row.declaration.artifact.is_some()
            && !row.declaration.contracts.is_empty()
            && row
                .cells
                .iter()
                .all(|c| c.observation.state == State::Passed);
        if row.green != green {
            return Err(Error::Invalid("coverage cannot promote a matrix row"));
        }
    }
    let mut required = vec![];
    let mut requested = BTreeSet::new();
    let (mut green, mut missing, mut ambiguous, mut blocked, mut unverified) = (0, 0, 0, 0, 0);
    for expected in &request.required {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let matches = matrix
            .rows
            .iter()
            .filter(|row| {
                let d = &row.declaration;
                d.tool == expected.tool
                    && d.interface == expected.interface
                    && d.target == expected.target
                    && d.profile == expected.profile
                    && expected.row_id.as_ref().is_none_or(|id| *id == d.id)
            })
            .collect::<Vec<_>>();
        let state = match matches.as_slice() {
            [] => CoverageState::Missing,
            [row] => disposition(row),
            _ => CoverageState::Ambiguous,
        };
        match state {
            CoverageState::Green => green += 1,
            CoverageState::Missing => missing += 1,
            CoverageState::Ambiguous => ambiguous += 1,
            CoverageState::Blocked => blocked += 1,
            CoverageState::Unverified => unverified += 1,
        }
        let matching_rows = matches
            .iter()
            .map(|row| row.declaration.id.clone())
            .collect::<Vec<_>>();
        requested.extend(matching_rows.iter().cloned());
        required.push(Required {
            expected: expected.clone(),
            matching_rows,
            state,
        });
    }
    let unrequested_rows = matrix
        .rows
        .iter()
        .filter(|r| !requested.contains(&r.declaration.id))
        .map(|r| r.declaration.id.clone())
        .collect();
    let request_sha256 = sha256(
        &serde_json::to_vec(
            &serde_json::to_value(request)
                .map_err(|_| Error::Invalid("coverage identity encoding"))?,
        )
        .map_err(|_| Error::Invalid("coverage identity encoding"))?,
    );
    Ok(Report {
        schema_version: "flow.release-coverage/v1".into(),
        request_sha256,
        matrix,
        all_requested_green: green == required.len(),
        required,
        green,
        missing,
        ambiguous,
        blocked,
        unverified,
        unrequested_rows,
        whole_suite_qualified: false,
    })
}
